import os
import random
import time

def load_wallpaper(id):
    # 这里应该是调用壁纸加载的逻辑
    pass

def save_current_wallpaper_id(current_wallpaper_id, config_file):
    with open(config_file, 'w') as file:
        file.write(f"current_wallpaper_id={current_wallpaper_id}\n")

ids = ["123", "456", "789"]  # 示例 ID 列表
mode = 1  # 示例模式

while True:
    if mode == 1:
        for id in ids:
            load_wallpaper(id)
            save_current_wallpaper_id(id, config_file)
    elif mode == 2:
        id = random.choice(ids)
        load_wallpaper(id)
        save_current_wallpaper_id(id, config_file)
    elif mode == 3:
        random.shuffle(ids)
        for id in ids:
            load_wallpaper(id)
            save_current_wallpaper_id(id, config_file)
    else:
        print("无效模式。请选择 1 (顺序播放)，2 (随机)，或 3 (随机不重复)。")
        sys.exit(1)