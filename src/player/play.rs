use log::info;
use regex::Regex;
use std::path::Path;
use std::process::Child;
use std::{fs, thread, time};

use crate::player::config::wallpaperengine_config::WallpaperEngineConfig;
use crate::player::wallpaper::{self, kill_all_wallpaperengine_process};
use crate::util::kill_process;

use super::config::app_config::AppConfig;
use super::wallpaper::load_wallpaper;

pub fn play(app_config: &AppConfig, playlist_name: &String, config_path: &Path) {
    info!("play wallpaper list {playlist_name} new");
    let mut wallpaper_engine_config =
        WallpaperEngineConfig::load_config_from(&app_config.general.wallpaperengine_config_file);
    wallpaper_engine_config.load_playlist(playlist_name);

    let playlist = wallpaper_engine_config.playlist.unwrap();
    let wallpapers_dir = Path::new(&app_config.general.wallpapers_dir);

    kill_all_wallpaperengine_process();
    let mut pre_processes: Vec<Child> = vec![];
    let current_wallpaper_id = app_config
        .general
        .current_wallpaper_id
        .clone()
        .unwrap_or("".to_string());
    let mut has_loaded_current_wallpaper = current_wallpaper_id == "";
    loop {
        for wallpaper_id in playlist.wallpapers.iter() {
            if !has_loaded_current_wallpaper {
                if &current_wallpaper_id == wallpaper_id {
                    has_loaded_current_wallpaper = true;
                } else {
                    continue;
                }
            }

            let wallpaper_dir = wallpapers_dir.join(wallpaper_id);
            let project_json = wallpaper_dir.join("project.json");

            if !wallpaper_dir.exists() || !project_json.exists() {
                info!("wallpaper {} not found.", wallpaper_id);
                continue;
            }

            let child_processes =
                load_wallpaper(&wallpaper_dir, &wallpaper_id, &app_config.play_command);

            save_current_wallpaper(config_path, &wallpaper_id);

            for p in pre_processes[..].as_mut().into_iter() {
                info!("Try to kill process: {:#?}!", &p.id());
                kill_process(p);
            }
            pre_processes = child_processes;

            thread::sleep(time::Duration::from_secs((playlist.delay * 60) as u64));
        }
    }
}
fn save_current_wallpaper(config_path: &Path, wallpaper_id: &String) {
    let contents = fs::read_to_string(config_path).expect("Can not open config file.");

    let re = Regex::new(r#"(?m)^current_wallpaper_id\s*=\s*"(.*)"#).unwrap();

    let modified_contents = re.replace_all(
        &contents,
        &format!(r#"current_wallpaper_id = "{}""#, wallpaper_id),
    );
    fs::write(config_path, modified_contents.as_bytes()).expect("Can not write into config file");
}
