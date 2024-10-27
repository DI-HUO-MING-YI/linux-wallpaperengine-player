use log::info;
use std::path::Path;
use std::process::Child;
use std::{thread, time};

use crate::player::config::wallpaperengine_config::WallpaperEngineConfig;
use crate::player::wallpaperengine;
use crate::util::kill_process;

use super::config::app_config::AppConfig;
use super::wallpaperengine::load_wallpaper;

pub fn play(app_config: &mut AppConfig, playlist_name: &String) {
    let wallpaperengine_config_file = app_config.general.wallpaperengine_config_file.clone();
    let wallpapers_dir = app_config.general.wallpapers_dir.clone();

    info!("play wallpaper list {playlist_name} new");
    let wallpaper_engine_config =
        WallpaperEngineConfig::load_config_from(&wallpaperengine_config_file);

    let wallpapers_dir = Path::new(&wallpapers_dir);

    wallpaperengine::kill_all_wallpaperengine_process();
    let mut pre_processes: Vec<Child> = vec![];
    let current_wallpaper_id = app_config
        .general
        .current_wallpaper_id
        .clone()
        .unwrap_or("".to_string());
    let mut has_loaded_current_wallpaper = current_wallpaper_id == "";
    loop {
        let playlist = wallpaper_engine_config.load_playlist(playlist_name);
        let videosequence = playlist.videosequence;
        for wallpaper_id in playlist.wallpaper_ids.iter() {
            if !has_loaded_current_wallpaper {
                if &current_wallpaper_id == wallpaper_id {
                    has_loaded_current_wallpaper = true;
                } else {
                    continue;
                }
            }
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

            let file_name = wallpaperengine::get_wallpaper_file(&project_json.to_str().unwrap());

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

            if videosequence {
                let duration: f64 =
                    if wallpaperengine::is_video_wallpaper(&project_json.to_str().unwrap()) {
                        let file = wallpaper_dir.join(file_name);
                        wallpaperengine::get_video_duration(file.to_str().unwrap())
                    } else {
                        0.0
                    };
                // thread::sleep(time::Duration::from_secs(max(
                //     (playlist.delay * 60) as u64,
                //     duration as u64,
                // )));
                if duration == 0.0 {
                    thread::sleep(time::Duration::from_secs((playlist.delay * 60) as u64));
                } else {
                    thread::sleep(time::Duration::from_secs(duration as u64));
                }
            } else {
                thread::sleep(time::Duration::from_secs((playlist.delay * 60) as u64));
            }
        }
        has_loaded_current_wallpaper = true;
    }
}
