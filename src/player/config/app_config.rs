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
    pub play_command: PlayCommandConfig,
}

#[derive(Debug, Deserialize)]
pub struct SddmConfig {
    pub pre_sddm_wallpaper_id: Option<String>,
    pub target_path: String,
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

    pub fn save_current_wallpaper(&mut self, wallpaper_id: &String) {
        let contents = fs::read_to_string(&self.config_path).expect("Can not open config file.");

        let re = Regex::new(r#"(?m)^current_wallpaper_id\s*=\s*"(.*)"#).unwrap();

        let modified_contents = re.replace_all(
            &contents,
            &format!(r#"current_wallpaper_id = "{}""#, wallpaper_id),
        );
        fs::write(&self.config_path, modified_contents.as_bytes())
            .expect("Can not write into config file");
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
}
