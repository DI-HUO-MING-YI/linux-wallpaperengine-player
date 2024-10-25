use std::fs::{self, File};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use log::{error, info};
use nix::libc::{kill, killpg, setpgid, SIGKILL, SIGTERM};

use super::config::app_config::PlayCommandConfig;

pub fn load_wallpaper(
    wallpaper_path: &Path,
    wallpaper_id: &str,
    config: &PlayCommandConfig,
) -> Vec<Child> {
    info!("start load_wallpaper");
    if fs::metadata(wallpaper_path).is_ok() {
        info!("loading wallpaper: {}", wallpaper_path.to_str().unwrap());

        let mut pids = vec![];
        for screen_root in &config.screen_root {
            let screen_root: &String = &screen_root;
            let mut command = build_command(config, screen_root, wallpaper_id);

            let log_file = match &(config.log_file) {
                Some(_) => config.log_file.as_ref().unwrap(),
                None => "./linux_wallpaperengine.log",
            };

            info!("Execute command: {:?}", command);
            let process = unsafe {
                command
                    .stdout(Stdio::from(
                        File::create(&log_file).expect("Can not cerea log file!"),
                    ))
                    .stderr(Stdio::from(
                        File::create(&log_file).expect("Can not cerea log file!"),
                    ))
                    .pre_exec(|| {
                        setpgid(0, 0);
                        Ok(())
                    })
                    .spawn()
                    .expect("Error to run command!")
            };
            pids.push(process);
            sleep(Duration::from_millis(500));
        }
        pids
    } else {
        error!("Wallpaper {} not found", wallpaper_path.to_str().unwrap());
        vec![]
    }
}

fn build_command(config: &PlayCommandConfig, screen_root: &String, wallpaper_id: &str) -> Command {
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
    command
}
