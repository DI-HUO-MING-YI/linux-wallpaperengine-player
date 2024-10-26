use std::{path::Path, sync::mpsc::channel, thread, time::Duration};

use log::info;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::{
    player::{
        config::wallpaperengine_config::WallpaperEngineConfig,
        wallpaper::{self, load_wallpaper},
    },
    util::kill_process,
};

use super::{
    config::app_config::{self, AppConfig},
    wallpaper::kill_all_wallpaperengine_process,
};

pub fn watch(app_config: &AppConfig, profile_name: &String) {
    info!("Watch wallpaper profile {profile_name} new");
    let mut wallpaper_engine_config =
        WallpaperEngineConfig::load_config_from(&app_config.general.wallpaperengine_config_file);
    let wallpapers_dir = Path::new(&app_config.general.wallpapers_dir);
    wallpaper_engine_config.load_profile(profile_name);

    let wallpaper_id = wallpaper_engine_config.profile.unwrap().wallpaper_id;
    kill_all_wallpaperengine_process();
    let mut pre_processes = load_wallpaper(wallpapers_dir, &wallpaper_id, &app_config.play_command);

    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();

    watcher
        .watch(
            Path::new(&app_config.general.wallpaperengine_config_file),
            RecursiveMode::Recursive,
        )
        .unwrap();

    println!("正在监控文件的修改...");

    let mut pre_wallpaper_id = wallpaper_id;
    // 循环接收文件事件
    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if let EventKind::Modify(_) = event.kind {
                    thread::sleep(Duration::from_millis(500));
                    wallpaper_engine_config = WallpaperEngineConfig::load_config_from(
                        &app_config.general.wallpaperengine_config_file,
                    );
                    let wallpapers_dir = Path::new(&app_config.general.wallpapers_dir);
                    wallpaper_engine_config.load_profile(profile_name);

                    let wallpaper_id = wallpaper_engine_config.profile.unwrap().wallpaper_id;
                    if pre_wallpaper_id != wallpaper_id {
                        for p in pre_processes[..].as_mut().into_iter() {
                            info!("Try to kill process: {:#?}!", &p.id());
                            kill_process(p);
                        }
                        pre_processes =
                            load_wallpaper(wallpapers_dir, &wallpaper_id, &app_config.play_command);
                        pre_wallpaper_id = wallpaper_id.to_string();
                    }
                }
            }
            Ok(Err(e)) => println!("监控错误: {:?}", e),
            Err(_) => println!("超时，没有文件变动"),
        }
    }
}
