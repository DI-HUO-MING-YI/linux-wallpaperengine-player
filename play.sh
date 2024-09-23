#!/bin/bash
# 检查文件夹和加载壁纸的函数
source ~/.config/wallpaperengine/load_wallpaper.sh

# 保存当前ID到配置文件
save_current_wallpaper_id() {
    sed -i "s/^current_wallpaper_id=.*/current_wallpaper_id=$current_wallpaper_id/" "$config_file"
}

# 开始循环
while true; do
    if [ "$mode" -eq 1 ]; then
        # 顺序播放
        found_current=false
        for id in "${ids[@]}"; do
            if [ "$found_current" = false ]; then
                if [ "$id" = "$current_wallpaper_id" ]; then
                    found_current=true
                    load_wallpaper "$id"
                fi
                continue
            fi
            current_wallpaper_id="$id"
            load_wallpaper "$id"
            save_current_wallpaper_id
        done
        # 播放到结尾后从头开始
        current_wallpaper_id="${ids[0]}"
        save_current_wallpaper_id
    elif [ "$mode" -eq 2 ]; then
        # 随机播放
        id="${ids[RANDOM % ${#ids[@]}]}"
        current_wallpaper_id="$id"
        load_wallpaper "$id"
        save_current_wallpaper_id
    elif [ "$mode" -eq 3 ]; then
        # 随机不重复播放
        shuffled_ids=($(shuf -e "${ids[@]}"))
        for id in "${shuffled_ids[@]}"; do
            current_wallpaper_id="$id"
            load_wallpaper "$id"
            save_current_wallpaper_id
        done
    else
        echo "无效模式。请选择 1 (顺序播放)，2 (随机)，或 3 (随机不重复)。"
        exit 1
    fi
done
