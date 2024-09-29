#!/usr/bin/env python3

import argparse
import json
import os
import shutil
from typing import Any, cast

from helper import GlobalConfig, read_config, setup_logging
from linux_wallpaperengine import kill_wallpaperengine, load_wallpaper

GLOBAL_CONFIG: GlobalConfig | None = None
CHECKED_SUFFIX = "_checked"
SKIPPED_SUFFIX = "_skipped"

type Item = dict[str, int]
type Folder = dict[str, Item | list[Folder] | str]
type Browser = dict[str, list[Folder]]
type General = dict[str, Browser]
type Steamuser = dict[str, General]
type WallpaperengineConfig = dict[str, str | Steamuser]


def get_reslut_folders(title: str, folders: list[Folder]) -> list[Folder]:
    result: list[Folder] = []
    for suffix in [CHECKED_SUFFIX, SKIPPED_SUFFIX]:
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
    return title.endswith("_checked") or title.endswith("_skipped")


def backup_wallpaperengine_config(config_path: str):
    backup_path: str = config_path.replace(".json", "_backup.json")
    shutil.copy(config_path, os.path.expanduser(backup_path))


def check(GLOBAL_CONFIG):
    if GLOBAL_CONFIG is None:
        print("Config is not setup!")
        return

    print("start check")

    wallpaperengine_config_path = (
        GLOBAL_CONFIG.wallpaperengine.wallpaperengine_config_file
    )
    backup_wallpaperengine_config(os.path.expanduser(wallpaperengine_config_path))

    wallpaperengine_config = {}

    with open(
        os.path.expanduser(wallpaperengine_config_path), "r"
    ) as wallpaperengine_config_file:
        wallpaperengine_config: WallpaperengineConfig = json.load(
            wallpaperengine_config_file
        )
    # get json data steamuser.general.browser.floders
    folders = (
        cast(Steamuser, wallpaperengine_config.get("steamuser", {}))
        .get("general", {})
        .get("browser", {})
        .get("folders", [])
    )
    # folders = [f for f in folders if not (is_generate_floder(f))]

    for folder in folders:
        if is_generate_floder(folder):
            continue

        title = folder.get("title")
        if title is None:
            print(
                f"Error when get folder title, please check your wallpaperengine config file.json data: {folder}"
            )

        while True:
            user_input = (
                input(f"是否check this folder: {title}? (y/n): ").strip().lower()
            )
            if user_input in ["y", "n"]:
                break
            print("无效输入，请输入 'y' 或 'n' 或者直接回车跳过")

        if user_input == "n":
            continue

        checked_folder, skipped_folder = get_reslut_folders(cast(str, title), folders)

        # 处理 items 中的每个 key
        items = cast(Item, folder.get("items", {}))
        based_items = items.copy()
        for wallpaper_id in based_items.keys():
            # 检查 ID 对应的目录是否存在
            wallpaper_dir = os.path.expanduser(
                GLOBAL_CONFIG.wallpaperengine.wallpaper_dir
            )
            item_dir = os.path.join(wallpaper_dir, wallpaper_id)
            if not os.path.exists(item_dir):
                continue  # 如果目录不存在，跳过此循环

            print(f"处理项: {wallpaper_id}")
            wallpaper_path: str = os.path.join(wallpaper_dir, wallpaper_id)
            if not os.path.isdir(wallpaper_path):
                print(f"wallpaper {wallpaper_id} not found.")
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
                print("无效输入，请输入 'y' 或 'n' 或者直接回车跳过")

            if user_input == "y":
                # 用户选择了 'y'，添加到新 folder 的 items 中
                cast(Item, checked_folder["items"])[wallpaper_id] = 1
                del cast(Item, folder["items"])[wallpaper_id]
            elif user_input == "n":
                # 用户选择了 'n'，在 id 末尾添加 #skipped
                cast(Item, skipped_folder["items"])[wallpaper_id] = 1
                del cast(Item, folder["items"])[wallpaper_id]
            else:
                # 用户未输入，直接跳过
                break
            with open(
                os.path.expanduser(wallpaperengine_config_path), "w"
            ) as wallpaperengine_config_file:
                # 更新配置文件内容并写入
                json.dump(
                    wallpaperengine_config, wallpaperengine_config_file, indent=4
                )  # 循环处理floders中的每一项

    kill_wallpaperengine()
    print("Checked all items")


def play():
    print("play")


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

    args = parser.parse_args()
    GLOBAL_CONFIG = read_config(args.config)
    setup_logging(GLOBAL_CONFIG.player.log_file)

    if args.check:
        check(GLOBAL_CONFIG)
    elif args.play:
        play()
    return


if __name__ == "__main__":
    main()
