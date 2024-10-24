use log::info;
use std::path::Path;
use std::{thread, time};

use crate::player::config::wallpaperengine_config::WallpaperEngineConfig;

use super::config::app_config::AppConfig;
use super::linux_wallpaperengine::load_wallpaper;

pub fn play(playlist_name: &String, app_config: &AppConfig) {
    info!("play wallpaper list {playlist_name} new");
    let mut wallpaper_engine_config =
        WallpaperEngineConfig::load_config_from(&app_config.general.wallpaperengine_config_file);
    wallpaper_engine_config.load_playlist(playlist_name);
    let playlist = wallpaper_engine_config.playlist.unwrap();

    let wallpapers_dir = Path::new(&app_config.general.wallpapers_dir);

    for wallpaper_id in playlist.wallpapers.iter() {
        let wallpaper_dir = wallpapers_dir.join(wallpaper_id);
        let project_json = wallpaper_dir.join("project.json");

        if !wallpaper_dir.exists() || !project_json.exists() {
            info!("wallpaper {} not found.", wallpaper_id);
            continue;
        }

        load_wallpaper(&wallpaper_dir, &wallpaper_id, &app_config.play_command);

        thread::sleep(time::Duration::from_secs((playlist.delay * 60) as u64));
    }
}

fn get_wallpaper_id(item: &str) -> &str {
    Path::new(item)
        .parent()
        .unwrap()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
}
