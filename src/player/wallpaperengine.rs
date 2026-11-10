use std::{fs, path::Path};

use serde_json::Value;

pub fn is_video_wallpaper(project_json: &str) -> bool {
    project_string_field(project_json, "type")
        .is_some_and(|wallpaper_type| wallpaper_type.to_lowercase().trim() == "video")
}

pub fn get_wallpaper_file(project_json: &str) -> String {
    project_string_field(project_json, "file").unwrap_or_else(|| "unknown".to_string())
}

fn project_string_field(project_json: &str, field: &str) -> Option<String> {
    let contents = fs::read_to_string(Path::new(project_json)).ok()?;
    let project = serde_json::from_str::<Value>(&contents).ok()?;
    project.get(field)?.as_str().map(str::to_string)
}
