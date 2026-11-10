use std::{fs, path::Path};

use config::{Config, File};
use regex::Regex;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(skip)]
    pub config_path: String,
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
        let mut config = Config::builder()
            .add_source(File::from(config_path))
            .build()
            .expect("Error add config file!")
            .try_deserialize::<AppConfig>()
            .expect("Error parse config file!");
        config.config_path = config_path.to_string_lossy().to_string();
        config
    }

    pub fn save_pre_sddm_wallpaper(&mut self, wallpaper_id: &String) {
        let contents = fs::read_to_string(&self.config_path).expect("Can not open config file.");

        let re = Regex::new(r#"(?m)^pre_sddm_wallpaper_id\s*=\s*"(.*)"#).unwrap();

        let modified_contents = re.replace_all(
            &contents,
            &format!(r#"pre_sddm_wallpaper_id = "{}""#, wallpaper_id),
        );
        fs::write(&self.config_path, modified_contents.as_bytes())
            .expect("Can not write into config file");
    }

    pub fn save_pre_walllock_wallpaper(&mut self, wallpaper_id: &String) {
        let contents = fs::read_to_string(&self.config_path).expect("Can not open config file.");

        let re = Regex::new(r#"(?m)^pre_wallock_wallpaper_id\s*=\s*"(.*)"#).unwrap();

        let modified_contents = re.replace_all(
            &contents,
            &format!(r#"pre_wallock_wallpaper_id = "{}""#, wallpaper_id),
        );
        fs::write(&self.config_path, modified_contents.as_bytes())
            .expect("Can not write into config file");
    }
}
