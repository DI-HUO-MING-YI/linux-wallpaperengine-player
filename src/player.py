#!/usr/bin/env python3

import argparse
import configparser
import json
import os
import shutil
from typing import Any

from py.helper import read_config, setup_logging
from py.linux_wallpaperengine import load_wallpaper

# from .linux_wallpaperengine import *

GLOBAL_CONFIG: configparser.ConfigParser
CHECKED_SUFFIX = "_checked"
SKIPPED_SUFFIX = "_skipped"


def get_reslut_folders(title: str, folders: list[dict[str, Any]]):
    result = []
    for suffix in [CHECKED_SUFFIX, SKIPPED_SUFFIX]:
        new_folder = next(
            (f for f in folders if f.get("title") == title + suffix),
            None,
        )

        if new_folder is None:
            new_folder = {
                "items": {},
                "subfolders": [],
                "title": title + suffix,
                "type": "folder",
            }
            folders.append(new_folder)
        result.append(new_folder)
    return new_folder


def is_generate_floder(floder: dict[str, Any]) -> bool:
    title: str = floder.get("title", "")
    return title.endswith("_checked") or title.endswith("_skipped")


def check():
    wallpaperengine_config_path = GLOBAL_CONFIG["wallpaperengine"][
        "wallpaperengine_config_file"
    ]
    backup_path: str = wallpaperengine_config_path.replace(".json", "_backup.json")
    shutil.copy(wallpaperengine_config_path, backup_path)

    # 读取JSON数据
    with open(wallpaperengine_config_path, "r") as wallpaperengine_config_file:
        data = json.load(wallpaperengine_config_file)
        # 获取 steamuser.general.browser.floders
        steam_user: dict[str, Any] = data.get("steamuser", {})
        general: dict[str, Any] = steam_user.get("general", {})
        browser: dict[str, Any] = general.get("broswer", {})
        floders: list[dict[str, Any]] = browser.get("floders", [])

        floders = [f for f in floders if not (is_generate_floder(f))]

        for folder in floders:
            checked_folder, skipped_folder = get_reslut_folders(
                folder.get("title", ""), floders
            )

            # 处理 items 中的每个 key
            for item in folder.get("items", {}).items():
                # 检查 ID 对应的目录是否存在
                wallpaper_dir = os.path.expanduser(
                    GLOBAL_CONFIG["wallpaperengine"]["wallpaper_dir"]
                )
                item_dir = os.path.join(wallpaper_dir, item_id)
                if not os.path.exists(item_dir):
                    continue  # 如果目录不存在，跳过此循环

                print(f"处理项: {item['key']}")
                load_wallpaper(item["key"], GLOBAL_CONFIG)

                while True:
                    user_input = input("是否添加到新项? (y/n): ").strip().lower()
                    if user_input in ["y", "n", ""]:
                        print("无效输入，请输入 'y' 或 'n' 或者直接回车跳过")
                        break

                with open(
                    wallpaperengine_config_path, "w"
                ) as wallpaperengine_config_file:
                    if user_input == "y":
                        # 用户选择了 'y'，添加到新 folder 的 items 中
                        checked_folder["items"][item_id] = item
                        del folder["items"][item_id]
                    elif user_input == "n":
                        # 用户选择了 'n'，在 id 末尾添加 #skipped
                        skipped_folder["items"][item_id] = item
                        del folder["items"][item_id]
                    else:
                        # 用户未输入，直接跳过
                        break
                    # 更新配置文件内容并写入
                    json.dump(
                        data, wallpaperengine_config_file, indent=4
                    )  # 循环处理floders中的每一项

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

    args = parser.parse_args()
    global GLOBAL_CONFIG
    GLOBAL_CONFIG = read_config()
    setup_logging(GLOBAL_CONFIG["player"]["log_file"])

    if args.check:
        check()
    elif args.play:
        play()


if __name__ == "__main__":
    main()
