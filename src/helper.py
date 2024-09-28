import configparser
import logging
import os
import sys


def setup_logging(log_file: str):
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s %(levelname)s: %(message)s",
        handlers=[
            logging.FileHandler(log_file),
            logging.StreamHandler(sys.stdout),
        ],
    )


def read_config():
    default_config_file = os.path.expanduser("~/.config/wallpaperengine/config")
    custom_config_file = sys.argv[1] if len(sys.argv) > 1 else default_config_file

    if not os.path.isfile(custom_config_file):
        print(f"config file not found: {custom_config_file}")
        sys.exit(1)

    print(f"using custom config file: {custom_config_file}")

    config = configparser.ConfigParser()
    _ = config.read(default_config_file)
    _ = config.read(custom_config_file)
    for section in config.sections():
        print(f"[{section}]")
        for key, value in config.items(section):
            print(f"{key} = {value}")
        print()

    return config
