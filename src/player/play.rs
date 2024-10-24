use log::info;
use rand::seq::SliceRandom;
use serde_json::Value;
use std::path::Path;
use std::{thread, time, usize};

use crate::player::config::wallpaperengine_config::WallpaperEngineConfig;

use super::config::app_config::AppConfig;
use super::config::wallpaperengine_config::Playlist;

pub fn play(playlist_name: &String, app_config: &AppConfig) {
    info!("play wallpaper list {playlist_name} new");
    let mut wallpaper_engine_config =
        WallpaperEngineConfig::load_config_from(&app_config.general.wallpaperengine_config_file);
    wallpaper_engine_config.load_playlist(playlist_name);
    let playlist = wallpaper_engine_config.playlist.unwrap();

    let wallpaper_dir = Path::new(&app_config.general.wallpapers_dir);

    for wallpaper_id in playlist.wallpapers {
        let item_dir = wallpaper_dir.join(wallpaper_id);
        let item_project_json = item_dir.join("project.json");

        if !item_dir.exists() && !item_project_json.exists() {
            info!("wallpaper {} not found.", wallpaper_id);
            continue;
        }

        load_wallpaper(wallpaper_id, wallpaper_dir);

        thread::sleep(time::Duration::from_secs((delay * 60) as u64));
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
