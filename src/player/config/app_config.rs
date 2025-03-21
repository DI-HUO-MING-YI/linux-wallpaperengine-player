use core::f64;
use std::{fs, path::Path};

use config::{Config, File};
use regex::Regex;
use serde::Deserialize;

use crate::player::control::{self, ControlAction, PlayState};

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(skip)]
    pub config_path: String,
    pub general: GeneralConfig,
    pub sddm: SddmConfig,
    pub wallock: WallockConfig,
    pub play_command: PlayCommandConfig,
}

#[derive(Debug, Deserialize)]
pub struct SddmConfig {
    pub pre_sddm_wallpaper_id: Option<String>,
    pub target_path: String,
}

#[derive(Debug, Deserialize)]
pub struct WallockConfig {
    pub pre_wallock_wallpaper_id: Option<String>,
    pub target_path: String,
}

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    pub mode: Option<String>,
    pub current_wallpaper_id: Option<String>,
    pub play_state: Option<String>,
    pub min_delay: Option<f64>,
    pub max_delay: Option<f64>,
    pub log_file: Option<String>,
    pub picked_types: Vec<String>,
    pub skipped_types: Vec<String>,
    pub play_list_name: String,
    pub wallpapers_dir: String,
    pub wallpaperengine_config_file: String,
    pub played_history_db: String,
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

    pub fn get_delay_range(&self) -> (f64, f64) {
        (
            self.general.min_delay.unwrap_or(f64::MIN),
            self.general.max_delay.unwrap_or(f64::MAX),
        )
    }
    pub fn save_current_wallpaper(&mut self, wallpaper_id: &String, wallpaper_name: &String) {
        let contents = fs::read_to_string(&self.config_path).expect("Can not open config file.");

        let re = Regex::new(r#"(?m)^current_wallpaper_id\s*=\s*"(.*)"#).unwrap();

        let modified_contents = re.replace_all(
            &contents,
            &format!(r#"current_wallpaper_id = "{}""#, wallpaper_id),
        );

        let re = Regex::new(r#"(?m)^current_wallpaper_name\s*=\s*"(.*)"#).unwrap();

        let modified_contents = re.replace_all(
            &modified_contents,
            &format!(
                r#"current_wallpaper_name = "{}""#,
                wallpaper_name.replace("\"", "\\\"")
            ),
        );
        fs::write(&self.config_path, modified_contents.as_bytes())
            .expect("Can not write into config file");
    }

    pub fn save_play_state(&mut self, action: &control::ControlAction) {
        let contents = fs::read_to_string(&self.config_path).expect("Can not open config file.");

        let re = Regex::new(r#"(?m)^play_state\s*=\s*"(.*)"#).unwrap();

        let new_state = ControlAction::to_state(self.general.play_state.clone(), &action);
        let modified_contents =
            re.replace_all(&contents, &format!(r#"play_state = "{}""#, &new_state));
        fs::write(&self.config_path, modified_contents.as_bytes())
            .expect("Can not write into config file");
        self.general.play_state = Some(new_state.to_string());
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

    pub fn get_current_wallpaper(&self) -> Option<String> {
        self.general.current_wallpaper_id.clone()
    }
}
