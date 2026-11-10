use std::path::Path;

use config::{Config, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub sddm: SddmConfig,
    pub wallock: WallockConfig,
}

#[derive(Debug, Deserialize)]
pub struct SddmConfig {
    pub target_path: String,
}

#[derive(Debug, Deserialize)]
pub struct WallockConfig {
    pub target_path: String,
}

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    pub log_file: Option<String>,
    pub wallpapers_dir: String,
    pub wallpaperengine_config_file: String,
    pub played_history_db: String,
}

impl AppConfig {
    pub fn get_app_config(config_path: Option<&String>) -> AppConfig {
        let config_path =
            config_path.map_or_else(|| Path::new("config.toml"), |path| Path::new(path));
        Config::builder()
            .add_source(File::from(config_path))
            .build()
            .expect("Error add config file!")
            .try_deserialize::<AppConfig>()
            .expect("Error parse config file!")
    }
}
