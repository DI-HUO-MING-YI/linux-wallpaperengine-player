#!/bin/bash

# 检查文件夹和加载壁纸的函数
source ~/.config/wallpaperengine/load_wallpaper.sh

# 读取已检查的ID
checked_file="${id_file}_checked"
mapfile -t checked_ids < <(sed 's/#.*//' "$checked_file")

interval=0
# 检查文件夹和加载壁纸的函数
check_and_load_wallpaper() {
    local id="$1"

    if [[ " ${checked_ids[@]} " =~ " ${id} " ]]; then
        echo "ID $id 已存在于结果文件中，跳过。"
        return
    fi

    if [ -d "$wallpaper_dir/$id" ]; then
        load_wallpaper "$id"
        # 等待用户输入
        echo -n "是否保存当前壁纸ID [$id] (y/n)? "
        read -r answer
        if [[ "$answer" == "y" ]]; then
            echo "$id" >>"$checked_file"
            echo "已保存ID $id 到 $checked_file"
        elif [[ "$answer" == "n" ]]; then
            echo "$id#skip" >>"$checked_file"
            echo "ID $id 标记为skip"
        fi
    else
        echo "$id#not_found" >>"$checked_file"
    fi
}

# 顺序播放
for id in "${ids[@]}"; do
    check_and_load_wallpaper "$id"
done
