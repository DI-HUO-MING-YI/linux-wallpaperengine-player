mod config;
mod control;
mod sddm;
mod wallock;
mod wallpaperengine;
mod waywallen;

use clap::{Arg, ArgGroup, Command};
use config::app_config::AppConfig;
use config::played_history::PlayedHistory;
use control::control;
use fern::Dispatch;
use sddm::sddm;
use wallock::wallock;

pub fn run() {
    let matches = register_command();

    let config_path = matches.get_one::<String>("config");
    let mut app_config = AppConfig::get_app_config(config_path);

    let log_file = app_config.general.log_file.as_ref();
    setup_logging(&log_file);

    if let Some(sddm_matches) = matches.subcommand_matches("sddm") {
        let folder_name = sddm_matches.get_one::<String>("folder").unwrap();
        sddm(&mut app_config, folder_name);
    } else if let Some(wallock_matches) = matches.subcommand_matches("wallock") {
        let folder_name = wallock_matches.get_one::<String>("folder").unwrap();
        wallock(&mut app_config, folder_name);
    } else if let Some(congtrol_matches) = matches.subcommand_matches("control") {
        let action = if congtrol_matches.get_flag("next") {
            Some("Next")
        } else if congtrol_matches.get_flag("prev") {
            Some("Prev")
        } else if congtrol_matches.get_flag("reload") {
            Some("Reload")
        } else if congtrol_matches.get_flag("stop") {
            Some("Stop")
        } else if congtrol_matches.get_flag("continue") {
            Some("Continue")
        } else if congtrol_matches.get_flag("pause") {
            Some("Pause")
        } else if congtrol_matches.get_flag("open-ui") {
            Some("OpenUi")
        } else if congtrol_matches.get_flag("mute") {
            Some("Mute")
        } else if congtrol_matches.get_flag("shuffle") {
            Some("Shuffle")
        } else {
            None
        };
        if let Err(error) = control(action) {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
    } else if let Some(export_matches) = matches.subcommand_matches("export") {
        let file_path = export_matches.get_one::<String>("file").unwrap();
        let history = PlayedHistory::new(&app_config.general.played_history_db.clone()).unwrap();
        if let Err(e) = history.export_stats_to_csv(file_path) {
            log::error!("导出播放历史失败: {}", e);
        } else {
            log::info!("成功导出播放历史到: {}", file_path);
        }
    }
}

fn register_command() -> clap::ArgMatches {
    Command::new("A script that plays and manages live wallpapers.")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .required(false)
                .help("Config file path"),
        )
        .subcommand(
            Command::new("sddm").about("Set the SDDM wallpaper").arg(
                Arg::new("folder")
                    .long("folder")
                    .value_parser(clap::value_parser!(String))
                    .required(true)
                    .help("Folder name from wallpaperengine"),
            ),
        )
        .subcommand(
            Command::new("control")
                .about("Execute control function")
                .arg(
                    Arg::new("next")
                        .long("next")
                        .short('n')
                        .action(clap::ArgAction::SetTrue)
                        .help("Play next wallpaper in the currnt playlist"),
                )
                .arg(
                    Arg::new("prev")
                        .long("prev")
                        .short('p')
                        .action(clap::ArgAction::SetTrue)
                        .help("Play prev wallpaper in the currnt playlist"),
                )
                .arg(
                    Arg::new("reload")
                        .long("reload")
                        .short('r')
                        .action(clap::ArgAction::SetTrue)
                        .help("Restart Waywallen"),
                )
                .arg(
                    Arg::new("stop")
                        .long("stop")
                        .short('s')
                        .action(clap::ArgAction::SetTrue)
                        .help("Pause global queue rotation"),
                )
                .arg(
                    Arg::new("continue")
                        .long("continue")
                        .short('c')
                        .action(clap::ArgAction::SetTrue)
                        .help("Resume global queue rotation and advance to the next wallpaper"),
                )
                .arg(
                    Arg::new("pause")
                        .long("pause")
                        .action(clap::ArgAction::SetTrue)
                        .help("Toggle pause for all Waywallen renderers"),
                )
                .arg(
                    Arg::new("open-ui")
                        .long("open-ui")
                        .action(clap::ArgAction::SetTrue)
                        .help("Open the Waywallen UI"),
                )
                .arg(
                    Arg::new("mute")
                        .long("mute")
                        .action(clap::ArgAction::SetTrue)
                        .help("Toggle mute for all Waywallen renderers"),
                )
                .arg(
                    Arg::new("shuffle")
                        .long("shuffle")
                        .action(clap::ArgAction::SetTrue)
                        .help("Toggle the global queue between shuffle and sequential modes"),
                )
                .group(
                    ArgGroup::new("actions")
                        .args([
                            "next", "prev", "reload", "stop", "continue", "pause", "open-ui",
                            "mute", "shuffle",
                        ])
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("wallock")
                .about("Execute wallock function")
                .arg(
                    Arg::new("folder")
                        .long("folder")
                        .value_parser(clap::value_parser!(String))
                        .required(true)
                        .help("Folder name from wallpaperengine"),
                ),
        )
        .subcommand(
            Command::new("export")
                .about("导出播放历史统计到CSV文件")
                .arg(
                    Arg::new("file")
                        .long("file")
                        .value_parser(clap::value_parser!(String))
                        .required(true)
                        .help("导出CSV文件的路径"),
                ),
        )
        .subcommand_required(true)
        .get_matches()
}
fn setup_logging(log_file: &Option<&String>) {
    let log_file = log_file;
    let default_log_file = String::from("debug.log");
    let log_file = log_file.unwrap_or(&default_log_file);
    Dispatch::new()
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file(log_file).unwrap())
        .apply()
        .unwrap();
}
