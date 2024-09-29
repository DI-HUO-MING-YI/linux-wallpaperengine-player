#!/usr/bin/python3

import hashlib
import os


def get_monitors():
    monitors = []
    drm_path = "/sys/class/drm"

    for entry in os.listdir(drm_path):
        if "-" in entry:  # 只处理连接的显示器
            edid_path = os.path.join(drm_path, entry, "edid")
            if os.path.isfile(edid_path):
                monitors.append((entry, edid_path))

    return monitors


def calculate_uuid(edid_path):
    with open(edid_path, "rb") as f:
        edid_data = f.read()
        uuid = hashlib.md5(edid_data).hexdigest()  # 使用 MD5 作为 UUID
    return uuid


def main():
    monitors = get_monitors()

    for monitor, edid_path in monitors:
        uuid = calculate_uuid(edid_path)
        print(f"Display: {monitor}, UUID: {uuid}")


if __name__ == "__main__":
    main()
