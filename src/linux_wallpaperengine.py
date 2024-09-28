import configparser
import logging
import os
import subprocess


def kill_wallpaperengine():
    try:
        _ = subprocess.run(["pkill", "-f", "linux-wallpaperengine"], check=True)
        logging.info("已终止linux-wallpaperengine进程")
    except subprocess.CalledProcessError:
        logging.warning("未找到运行中的linux-wallpaperengine进程")


def load_wallpaper(id: str, config: configparser.ConfigParser):
    # wallpaper_path: str =
    os.path.join(config["wallpaper_dir"], id)
    if os.path.isdir(wallpaper_path):
        kill_wallpaperengine()
        time.sleep(1)
        logging.info(f"加载壁纸: {id}")

        base_command = ["linux-wallpaperengine"]
        screen_roots = config.get("screen_root", "").split(",")

        for i, screen_root in enumerate(screen_roots):
            command = base_command.copy()
            command += ["--screen-root", screen_root]

            if i == 0:
                if config.getboolean("silent", fallback=False):
                    command.append("--silent")
                if config.get("volume"):
                    command += ["--volume", config["volume"]]
                if config.getboolean("noautomute", fallback=False):
                    command.append("--noautomute")
                if config.getboolean("no_audio_processing", fallback=False):
                    command.append("--no-audio-processing")
            else:
                command.append("--silent")

            if config.get("scaling"):
                command += ["--scaling", config["scaling"]]
            if config.getboolean("window", fallback=False):
                command.append("--window")
            if config.get("fps"):
                command += ["--fps", config["fps"]]
            if config.get("assets_dir"):
                command += ["--assets-dir", config["assets_dir"]]
            if config.getboolean("screenshot", fallback=False):
                command.append("--screenshot")
            if config.getboolean("list_propertites", fallback=False):
                command.append("--list-propertites")
            if config.get("set_property"):
                set_properties = config.get("set_property").split(",")
                for sp in set_properties:
                    command += ["--set-property", sp]
            if config.getboolean("no_fullscreen_pause", fallback=False):
                command.append("--no-fullscreen-pause")
            if config.getboolean("disable_mouse", fallback=False):
                command.append("--disable-mouse")
            if config.get("clamping"):
                command += ["--clamping", config["clamping"]]

            command.append(id)

            logging.info(f"执行命令: {' '.join(command)}")
            log_output = f"{config['wallpaperengine_log_file']}_in_{screen_root}"
            with open(log_output, "a") as log_file:
                subprocess.Popen(command, stdout=log_file, stderr=log_file)
            time.sleep(1)

        time.sleep(int(config["interval"]))
    else:
        logging.error(f"未找到文件夹: {id}")
