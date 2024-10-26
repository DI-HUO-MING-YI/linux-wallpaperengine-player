pub(crate) use std::fs;
use std::path::Path;

use rand::seq::SliceRandom;
use serde::Deserialize;
use serde_json::Value;

use crate::{player::wallpaper, util::extract_last_directory_name};

#[derive(Debug, Deserialize)]
pub struct WallpaperEngineConfig {
    pub config: Value,
    pub playlist: Option<Playlist>,
    pub profile: Option<Profile>,
}

#[derive(Debug, Deserialize)]
pub struct Profile {
    pub wallpaper_id: String,
}

#[derive(Debug, Deserialize)]
pub struct Playlist {
    pub wallpapers: Vec<String>,
    pub mode: String,
    pub order: String,
    pub delay: u64,
}

impl WallpaperEngineConfig {
    pub fn load_config_from(config_path: &String) -> Self {
        let config: Value = fs::read_to_string(config_path)
            .map(|path| serde_json::from_str(&path))
            .expect(&format!(
                "Error to read wallpaerengine config json file: {config_path}!"
            ))
            .expect(&format!(
                "Error to read wallpaerengine config json file: {config_path}!"
            ));
        Self {
            config,
            playlist: None,
            profile: None,
        }
    }
    pub fn load_profile(&mut self, profile_name: &String) {
        let profile = self.get_profile(profile_name);
        let wallpaperid = profile
            .get("selectedwallpapers")
            .expect("Node selectedwallpapers not found")
            .get("Monitor0")
            .expect("Node Monitor0 not found")
            .get("file")
            .expect("Node file not found")
            .as_str()
            .expect("Node file is not a string");
        let wallpaper_id = Path::new(&wallpaperid)
            .parent()
            .expect("Not found dir name from file")
            .file_name()
            .expect("Not found dir name from file")
            .to_str()
            .expect("Can not covert string")
            .to_string();
        self.profile = Some(Profile { wallpaper_id });
    }
    pub fn load_playlist(&mut self, playlist_name: &String) {
        let playlist = self.get_playlist(playlist_name);
        let settings = playlist.get("settings").expect("Node settings not found!");
        let order = Self::get_playlist_order(settings);
        let delay = Self::get_playlist_delay(settings);
        let mode = Self::get_playlist_mode(settings);

        let mut wallpapers: Vec<String> = Self::get_wallpaper_ids(playlist);
        if order == "random" {
            let mut rng = rand::thread_rng();
            wallpapers.shuffle(&mut rng);
        }

        self.playlist = Some(Playlist {
            wallpapers,
            mode,
            order,
            delay,
        });
    }
    fn get_playlist_mode(settings: &Value) -> String {
        settings
            .get("mode")
            .expect("Node mode not found!")
            .as_str()
            .expect("Node mode is not a string!")
            .to_string()
    }
    fn get_playlist_delay(settings: &Value) -> u64 {
        settings
            .get("delay")
            .expect("Node delay not found!")
            .as_u64()
            .expect("Node delay is not an u64!")
    }
    fn get_playlist_order(settings: &Value) -> String {
        settings
            .get("order")
            .expect("Node order not found!")
            .as_str()
            .expect("Node order not found!")
            .to_string()
    }

    fn get_wallpaper_ids(playlist: &Value) -> Vec<String> {
        playlist
            .get("items")
            .expect("Node items not found!")
            .as_array()
            .expect("Node items is not an array")
            .iter()
            .map(|item| item.as_str().expect("Item is not a string!").to_string())
            .map(|path| {
                extract_last_directory_name(&path).expect("Wallpaper path has no wallpaper id!")
            })
            .collect()
    }

    fn get_playlist(&self, playlist_name: &String) -> &Value {
        self.config
            .get("steamuser")
            .expect("Node steamuser not found!")
            .get("general")
            .expect("Node general not found !")
            .get("playlists")
            .expect("Node playlists not found!")
            .as_array()
            .expect("Node playlists is not an array")
            .iter()
            .find(|p| p.get("name").map(|name| name.as_str()) == Some(Some(playlist_name)))
            .expect(&format!("No such playlist named {}", &playlist_name))
    }

    fn get_profile(&self, profile_name: &str) -> &Value {
        self.config
            .get("steamuser")
            .expect("Node steamuser not found!")
            .get("general")
            .expect("Node general not found !")
            .get("profiles")
            .expect("Node profiles not found!")
            .as_array()
            .expect("Node profiles is not an array")
            .iter()
            .find(|p| p.get("name").map(|name| name.as_str()) == Some(Some(profile_name)))
            .expect(&format!("No such playlist named {}", &profile_name))
    }
}
