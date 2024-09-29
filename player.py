#!/usr/bin/env python3

import argparse
import json
import logging
import os
import random
import shutil
import time
from typing import Any, cast

from helper import GlobalConfig, read_config
from linux_wallpaperengine import kill_wallpaperengine_process, load_wallpaper

logging.basicConfig(
    level=logging.DEBUG,  # 确保设置的日志级别能够打印INFO消息
    format="%(asctime)s - %(levelname)s - %(message)s",
)

GLOBAL_CONFIG: GlobalConfig | None = None
PICKED_SUFFIX = "_picked"
SKIPPED_SUFFIX = "_skipped"

type Item = dict[str, int]
type Folder = dict[str, Item | list[Folder] | str]
type Settings = dict[str, int | str | bool]
type PlayList = dict[str, str | Settings | list[str]]
type PlayLists = list[PlayList]
type Browser = dict[str, list[Folder]]
type General = dict[str, Browser | PlayLists]
type Steamuser = dict[str, General]
type WallpaperengineConfig = dict[str, str | Steamuser]


def get_reslut_folders(title: str, folders: list[Folder]) -> list[Folder]:
    result: list[Folder] = []
    for suffix in [PICKED_SUFFIX, SKIPPED_SUFFIX]:
        exsit_folder = next(
            (f for f in folders if f.get("title") == title + suffix),
            None,
        )

        if exsit_folder is None:
            new_folder: Folder = {
                "items": {},
                "subfolders": [],
                "title": title + suffix,
                "type": "folder",
            }
            folders.append(new_folder)
            exsit_folder = new_folder
        result.append(exsit_folder)
    return result


def is_generate_floder(floder: dict[str, Any]) -> bool:
    title: str = floder.get("title", "")
    return title.endswith(PICKED_SUFFIX) or title.endswith(SKIPPED_SUFFIX)


def backup_wallpaperengine_config(config_path: str):
    backup_path: str = config_path.replace(".json", "_backup.json")
    shutil.copy(config_path, os.path.expanduser(backup_path))


def get_wallpaperengine_config(
    wallpaperengine_config_path: str,
) -> WallpaperengineConfig:
    with open(
        os.path.expanduser(wallpaperengine_config_path), "r"
    ) as wallpaperengine_config_file:
        wallpaperengine_config: WallpaperengineConfig = json.load(
            wallpaperengine_config_file
        )
        return wallpaperengine_config


def get_floders(wallpaperengine_config: WallpaperengineConfig) -> list[Folder]:
    general = cast(Steamuser, wallpaperengine_config.get("steamuser", {})).get(
        "general", {}
    )
    folders = cast(Browser, general.get("browser", {})).get("folders", [])
    return folders


def get_play_lists(wallpaperengine_config: WallpaperengineConfig) -> PlayLists:
    general = cast(Steamuser, wallpaperengine_config.get("steamuser", {})).get(
        "general", {}
    )
    return cast(PlayLists, general.get("playlists", {}))


def should_picked(
    GLOBAL_CONFIG: GlobalConfig, type: str | None, wallpaper_id: str
) -> bool:
    logging.info(
        f"Test if should picked: {GLOBAL_CONFIG.player.picked_types}, {type}, {wallpaper_id}"
    )
    if type is None:
        return False
    picked_types = GLOBAL_CONFIG.player.picked_types
    if picked_types is not None and picked_types.__contains__(type.lower()):
        logging.info(f"Picked wallpaper {wallpaper_id} by picked_types config item.")
        return True
    return False


def should_skip(
    GLOBAL_CONFIG: GlobalConfig, type: str | None, wallpaper_id: str
) -> bool:
    if type is None:
        return False
    skipped_types = GLOBAL_CONFIG.player.skipped_types
    if skipped_types is not None and skipped_types.__contains__(type.lower()):
        logging.info(f"Skipped wallpaper {wallpaper_id} by skipped_types config item.")
        return True
    return False


def check_items(
    GLOBAL_CONFIG: GlobalConfig,
    picked_folder: Folder,
    skipped_folder: Folder,
    folder: Folder,
    item: Item,
    wallpaperengine_config: WallpaperengineConfig,
    wallpaperengine_config_path: str,
):
    wallpaper_ids = item.copy().items()
    for wallpaper_id, _ in wallpaper_ids:
        logging.info(f"处理项: {wallpaper_id}")
        # 检查 ID 对应的目录是否存在
        wallpaper_dir = os.path.expanduser(GLOBAL_CONFIG.wallpaperengine.wallpaper_dir)
        item_dir = os.path.join(wallpaper_dir, wallpaper_id)
        item_project_path = os.path.join(item_dir, "project.json")
        if not os.path.exists(item_dir) and not os.path.exists(item_project_path):
            logging.info(f"wallpaper {wallpaper_id} not found.")
            continue

        with open(item_project_path, "r") as item_project_file:
            project_json: dict[str, str] = json.load(item_project_file)
            type = project_json.get("type")

        if should_skip(GLOBAL_CONFIG, type, wallpaper_id):
            continue

        if should_picked(GLOBAL_CONFIG, type, wallpaper_id):
            move_to_folder(
                folder,
                picked_folder,
                wallpaper_id,
                wallpaperengine_config_path,
                wallpaperengine_config,
            )
            continue

        load_wallpaper(
            wallpaper_id,
            wallpaper_dir,
            GLOBAL_CONFIG.player.wallpaperengine_log_file,
            GLOBAL_CONFIG.linux_wallpaperengine,
        )

        while True:
            user_input = input("是否添加到新项? (y/n): ").strip().lower()
            if user_input in ["y", "n", ""]:
                break
            logging.info("无效输入，请输入 'y' 或 'n' 或者直接回车跳过")

        if user_input == "y":
            move_to_folder(
                folder,
                picked_folder,
                wallpaper_id,
                wallpaperengine_config_path,
                wallpaperengine_config,
            )
        elif user_input == "n":
            move_to_folder(
                folder,
                skipped_folder,
                wallpaper_id,
                wallpaperengine_config_path,
                wallpaperengine_config,
            )
        else:
            break


