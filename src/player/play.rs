use log::info;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::{thread, time};

use crate::player::config::wallpaperengine_config::{Playlist, WallpaperEngineConfig};
use crate::player::wallpaperengine;
use crate::util::{kill_process, secs_to_nanos};

use super::config::app_config::AppConfig;
use super::wallpaperengine::load_wallpaper;

pub fn play(app_config: &mut AppConfig, playlist_name: &String) {
    let wallpaperengine_config_file = app_config.general.wallpaperengine_config_file.clone();
    let wallpapers_dir = app_config.general.wallpapers_dir.clone();
    let wallpapers_dir = Path::new(&wallpapers_dir);

    let wallpaper_engine_config =
        WallpaperEngineConfig::load_config_from(&wallpaperengine_config_file);

    wallpaperengine::kill_all_wallpaperengine_process();
    let mut pre_processes: Vec<Child> = vec![];
    let current_wallpaper = app_config.get_current_wallpaper();
    let playlist = wallpaper_engine_config.load_playlist(playlist_name, &current_wallpaper);
    let switch_mod = WallpaperSwitchMode::new(&playlist);
    let delay = playlist.delay;
    let (min_delay, max_delay) = app_config.get_delay_range();
    info!("play wallpaper list {playlist_name} now!");
    loop {
        for wallpaper_id in playlist.wallpaper_ids.iter() {
            let wallpaper_dir = wallpapers_dir.join(&wallpaper_id);
            let project_json = wallpaper_dir.join("project.json");

            if !wallpaper_dir.exists() || !project_json.exists() {
                info!(
                    "wallpaper {} not found in {}.",
                    &wallpaper_id,
                    wallpaper_dir.to_string_lossy()
                );
                continue;
            }

            let child_processes =
                load_wallpaper(&wallpaper_dir, &wallpaper_id, &app_config.play_command);

            if child_processes.is_empty() {
                continue;
            }

            app_config.save_current_wallpaper(&wallpaper_id);
            for p in pre_processes[..].as_mut().into_iter() {
                info!("Try to kill process: {:#?}!", &p.id());
                kill_process(p);
            }
            pre_processes = child_processes;

            wait_for_delay(
                &switch_mod,
                &delay,
                &project_json,
                &wallpaper_dir,
                min_delay,
                max_delay,
            )
        }
    }
}

enum WallpaperSwitchMode {
    Timer,
    Videosequence,
}

impl WallpaperSwitchMode {
    pub fn new(playlist: &Playlist) -> Self {
        if playlist.videosequence {
            WallpaperSwitchMode::Videosequence
        } else if playlist.mode == "timer" {
            WallpaperSwitchMode::Timer
        } else {
            WallpaperSwitchMode::Timer
        }
    }
}

fn wait_for_delay(
    switch_mode: &WallpaperSwitchMode,
    delay: &u64,
    project_json: &PathBuf,
    wallpaper_dir: &PathBuf,
    min_delay: f64,
    max_delay: f64,
) {
    let file_name = wallpaperengine::get_wallpaper_file(&project_json.to_str().unwrap());

    match switch_mode {
        WallpaperSwitchMode::Videosequence => {
            if wallpaperengine::is_video_wallpaper(&project_json.to_str().unwrap()) {
                let file = wallpaper_dir.join(file_name);
                let delay = wallpaperengine::get_video_duration(file.to_str().unwrap());
                let delay = secs_to_nanos(delay.min(max_delay).max(min_delay));
                thread::sleep(time::Duration::from_nanos(delay));
            } else {
                thread::sleep(time::Duration::from_secs((delay * 60) as u64));
            }
        }
        WallpaperSwitchMode::Timer => {
            thread::sleep(time::Duration::from_secs((delay * 60) as u64));
        }
    }
}
