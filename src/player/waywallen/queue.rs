use std::{env, fs, path::PathBuf};

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use zbus::blocking::Connection;

use super::{next, request, waywallen_proxy, WaywallenWebSocket};

#[derive(Serialize, Deserialize)]
struct QueuePauseState {
    rotation_secs: u32,
}

struct QueueStatus {
    interval_secs: u32,
    count: u32,
}

pub fn pause() -> Result<()> {
    if has_active_named_playlist()? {
        return Ok(());
    }

    let status = queue_status()?;
    if status.count == 0 || status.interval_secs == 0 {
        return Ok(());
    }

    let state = QueuePauseState {
        rotation_secs: status.interval_secs,
    };
    save_state(&state)?;
    if let Err(error) = set_rotation_interval(0) {
        if let Err(cleanup_error) = remove_state() {
            eprintln!("Failed to remove the queue pause state after an error: {cleanup_error:#}");
        }
        return Err(error);
    }
    println!("Waywallen global queue paused");
    Ok(())
}

pub fn resume() -> Result<()> {
    let Some(state) = read_state()? else {
        return Ok(());
    };
    ensure!(
        state.rotation_secs > 0,
        "Queue pause state has an invalid zero interval"
    );
    if has_active_named_playlist()? {
        return Ok(());
    }

    let status = queue_status()?;
    if status.count == 0 {
        return Ok(());
    }
    if status.interval_secs != 0 {
        remove_state()?;
        return Ok(());
    }

    set_rotation_interval(state.rotation_secs)?;
    next()?;
    remove_state().context("Queue resumed, but failed to remove the pause state")?;
    println!("Waywallen global queue resumed");
    Ok(())
}

fn queue_status() -> Result<QueueStatus> {
    let connection = Connection::session().context("Failed to connect to the session bus")?;
    let proxy = waywallen_proxy(&connection)?;
    let (_, _, interval_secs, _, _, count, _): (i64, String, u32, String, u32, u32, bool) =
        proxy.call("QueueStatus", &())?;
    Ok(QueueStatus {
        interval_secs,
        count,
    })
}

fn set_rotation_interval(secs: u32) -> Result<()> {
    let connection = Connection::session().context("Failed to connect to the session bus")?;
    waywallen_proxy(&connection)?.call::<_, _, ()>("SetRotationInterval", &(secs,))?;
    Ok(())
}

fn has_active_named_playlist() -> Result<bool> {
    let mut websocket = WaywallenWebSocket::connect()?;
    let response = websocket.call(request::Payload::PlaylistStatus(PlaylistStatusRequest {}))?;
    let status = response
        .playlist_status
        .context("Waywallen returned no playlist status")?;
    Ok(status.displays.iter().any(|display| display.active_id > 0))
}

fn state_path() -> Result<PathBuf> {
    let state_home = match env::var_os("XDG_STATE_HOME") {
        Some(path) if !path.is_empty() => PathBuf::from(path),
        _ => {
            let home = env::var_os("HOME").context("HOME is not set")?;
            PathBuf::from(home).join(".local").join("state")
        }
    };
    Ok(state_home
        .join("linux-wallpaperengine-player")
        .join("queue-pause.json"))
}

fn save_state(state: &QueuePauseState) -> Result<()> {
    let path = state_path()?;
    let parent = path.parent().context("Queue pause state has no parent")?;
    fs::create_dir_all(parent)?;
    let temporary_path = path.with_extension("json.tmp");
    fs::write(&temporary_path, serde_json::to_vec(state)?)?;
    if let Err(error) = fs::rename(&temporary_path, &path) {
        let _ = fs::remove_file(&temporary_path);
        return Err(error.into());
    }
    Ok(())
}

fn read_state() -> Result<Option<QueuePauseState>> {
    match fs::read(state_path()?) {
        Ok(data) => Ok(Some(serde_json::from_slice(&data)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn remove_state() -> Result<()> {
    let path = state_path()?;
    match fs::remove_file(&path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    }

    if let Some(parent) = path.parent() {
        match fs::remove_dir(parent) {
            Ok(()) => {}
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::DirectoryNotEmpty
                ) => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

#[derive(prost::Message)]
pub(super) struct PlaylistStatusRequest {}

#[derive(prost::Message)]
pub(super) struct PlaylistStatusResponse {
    #[prost(message, repeated, tag = "1")]
    pub displays: Vec<PlaylistDisplayStatus>,
}

#[derive(prost::Message)]
pub(super) struct PlaylistDisplayStatus {
    #[prost(int64, tag = "2")]
    pub active_id: i64,
}
