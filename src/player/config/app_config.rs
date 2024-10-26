use std::path::Path;

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
    pub base_command: String,
    pub log_file: Option<String>,
    pub silent: Option<bool>,
    pub volume: Option<usize>,
    pub noautomute: Option<bool>,
    pub no_audio_processing: Option<bool>,
    pub screen_root: Vec<String>,
    pub window: Option<bool>,
    pub fps: Option<usize>,
    pub assets_dir: Option<String>,
    pub screenshot: Option<bool>,
    pub list_propertites: Option<bool>,
    pub set_property: Option<Vec<String>>,
    pub no_fullscreen_pause: Option<bool>,
    pub disable_mouse: Option<bool>,
    pub scaling: Option<String>,
    pub clamping: Option<String>,
}

impl AppConfig {
    pub fn get_app_config(config_path: Option<&String>) -> (AppConfig, &Path) {
        let config_path =
            config_path.map_or_else(|| Path::new("config.toml"), |path| Path::new(path));
        (
            Config::builder()
                .add_source(File::from(config_path))
                .build()
                .expect("Error add config file!")
                .try_deserialize::<AppConfig>()
                .expect("Error parse config file!"),
            config_path,
        )
    }
}
