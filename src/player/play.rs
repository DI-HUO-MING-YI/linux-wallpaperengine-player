use log::info;
use std::collections::VecDeque;
use std::path::Path;
use std::process::Child;
use std::str::FromStr;
use std::time;

use crate::player::config::wallpaperengine_config::WallpaperEngineConfig;
use crate::player::control::{self, ControlAction, PlayState};
use crate::player::wallpaperengine::{self, WallpaperSwitchMode};
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
    let switch_mode = WallpaperSwitchMode::new(&playlist);
    let delay = playlist.delay;
    let (min_delay, max_delay) = app_config.get_delay_range();
    info!("play wallpaper list {playlist_name} now!");
    let mut play_queue = VecDeque::from(playlist.wallpaper_ids);
    while let Some(wallpaper_id) = play_queue.pop_front() {
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

        play_queue.push_back(wallpaper_id.clone());
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

        let file_name = wallpaperengine::get_wallpaper_file(&project_json.to_str().unwrap());
        let delay = match &switch_mode {
            WallpaperSwitchMode::Videosequence => {
                if wallpaperengine::is_video_wallpaper(&project_json.to_str().unwrap()) {
                    let file = wallpaper_dir.join(file_name);
                    let delay = wallpaperengine::get_video_duration(file.to_str().unwrap());
                    let delay = secs_to_nanos(delay.min(max_delay).max(min_delay));
                    time::Duration::from_nanos(delay)
                } else {
                    time::Duration::from_secs(delay * 60)
                }
            }
            WallpaperSwitchMode::Timer => time::Duration::from_secs(delay * 60),
        };

        let mut stopped = app_config
            .general
            .play_state
            .clone()
            .map_or(false, |state| PlayState::is_stopped(&state));

        if let Some(message) = control::wait_for_control_message(&delay) {
            app_config.save_play_state(&message);
            match message {
                control::ControlAction::Next => continue,
                control::ControlAction::Prev => {
                    let pre_wallpaper = play_queue.pop_back().unwrap();
                    play_queue.push_front(pre_wallpaper);
                    let pre_wallpaper = play_queue.pop_back().unwrap();
                    play_queue.push_front(pre_wallpaper);
                }
                control::ControlAction::Reload => {
                    let pre_wallpaper = play_queue.pop_back().unwrap();
                    play_queue.push_front(pre_wallpaper);
                }
                control::ControlAction::Stop => stopped = true,
                control::ControlAction::Continue => continue,
            }
        }

        if stopped {
            loop {
                if let Some(message) =
                    control::wait_for_control_message(&time::Duration::from_nanos(u64::MAX))
                {
                    app_config.save_play_state(&message);
                    match message {
                        control::ControlAction::Next => break,
                        control::ControlAction::Prev => {
                            let pre_wallpaper = play_queue.pop_back().unwrap();
                            play_queue.push_front(pre_wallpaper);
                            let pre_wallpaper = play_queue.pop_back().unwrap();
                            play_queue.push_front(pre_wallpaper);
                            break;
                        }
                        control::ControlAction::Reload => {
                            let pre_wallpaper = play_queue.pop_back().unwrap();
                            play_queue.push_front(pre_wallpaper);
                            continue;
                        }
                        control::ControlAction::Stop => continue,
                        control::ControlAction::Continue => break,
                    }
                }
            }
        }
    }
}
