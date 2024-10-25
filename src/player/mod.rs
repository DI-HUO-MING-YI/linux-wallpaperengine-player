mod check;
mod config;
mod linux_wallpaperengine;
mod play;
mod watch;

use clap::{Arg, Command};
use config::app_config::AppConfig;
use fern::Dispatch;
use play::play;

pub fn run() {
    let matches = register_command();
    let config_path = matches.get_one::<String>("config");
    let app_config = AppConfig::get_app_config(config_path);
    let log_file = app_config.general.log_file.as_ref();
    let default_log_file = String::from("debug.log");

    setup_logging(log_file.unwrap_or(&default_log_file));

    if let Some(_) = matches.subcommand_matches("check") {
        // check();
    } else if let Some(play_matches) = matches.subcommand_matches("play") {
        let playlist_name = play_matches.get_one::<String>("playlist").unwrap();
        play(playlist_name, &app_config);
    } else if let Some(watch_matches) = matches.subcommand_matches("watch") {
        let profile = watch_matches.get_one::<String>("profile").unwrap();
        // watch(profile);
    }
}

fn register_command() -> clap::ArgMatches {
    Command::new("A script that either checks or plays a live wallpaper.")
        .arg(
            Arg::new("config")
                .long("config")
                .required(false)
                .help("Config file path"),
        )
        .subcommand(Command::new("check").about("Execute check function"))
        .subcommand(
            Command::new("play").about("Execute play function").arg(
                Arg::new("playlist")
                    .long("playlist")
                    .value_parser(clap::value_parser!(String))
                    .required(true)
                    .help("Playlist file path"),
            ),
        )
        .subcommand(
            Command::new("watch").about("Execute watch function").arg(
                Arg::new("profile")
                    .long("profile")
                    .value_parser(clap::value_parser!(String))
                    .required(true)
                    .help("Profile file path"),
            ),
        )
        .subcommand_required(true)
        .get_matches()
}
fn setup_logging(log_file: &str) {
    Dispatch::new()
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file(log_file).unwrap())
        .apply()
        .unwrap();
}