import configparser
import logging
import os
import sys
from dataclasses import dataclass
from enum import Enum


class Mode(Enum):
    ORDER = 1
    RANDOM = 2
    RANDOM_IN_ORDER = 3


@dataclass
class PlayerConfig:
    mode: Mode | None
    current_wallpaper_id: str | None
    interval: int | None
    log_file: str | None
    wallpaperengine_log_file: str | None
    picked_types: list[str] | None
    skipped_types: list[str] | None


@dataclass
class WallpaperengineConfig:
    wallpaper_dir: str
    wallpaperengine_config_file: str


@dataclass
class LinuxWallpaperengineConfig:
    silent: bool | None
    volume: int | None
    noautomute: bool | None
    no_audio_processing: bool | None
    screen_root: list[str] | None
    window: bool | None
    fps: int | None
    assets_dir: str | None
    screenshot: bool | None
    list_properties: bool | None
    set_property: dict[str, str]
    no_fullscreen_pause: bool | None
    disable_mouse: bool | None
    scaling: str | None
    clamping: str | None


@dataclass
class GlobalConfig:
    player: PlayerConfig
    wallpaperengine: WallpaperengineConfig
    linux_wallpaperengine: LinuxWallpaperengineConfig


def read_config(config_path: str | None) -> GlobalConfig:
    script_path = os.path.dirname(__file__)
    default_config_file = os.path.expanduser(f"{script_path}/config.ini")
    custom_config_file = (
        os.path.expanduser(config_path)
        if config_path is not None
        else default_config_file
    )

    if not os.path.isfile(custom_config_file):
        logging.info(f"config file not found: {custom_config_file}")
        sys.exit(1)

    logging.info(f"using custom config file: {custom_config_file}")

    config = configparser.ConfigParser()
    _ = config.read(default_config_file)
    _ = config.read(custom_config_file)
    player_config = PlayerConfig(
        mode=(
            Mode[config["player"]["mode"]]
            if config["player"]["mode"] in Mode.__members__
            else None
        ),
        current_wallpaper_id=config.get(
            "player", "current_wallpaper_id", fallback=None
        ),
        interval=config.getint("player", "interval", fallback=None),
        log_file=config.get("player", "log_file", fallback=None),
        wallpaperengine_log_file=config.get(
            "player", "wallpaperengine_log_file", fallback=None
        ),
        picked_types=(
            config.get("player", "picked_types").split(",")
            if "picked_types" in config["player"]
            else None
        ),
        skipped_types=(
            config.get("player", "skipped_types").split(",")
            if "skipped_types" in config["player"]
            else None
        ),
    )

    wallpaperengine_config = WallpaperengineConfig(
        wallpaper_dir=config.get("wallpaperengine", "wallpaper_dir"),
        wallpaperengine_config_file=config.get(
            "wallpaperengine", "wallpaperengine_config_file"
        ),
    )

    linux_wallpaperengine_config = LinuxWallpaperengineConfig(
        silent=config.getboolean("linux_wallpaperengine", "silent", fallback=None),
        volume=config.getint("linux_wallpaperengine", "volume", fallback=None),
        noautomute=config.getboolean(
            "linux_wallpaperengine", "noautomute", fallback=None
        ),
        no_audio_processing=config.getboolean(
            "linux_wallpaperengine", "no_audio_processing", fallback=None
        ),
        screen_root=(
            config.get("linux_wallpaperengine", "screen_root").split(",")
            if "screen_root" in config["linux_wallpaperengine"]
            else None
        ),
        window=config.getboolean("linux_wallpaperengine", "window", fallback=None),
        fps=config.getint("linux_wallpaperengine", "fps", fallback=None),
        assets_dir=config.get("linux_wallpaperengine", "assets_dir", fallback=None),
        screenshot=config.getboolean(
            "linux_wallpaperengine", "screenshot", fallback=None
        ),
        list_properties=config.getboolean(
            "linux_wallpaperengine", "list_properties", fallback=None
        ),
        set_property=(
            dict(
                item.split("=")
                for item in config.get(
                    "linux_wallpaperengine", "set_property", fallback=""
                ).split(",")
            )
            if "set_property" in config["linux_wallpaperengine"]
            else {}
        ),
        no_fullscreen_pause=config.getboolean(
            "linux_wallpaperengine", "no_fullscreen_pause", fallback=None
        ),
        disable_mouse=config.getboolean(
            "linux_wallpaperengine", "disable_mouse", fallback=None
        ),
        scaling=config.get("linux_wallpaperengine", "scaling", fallback=None),
        clamping=config.get("linux_wallpaperengine", "clamping", fallback=None),
    )

    return GlobalConfig(
        player=player_config,
        wallpaperengine=wallpaperengine_config,
        linux_wallpaperengine=linux_wallpaperengine_config,
    )
