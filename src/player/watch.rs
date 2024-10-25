use std::path::Path;

use log::info;

use crate::player::{
    config::wallpaperengine_config::WallpaperEngineConfig, wallpaper::load_wallpaper,
};

use super::config::app_config::AppConfig;

pub fn watch(app_config: &AppConfig, profile_name: &String) {
    info!("Watch wallpaper profile {profile_name} new");
    let mut wallpaper_engine_config =
        WallpaperEngineConfig::load_config_from(&app_config.general.wallpaperengine_config_file);
    let wallpapers_dir = Path::new(&app_config.general.wallpapers_dir);

    let wallpaper_id = "";
    load_wallpaper(wallpapers_dir, wallpaper_id, &app_config.play_command);
}