def move_to_folder(
    from_folder: Folder,
    to_folder: Folder,
    wallpaper_id: str,
    wallpaperengine_config_path: str,
    wallpaperengine_config: WallpaperengineConfig,
):
    cast(Item, to_folder["items"])[wallpaper_id] = 1
    del cast(Item, from_folder["items"])[wallpaper_id]
    with open(
        os.path.expanduser(wallpaperengine_config_path), "w"
    ) as wallpaperengine_config_file:
        json.dump(wallpaperengine_config, wallpaperengine_config_file, indent=4)


def check_folders(
    GLOBAL_CONFIG: GlobalConfig,
    folders: list[Folder],
    wallpaperengine_config: WallpaperengineConfig,
    wallpaperengine_config_path: str,
):
    for folder in folders:
        if is_generate_floder(folder):
            continue

        title = folder.get("title")
        if title is None:
            logging.info(
                f"Error when get folder title, please check your wallpaperengine config file.json data: {folder}"
            )

        while True:
            user_input = (
                input(f"是否check this folder: {title}? (y/n): ").strip().lower()
            )
            if user_input in ["y", "n"]:
                break
            logging.info("无效输入，请输入 'y' 或 'n' 或者直接回车跳过")

        if user_input == "n":
            continue

        picked_folder, skipped_folder = get_reslut_folders(cast(str, title), folders)

        # 处理 items 中的每个 key
        item = cast(Item, folder.get("items", {}))
        # based_items = items.copy()
        check_items(
            GLOBAL_CONFIG,
            picked_folder,
            skipped_folder,
            folder,
            item,
            wallpaperengine_config,
            wallpaperengine_config_path,
        )


def check(GLOBAL_CONFIG: GlobalConfig):
    logging.info("start check")

    wallpaperengine_config_path = (
        GLOBAL_CONFIG.wallpaperengine.wallpaperengine_config_file
    )

    backup_wallpaperengine_config(os.path.expanduser(wallpaperengine_config_path))

    wallpaperengine_config = get_wallpaperengine_config(wallpaperengine_config_path)

    folders = get_floders(wallpaperengine_config)

    check_folders(
        GLOBAL_CONFIG, folders, wallpaperengine_config, wallpaperengine_config_path
    )

    kill_wallpaperengine_process()

    logging.info("Checked all items")


def get_wallpaper_id(item: str) -> str:
    return os.path.basename(os.path.dirname(item))


def play_by_timer(
    GLOBAL_CONFIG: GlobalConfig, items: list[str], settings: Settings
) -> None:
    delay = cast(int, settings.get("delay"))
    order = cast(str, settings.get("order"))
    wallpaper_ids = list(map(get_wallpaper_id, items))
    if order == "random":
        random.shuffle(wallpaper_ids)

    for wallpaper_id in wallpaper_ids:
        wallpaper_dir = os.path.expanduser(GLOBAL_CONFIG.wallpaperengine.wallpaper_dir)
        item_dir = os.path.join(wallpaper_dir, wallpaper_id)
        item_project_path = os.path.join(item_dir, "project.json")
        if not os.path.exists(item_dir) and not os.path.exists(item_project_path):
            logging.info(f"wallpaper {wallpaper_id} not found.")
            continue

        load_wallpaper(
            wallpaper_id,
            wallpaper_dir,
            GLOBAL_CONFIG.player.wallpaperengine_log_file,
            GLOBAL_CONFIG.linux_wallpaperengine,
        )

        time.sleep(delay * 60)


def play(GLOBAL_CONFIG: GlobalConfig, play_list_name: str):
    logging.info("play wallpaper list new")
    wallpaperengine_config_path = (
        GLOBAL_CONFIG.wallpaperengine.wallpaperengine_config_file
    )

    wallpaperengine_config = get_wallpaperengine_config(wallpaperengine_config_path)
    play_lists = get_play_lists(wallpaperengine_config)
    play_list = next(
        (p for p in play_lists if cast(str, p.get("name")) == play_list_name), None
    )
    if play_list is None:
        logging.error(f"No such playlist named {play_list_name}")
        exit(1)

    items = cast(list[str], play_list.get("items"))
    settings = cast(Settings, play_list.get("settings"))
    mode = settings.get("mode")
    if mode == "timer":
        play_by_timer(GLOBAL_CONFIG, items, settings)


def main():
    parser = argparse.ArgumentParser(
        description="A script that either checks or plays."
    )

    group = parser.add_mutually_exclusive_group(required=True)
    _ = group.add_argument(
        "--check", action="store_true", help="Execute check function"
    )
    _ = group.add_argument("--play", action="store_true", help="Execute play function")
    _ = parser.add_argument("--config", type=str, help="Config file path")
    _ = parser.add_argument("--playlist", type=str, help="Config file path")

    args = parser.parse_args()
    GLOBAL_CONFIG = read_config(args.config)

    if args.check:
        check(GLOBAL_CONFIG)
    elif args.play:
        play(GLOBAL_CONFIG, args.playlist)
    return


if __name__ == "__main__":
    main()
