use std::fs::{self, File};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use log::{error, info};

use super::config::app_config::PlayCommandConfig;

fn kill_wallpaperengine_process() {
    let result = Command::new("sh")
            .arg("-c")
            .arg("ps aux | grep \"linux-wallpaperengine\" | grep -v \"grep\" | grep -v \"linux-wallpaperengine-player\" | awk '{print $2}' | xargs kill -9")
            .status();

    match result {
        Ok(status) if status.success() => info!("Killed all linux-wallpaperengine process"),
        _ => info!("Can not kill the linux-wallpaperengine process"),
    }
}

pub fn load_wallpaper(wallpaper_path: &Path, wallpaper_id: &str, config: &PlayCommandConfig) {
    info!("start load_wallpaper");
    if fs::metadata(wallpaper_path).is_ok() {
        kill_wallpaperengine_process();
        info!("loading wallpaper: {}", wallpaper_path.to_str().unwrap());

        for screen_root in &config.screen_root {
            play_on_screen_root(config, &screen_root, wallpaper_id);
            sleep(Duration::from_millis(500));
        }
    } else {
        error!("Wallpaper {} not found", wallpaper_path.to_str().unwrap());
    }
}

fn play_on_screen_root(config: &PlayCommandConfig, screen_root: &String, wallpaper_id: &str) {
    let mut command = Command::new("linux-wallpaperengine");
    if let Some(scaling) = &config.scaling {
        command.arg("--scaling").arg(scaling);
    }
    if let Some(clamping) = &config.clamping {
        command.arg("--clamping").arg(clamping);
    }
    command.arg("--screen-root").arg(screen_root);

    // [todo]: add silent
    // if i == 0 {
    //     if config.silent.unwrap_or_else(|| false) {
    //         command.arg("--silent");
    //     }
    //     if let Some(volume) = config.volume {
    //         command.arg("--volume").arg(volume.to_string());
    //     }
    //     if config.noautomute.unwrap_or_else(|| false) {
    //         command.arg("--noautomute").arg("--no-audio-processing");
    //     }
    // } else {
    //     command.arg("--silent");
    // }
    command.arg("--silent");

    if config.window.unwrap_or_else(|| false) {
        command.arg("--window");
    }
    if let Some(fps) = config.fps {
        command.arg("--fps").arg(fps.to_string());
    }
    if let Some(assets_dir) = &config.assets_dir {
        command.arg("--assets-dir").arg(assets_dir);
    }
    if config.screenshot.unwrap_or_else(|| false) {
        command.arg("--screenshot");
    }
    if config.list_propertites.unwrap_or_else(|| false) {
        command.arg("--list-propertites");
    }
    // Can not set propertites for all wallpaper
    // if let Some(set_properties) = &config.set_property {
    //     for s in set_properties {
    //         command
    //             .arg("--set-property")
    //             .arg(format!("{}={}", key, value));
    //     }
    // }
    if config.no_fullscreen_pause.unwrap_or_else(|| false) {
        command.arg("--no-fullscreen-pause");
    }
    if config.disable_mouse.unwrap_or_else(|| false) {
        command.arg("--disable-mouse");
    }

    command.arg(wallpaper_id);

    let log_file = match &(config.log_file) {
        Some(_) => config.log_file.as_ref().unwrap(),
        None => "./linux_wallpaperengine.log",
    };

    info!("Execute command: {:?}", command);
    let process = command
        .stdout(Stdio::from(
            File::create(&log_file).expect("Can not cerea log file!"),
        ))
        .stderr(Stdio::from(
            File::create(&log_file).expect("Can not cerea log file!"),
        ))
        .spawn()
        .expect("Error to run command!");
    process.id();
}
