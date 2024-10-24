use std::fs;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use log::{error, info, warn};

struct LinuxWallpaperengineConfig {
    screen_root: Option<Vec<String>>,
    scaling: Option<String>,
    clamping: Option<String>,
    silent: bool,
    volume: Option<u32>,
    noautomute: bool,
    window: bool,
    fps: Option<u32>,
    assets_dir: Option<String>,
    screenshot: bool,
    list_properties: bool,
    set_property: Option<std::collections::HashMap<String, String>>,
    no_fullscreen_pause: bool,
    disable_mouse: bool,
}

fn kill_wallpaperengine_process() {
    let result = Command::new("sh")
            .arg("-c")
            .arg("ps aux | grep \"linux-wallpaperengine\" | grep -v \"grep\" | grep -v \"player.py\" | awk '{print $2}' | xargs kill -9")
            .status();

    match result {
        Ok(status) if status.success() => info!("Killed all linux-wallpaperengine process"),
        _ => info!("Can not kill the linux-wallpaperengine process"),
    }
}

fn load_wallpaper(
    id: &str,
    wallpaper_dir: &str,
    log_path: Option<&str>,
    config: &LinuxWallpaperengineConfig,
    current_process: &mut Vec<u32>,
) {
    info!("start load_wallpaper");
    let wallpaper_path = format!("{}/{}", wallpaper_dir, id);
    if fs::metadata(&wallpaper_path).is_ok() {
        kill_wallpaperengine_process();
        info!("loading wallpaper: {}", id);

        let screen_roots = match &config.screen_root {
            Some(roots) => roots,
            None => {
                warn!("No screen root config found. Exit.");
                return;
            }
        };

        for (i, screen_root) in screen_roots.iter().enumerate() {
            play_on_screen_root(config, screen_root, i, id, log_path, current_process);
            sleep(Duration::from_millis(500));
        }
    } else {
        error!("Wallpaper {} not found", id);
    }
}

fn play_on_screen_root(
    config: &LinuxWallpaperengineConfig,
    screen_root: &String,
    i: usize,
    id: &str,
    log_path: Option<&str>,
    current_process: &mut Vec<u32>,
) {
    let mut command = Command::new("linux-wallpaperengine");
    if let Some(scaling) = &config.scaling {
        command.arg("--scaling").arg(scaling);
    }
    if let Some(clamping) = &config.clamping {
        command.arg("--clamping").arg(clamping);
    }
    command.arg("--screen-root").arg(screen_root);

    if i == 0 {
        if config.silent {
            command.arg("--silent");
        }
        if let Some(volume) = config.volume {
            command.arg("--volume").arg(volume.to_string());
        }
        if config.noautomute {
            command.arg("--noautomute").arg("--no-audio-processing");
        }
    } else {
        command.arg("--silent");
    }

    if config.window {
        command.arg("--window");
    }
    if let Some(fps) = config.fps {
        command.arg("--fps").arg(fps.to_string());
    }
    if let Some(assets_dir) = &config.assets_dir {
        command.arg("--assets-dir").arg(assets_dir);
    }
    if config.screenshot {
        command.arg("--screenshot");
    }
    if config.list_properties {
        command.arg("--list-propertites");
    }
    if let Some(set_properties) = &config.set_property {
        for (key, value) in set_properties {
            command
                .arg("--set-property")
                .arg(format!("{}={}", key, value));
        }
    }
    if config.no_fullscreen_pause {
        command.arg("--no-fullscreen-pause");
    }
    if config.disable_mouse {
        command.arg("--disable-mouse");
    }

    command.arg(id);

    info!("Execute command: {:?}", command);
    if let Some(log_output) = log_path {
        let log_output = format!("{}_in_{}", log_output, screen_root);
        let log_file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(log_output);

        if let Ok(mut log_file) = log_file {
            let process = command
                .stdout(Stdio::from(log_file.try_clone().unwrap()))
                .stderr(Stdio::from(log_file))
                .spawn();

            if let Ok(child) = process {
                current_process.push(child.id());
            }
        }
    }
}
