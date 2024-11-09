mod check;
mod config;
mod control;
mod play;
mod sddm;
mod wallock;
mod wallpaperengine;
mod watch;

use check::check;
use clap::{Arg, ArgGroup, Command};
use config::app_config::AppConfig;
use control::control;
use fern::Dispatch;
use play::play;
use sddm::sddm;
use wallock::wallock;
use watch::watch;

pub fn run() {
    let matches = register_command();

    let config_path = matches.get_one::<String>("config");
    let mut app_config = AppConfig::get_app_config(config_path);

    let log_file = app_config.general.log_file.as_ref();
    setup_logging(&log_file);

    if let Some(_) = matches.subcommand_matches("check") {
        check(&app_config);
    } else if let Some(play_matches) = matches.subcommand_matches("play") {
        let playlist_name = play_matches.get_one::<String>("playlist").unwrap();
        play(&mut app_config, playlist_name);
    } else if let Some(watch_matches) = matches.subcommand_matches("watch") {
        let profile_name = watch_matches.get_one::<String>("profile").unwrap();
        watch(&app_config, profile_name);
    } else if let Some(sddm_matches) = matches.subcommand_matches("sddm") {
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
        } else {
            None
        };
        control(action);
    }
}

fn register_command() -> clap::ArgMatches {
    Command::new("A script that either checks or plays a live wallpaper.")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .required(false)
                .help("Config file path"),
        )
        .subcommand(Command::new("check").about("Execute check function"))
        .subcommand(
            Command::new("sddm").about("Execute check function").arg(
                Arg::new("folder")
                    .long("folder")
                    .value_parser(clap::value_parser!(String))
                    .required(true)
                    .help("Folder name from wallpaperengine"),
            ),
        )
        .subcommand(
            Command::new("play").about("Execute play function").arg(
                Arg::new("playlist")
                    .long("playlist")
                    .value_parser(clap::value_parser!(String))
                    .required(true)
                    .help("Playlist name from wallpaperengine"),
            ),
        )
        .subcommand(
            Command::new("watch").about("Execute watch function").arg(
                Arg::new("profile")
                    .long("profile")
                    .value_parser(clap::value_parser!(String))
                    .required(true)
                    .help("Profile name from wallpaperengine"),
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
                        .help("Reload wallpaper in the currnt playlist"),
                )
                .arg(
                    Arg::new("stop")
                        .long("stop")
                        .short('s')
                        .action(clap::ArgAction::SetTrue)
                        .help("Stop wallpaper in the currnt playlist"),
                )
                .arg(
                    Arg::new("continue")
                        .long("continue")
                        .short('c')
                        .action(clap::ArgAction::SetTrue)
                        .help("Continue wallpaper in the currnt playlist"),
                )
                .group(
                    ArgGroup::new("actions")
                        .args(["next", "prev", "reload", "stop", "continue"])
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
