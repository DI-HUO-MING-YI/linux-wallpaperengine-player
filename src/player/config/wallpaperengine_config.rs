use std::fs;
use std::path::Path;

use rand::seq::SliceRandom;
use serde::Deserialize;
use serde_json::Value;

use crate::util::extract_last_directory_name;

#[derive(Debug, Deserialize)]
pub struct WallpaperEngineConfig {
    pub source: Value,
    pub profile: Option<Profile>,
}

#[derive(Debug, Deserialize)]
pub struct Profile {
    pub wallpaper_id: String,
}

pub struct Playlist {
    pub wallpaper_ids: Vec<String>,
    pub mode: String,
    pub order: String,
    pub delay: u64,
    pub videosequence: bool,
    pub beginfirst: bool,
}

#[derive(Clone)]
pub struct Folder {
    pub items: Vec<String>,
    pub title: String,
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
            source: config,
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

    pub fn load_playlist(
        &self,
        playlist_name: &String,
        current_wallpaer_id: &Option<String>,
    ) -> Playlist {
        let playlist = self.get_playlist(playlist_name);
        let settings = playlist.get("settings").expect("Node settings not found!");
        let order = Self::get_playlist_order(settings);
        let delay = Self::get_playlist_delay(settings);
        let mode = Self::get_playlist_mode(settings);
        let videosequence = Self::get_playlist_videosequence(settings);
        let beginfirst = Self::get_playlist_beginfirst(settings);

        let mut wallpaper_ids: Vec<String> = Self::get_wallpaper_files(playlist);
        if order == "random" {
            let mut rng = rand::thread_rng();
            wallpaper_ids.shuffle(&mut rng);
        }

        if !beginfirst {
            if let Some(current_wallpaer_id) = current_wallpaer_id {
                wallpaper_ids = match wallpaper_ids.iter().position(|i| i == current_wallpaer_id) {
                    Some(index) => {
                        let (left, right) = wallpaper_ids.split_at(index);
                        [right, left].concat()
                    }
                    None => wallpaper_ids,
                };
            }
        }

        Playlist {
            wallpaper_ids,
            mode,
            order,
            delay,
            videosequence,
            beginfirst,
        }
    }

    pub fn get_folders(&self) -> Vec<Folder> {
        self.source
            .get("steamuser")
            .expect("Node steamuser not found!")
            .get("general")
            .expect("Node general not found !")
            .get("browser")
            .expect("Node browser not found!")
            .get("folders")
            .expect("Node folders not found!")
            .as_array()
            .expect("Node folders is not an array")
            .iter()
            .map(|f| Folder {
                title: f
                    .get("title")
                    .and_then(|t| t.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                items: f
                    .get("items")
                    .and_then(|items| items.as_object())
                    .map(|obj| obj.keys().cloned().collect())
                    .unwrap_or_else(Vec::new),
            })
            .collect()
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

    fn get_wallpaper_files(playlist: &Value) -> Vec<String> {
        playlist
            .get("items")
            .expect("Node items not found!")
            .as_array()
            .expect("Node items is not an array")
            .iter()
            .map(|item| item.as_str().expect("Item is not a string!").to_string())
            .filter(|path| !&path.is_empty())
            .map(|path| {
                extract_last_directory_name(&path).expect(&format!("Wallpaper path <{}> has no wallpaper id!", path))
            })
            .collect()
    }

    fn get_playlist(&self, playlist_name: &String) -> &Value {
        self.source
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
        self.source
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

    fn get_playlist_videosequence(settings: &Value) -> bool {
        settings
            .get("videosequence")
            .map_or(false, |it| it.as_bool().unwrap_or(false))
    }
    fn get_playlist_beginfirst(settings: &Value) -> bool {
        settings
            .get("beginfirst")
            .map_or(false, |it| it.as_bool().unwrap_or(false))
    }

    pub fn get_wallpaper_fodler(&self, folder_name: &str) -> Option<Folder> {
        self.get_folders()
            .iter()
            .find(|f| f.title == folder_name)
            .cloned()
    }
}
