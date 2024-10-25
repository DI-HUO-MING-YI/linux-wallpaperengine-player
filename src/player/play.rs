use log::info;
use std::path::Path;
use std::process::{Child, Command};
use std::{thread, time};

use crate::player::config::wallpaperengine_config::WallpaperEngineConfig;
use crate::util::kill_process;

use super::config::app_config::AppConfig;
use super::wallpaper::load_wallpaper;

pub fn play(app_config: &AppConfig, playlist_name: &String) {
    info!("play wallpaper list {playlist_name} new");
    let mut wallpaper_engine_config =
        WallpaperEngineConfig::load_config_from(&app_config.general.wallpaperengine_config_file);
    wallpaper_engine_config.load_playlist(playlist_name);

    let playlist = wallpaper_engine_config.playlist.unwrap();
    let wallpapers_dir = Path::new(&app_config.general.wallpapers_dir);

    kill_all_wallpaperengine_process();
    let mut pre_processes: Vec<Child> = vec![];
    for wallpaper_id in playlist.wallpapers.iter() {
        let wallpaper_dir = wallpapers_dir.join(wallpaper_id);
        let project_json = wallpaper_dir.join("project.json");

        if !wallpaper_dir.exists() || !project_json.exists() {
            info!("wallpaper {} not found.", wallpaper_id);
            continue;
        }

        let child_processes =
            load_wallpaper(&wallpaper_dir, &wallpaper_id, &app_config.play_command);
        for p in pre_processes[..].as_mut().into_iter() {
            info!("Try to kill process: {:#?}!", &p.id());
            kill_process(p);
        }
        pre_processes = child_processes;

        thread::sleep(time::Duration::from_secs((playlist.delay * 60) as u64));
    }
}

fn kill_all_wallpaperengine_process() {
    let result = Command::new("sh")
            .arg("-c")
            .arg("ps aux | grep \"linux-wallpaperengine\" | grep -v \"grep\" | grep -v \"linux-wallpaperengine-player\" | awk '{print $2}' | xargs kill -9")
            .status();

    match result {
        Ok(status) if status.success() => info!("Killed all linux-wallpaperengine process"),
        _ => info!("Can not kill the linux-wallpaperengine process"),
    }
}
