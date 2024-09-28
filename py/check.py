import os
import sys
from load_wallpaper import load_wallpaper 


if len(sys.argv) < 2:
    print("请提供ID文件路径")
    sys.exit(1)

id_file = sys.argv[1] + "_checked"
try:
    with open(id_file, 'r') as file:
        checked_ids = [line.strip().split('#')[0] for line in file if line.strip()]

    for id in checked_ids:
        if os.path.isdir(f"{wallpaper_dir}/{id}"):
            load_wallpaper(id)
            answer = input(f"是否保存当前壁纸ID [{id}] (y/n)? ")
            if answer.lower() == 'y':
                with open(id_file, 'a') as file:
                    file.write(f"{id}\n")
            elif answer.lower() == 'n':
                with open(id_file, 'a') as file:
                    file.write(f"{id}#skip\n")
        else:
            with open(id_file, 'a') as file:
                file.write(f"{id}#not_found\n")
except FileNotFoundError:
    print(f"文件 {id_file} 未找到")
