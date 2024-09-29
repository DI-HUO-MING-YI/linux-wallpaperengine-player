import logging
import os
import subprocess
import time

from helper import LinuxWallpaperengineConfig

current_process: list[int] = []


def kill_wallpaperengine_process():
    try:
        for pid in current_process:
            result = subprocess.run(
                [
                    'ps aux | grep "linux-wallpaperengine" | grep -v "grep" | grep -v "player.py" | awk \'{print $2}\' | xargs kill -9'
                ],
                shell=True,
            )
            if result.returncode != 0:
                logging.info("Can not kill the linux-wallpaperengine process")
            else:
                logging.info("Killed all linux-wallpaperengine process")

    except subprocess.CalledProcessError:
        logging.info("Can not kill the linux-wallpaperengine process")


def load_wallpaper(
    id: str,
    wallpaper_dir: str,
    log_path: str | None,
    config: LinuxWallpaperengineConfig,
):
    logging.info("start load_wallpaper")
    wallpaper_path: str = os.path.join(wallpaper_dir, id)
    if os.path.isdir(wallpaper_path):
        kill_wallpaperengine_process()
        logging.info(f"加载壁纸: {id}")

        base_command = ["linux-wallpaperengine"]
        screen_roots = config.screen_root

        if screen_roots is None:
            logging.warning("No screen root config found. Exit.")
            return

        for i, screen_root in enumerate(screen_roots):
            command = base_command.copy()
            if config.scaling:
                command += ["--scaling", config.scaling]
            if config.clamping:
                command += ["--clamping", config.clamping]
            command += ["--screen-root", screen_root]

            if i == 0:
                if config.silent:
                    command.append("--silent")
                if config.volume is not None:
                    command += ["--volume", str(config.volume)]
                if config.noautomute:
                    command.append("--noautomute")
                if config.noautomute:
                    command.append("--no-audio-processing")
            else:
                command.append("--silent")

            if config.window:
                command.append("--window")
            if config.fps:
                command += ["--fps", str(config.fps)]
            if config.assets_dir:
                command += ["--assets-dir", config.assets_dir]
            if config.screenshot:
                command.append("--screenshot")
            if config.list_properties:
                command.append("--list-propertites")
            if config.set_property:
                set_properties = config.set_property
                for key, value in set_properties.items():
                    command += ["--set-property", f"{key}={value}"]
            if config.no_fullscreen_pause:
                command.append("--no-fullscreen-pause")
            if config.disable_mouse:
                command.append("--disable-mouse")

            command.append(id)

            logging.info(f"Execute command: {' '.join(command)}")
            log_output = f"{log_path}_in_{screen_root}"
            with open(os.path.expanduser(log_output), "a") as log_file:
                process = subprocess.Popen(command, stdout=log_file, stderr=log_file)
                current_process.append(process.pid)
            time.sleep(0.5)
    else:
        logging.error(f"Wallpaper {id} not found")
