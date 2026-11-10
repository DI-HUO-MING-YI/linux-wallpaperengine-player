use std::fs;

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct WallpaperEngineConfig {
    pub source: Value,
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
        Self { source: config }
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
            .map(|folder| Folder {
                title: folder
                    .get("title")
                    .and_then(|title| title.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                items: folder
                    .get("items")
                    .and_then(|items| items.as_object())
                    .map(|items| items.keys().cloned().collect())
                    .unwrap_or_else(Vec::new),
            })
            .collect()
    }

    pub fn get_wallpaper_fodler(&self, folder_name: &str) -> Option<Folder> {
        self.get_folders()
            .iter()
            .find(|folder| folder.title == folder_name)
            .cloned()
    }
}
