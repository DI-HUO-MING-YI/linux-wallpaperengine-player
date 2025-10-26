use std::{fs, io, path::Path};

use chrono::Local;
use log::info;
use serde_json::{json, Map, Value};

use crate::{
    player::wallpaperengine::{kill_all_wallpaperengine_process, load_wallpaper},
    util::kill_process,
};

use super::config::{
    app_config::AppConfig,
    wallpaperengine_config::{Folder, WallpaperEngineConfig},
};

pub fn check(app_config: &AppConfig) {
    back_config_file(&app_config.general.wallpaperengine_config_file);
    let wallpaper_engine_config =
        WallpaperEngineConfig::load_config_from(&app_config.general.wallpaperengine_config_file);
    let folders = wallpaper_engine_config.get_folders();

    kill_all_wallpaperengine_process();

    for folder in folders {
        let mut input = String::new();
        let title = &folder.title;
        loop {
            info!("Whether to check this folder: {title}? (y/n)");
            io::stdin().read_line(&mut input).unwrap();
            match input.to_lowercase().trim() {
                "y" => break,
                "n" => break,
                "" => break,
                _ => {
                    input = String::new();
                    println!("Invalid input! Please enter y, n, or just press Enter.")
                }
            }
        }

        if input.to_lowercase().trim() == "y".to_string() {
            info!("To check folder: {title}");
            let (picked, skipped, remained) = check_items(&folder, &app_config);

            update_config_file(
                picked,
                skipped,
                remained,
                title,
                &app_config.general.wallpaperengine_config_file,
            );
        }
    }
}

fn back_config_file(wallpaperengine_config_file: &str) {
    let original_path = Path::new(wallpaperengine_config_file);
    let backup_path =
        wallpaperengine_config_file.to_string() + ".back-" + &Local::now().to_string();
    let backup_path = Path::new(&backup_path);

    fs::copy(&original_path, &backup_path).expect("Backup config file error!");
}

fn update_config_file(
    picked: Vec<String>,
    skipped: Vec<String>,
    remained: Vec<String>,
    title: &String,
    config_file: &String,
) {
    let mut config: Value = fs::read_to_string(config_file)
        .map(|path| serde_json::from_str(&path))
        .expect(&format!(
            "Error to read wallpaerengine config json file: {config_file}!"
        ))
        .expect(&format!(
            "Error to read wallpaerengine config json file: {config_file}!"
        ));
    let folders = config
        .get_mut("steamuser")
        .expect("Node steamuser not found!")
        .get_mut("general")
        .expect("Node general not found !")
        .get_mut("browser")
        .expect("Node browser not found!")
        .get_mut("folders")
        .expect("Node folders not found!")
        .as_array_mut()
        .expect("Node folders is not an array");

    let mut picked_items = get_pre_folder(&folders, title, "_picked");
    let mut skipped_items = get_pre_folder(&folders, title, "_skipped");
    let mut remained_items = Map::new();

    for key in picked {
        picked_items.insert(key, json!(1));
    }
    for key in skipped {
        skipped_items.insert(key, json!(1));
    }
    for key in remained {
        remained_items.insert(key, json!(1));
    }
    let picked_folder = build_new_folder(title, "_picked", picked_items);
    let skipped_folder = build_new_folder(title, "_skipped", skipped_items);
    let remained_folder = build_new_folder(title, "", remained_items);

    folders.retain(|f| {
        f["title"] != title.to_string() + "_picked"
            && f["title"] != title.to_string() + "_skipped"
            && f["title"] != title.to_string()
    });

    folders.append(vec![picked_folder, skipped_folder, remained_folder].as_mut());

    fs::write(
        config_file,
        serde_json::to_string_pretty(&config).unwrap().as_bytes(),
    )
    .expect("Can not wirt to config file");
}

fn build_new_folder(title: &String, suffix: &str, items: Map<String, Value>) -> Value {
    json!(
            {
                "title": title.clone() + suffix,
                "subfolders": [],
                "items": items,
                "type": "folder",
            }
    )
}

fn get_pre_folder(folders: &Vec<Value>, title: &String, suffix: &str) -> Map<String, Value> {
    let pre_folder = folders
        .iter()
        .find(|f| f["title"] == title.to_string() + suffix);
    let items = if pre_folder == None {
        Map::new()
    } else {
        let mut map = Map::new();
        for f in pre_folder
            .unwrap()
            .get("items")
            .unwrap()
            .as_object()
            .unwrap()
        {
            map.insert(f.0.to_string(), json!(1));
        }
        map
    };
    items
}

fn get_wallpaper_type(wallpapers_path: &Path, wallpaper_id: &String) -> Option<String> {
    let project_json_path = wallpapers_path.join(wallpaper_id).join("project.json");
    if !project_json_path.exists() {
        return None;
    }

    match fs::read_to_string(&project_json_path) {
        Ok(content) => {
            match serde_json::from_str::<Value>(&content) {
                Ok(json) => {
                    if let Some(type_value) = json.get("type") {
                        if let Some(type_str) = type_value.as_str() {
                            return Some(type_str.to_lowercase());
                        }
                    }
                    None
                }
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

fn check_items(folder: &Folder, config: &AppConfig) -> (Vec<String>, Vec<String>, Vec<String>) {
    let wallpapers_path = Path::new(&config.general.wallpapers_dir);
    let mut input;
    let mut picked = vec![];
    let mut skipped = vec![];
    let mut remained = vec![];
    for wallpaper_id in &folder.items {
        if !fs::metadata(wallpapers_path.join(Path::new(wallpaper_id))).is_ok() {
            skipped.push(wallpaper_id.clone());
            continue;
        }

        // Check wallpaper type and pre-select based on config
        if let Some(wallpaper_type) = get_wallpaper_type(wallpapers_path, wallpaper_id) {
            if config.general.picked_types.contains(&wallpaper_type) {
                info!("Auto-picking wallpaper {} of type '{}'", wallpaper_id, wallpaper_type);
                picked.push(wallpaper_id.clone());
                continue;
            }
            if config.general.skipped_types.contains(&wallpaper_type) {
                info!("Auto-skipping wallpaper {} of type '{}'", wallpaper_id, wallpaper_type);
                skipped.push(wallpaper_id.clone());
                continue;
            }
        }

        let mut processes = load_wallpaper(wallpapers_path, &wallpaper_id, &config.play_command);

        if processes.is_empty() {
            continue;
        }
        loop {
            input = String::new();
            info!("Whether to pick this wallpaper: {wallpaper_id}? (y/n)");
            io::stdin().read_line(&mut input).unwrap();
            match input.to_lowercase().trim() {
                "y" => break,
                "n" => break,
                "" => break,
                _ => println!("Invalid input! Please enter y, n, or just press Enter."),
            }
        }

        if input.to_lowercase().trim() == "y" {
            picked.push(wallpaper_id.clone());
        } else if input.to_lowercase().trim() == "n" {
            skipped.push(wallpaper_id.clone());
        } else {
            remained.push(wallpaper_id.clone());
        }

        for process in processes.as_mut_slice() {
            kill_process(process);
        }
    }

    (picked, skipped, remained)
}
