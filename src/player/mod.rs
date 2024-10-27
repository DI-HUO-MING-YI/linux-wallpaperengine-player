mod check;
mod config;
mod play;
mod sddm;
mod wallpaperengine;
mod watch;

use check::check;
use clap::{Arg, Command};
use config::app_config::AppConfig;
use fern::Dispatch;
use play::play;
use sddm::sddm;
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
