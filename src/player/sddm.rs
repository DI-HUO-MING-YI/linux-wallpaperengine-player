use std::{fs, path::Path};

use log::info;
use rand::{seq::SliceRandom, thread_rng};

use crate::player::config::wallpaperengine_config::WallpaperEngineConfig;

use super::{
    config::app_config::AppConfig,
    wallpaperengine::{get_wallpaper_file, is_video_wallpaper},
};

pub fn sddm(app_config: &mut AppConfig, folder_name: &String) {
    let wallpapers_dir = &app_config.general.wallpapers_dir;
    let wallpapers_dir = Path::new(&wallpapers_dir);
    let wallpaperengine_config_file = app_config.general.wallpaperengine_config_file.clone();

    let wallpaper_engine_config =
        WallpaperEngineConfig::load_config_from(&wallpaperengine_config_file);
    let folder = wallpaper_engine_config
        .get_wallpaper_fodler(folder_name)
        .expect(&format!("No such folder named: {folder_name}."));
    let wallpaper_ids = folder.items;
    for _ in 0..wallpaper_ids.len() {
        let mut rng = thread_rng();
        let wallpaper_id = wallpaper_ids.choose(&mut rng).expect("Folder is empty!");
        let project_json = wallpapers_dir
            .join(Path::new(&wallpaper_id))
            .join(Path::new("project.json"));

        if fs::metadata(&project_json).is_ok() {
            if is_video_wallpaper(project_json.to_str().unwrap()) {
                let wallpaper_file = get_wallpaper_file(project_json.to_str().unwrap());
                let wallpaper_file = wallpapers_dir
                    .join(Path::new(&wallpaper_id))
                    .join(Path::new(&wallpaper_file));

                info!("wallaper_file: {wallpaper_id}");
                if fs::metadata(&wallpaper_file).is_ok() {
                    let target_path = Path::new(&app_config.sddm.target_path);

                    fs::copy(&wallpaper_file, target_path).expect("Can not copy to sddm path!");
                    app_config.save_pre_sddm_wallpaper(wallpaper_id);
                    return;
                }
            }
        }
    }
}
