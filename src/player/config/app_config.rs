use std::{env, fs, path::Path};

use config::{Config, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub play_command: PlayCommandConfig,
}

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    pub mode: Option<String>,
    pub current_wallpaper_id: Option<String>,
    pub interval: Option<usize>,
    pub log_file: Option<String>,
    pub picked_types: Vec<String>,
    pub skipped_types: Vec<String>,
    pub play_list_name: String,
    pub wallpapers_dir: String,
    pub wallpaperengine_config_file: String,
}

#[derive(Debug, Deserialize)]
pub struct PlayCommandConfig {
    base_command: String,
    log_file: Option<String>,
    silent: Option<bool>,
    volume: Option<usize>,
    noautomute: Option<bool>,
    no_audio_processing: Option<bool>,
    screen_root: Vec<String>,
    window: Option<bool>,
    fps: Option<usize>,
    assets_dir: Option<String>,
    screenshot: Option<bool>,
    list_propertites: Option<bool>,
    set_property: Option<Vec<String>>,
    no_fullscreen_pause: Option<bool>,
    disable_mouse: Option<bool>,
    scaling: Option<String>,
    clamping: Option<String>,
}

impl AppConfig {
    pub fn get_app_config(config_path: Option<&String>) -> AppConfig {
        let config_path =
            config_path.map_or_else(|| Path::new("./config.toml"), |path| Path::new(path));
        Config::builder()
            .add_source(File::from(config_path))
            .build()
            .expect("Error add config file!")
            .try_deserialize::<AppConfig>()
            .expect("Error parse config file!")
    }
}
