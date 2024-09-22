#!/bin/bash

# 默认配置文件路径
default_config_file="$HOME/.config/wallpaperengine/config.conf"

# 检查是否提供配置文件路径，如果没有提供则使用默认配置文件
config_file="${1:-$default_config_file}"

# 检查配置文件是否存在
if [ ! -f "$config_file" ]; then
    echo "配置文件不存在: $config_file"
    exit 1
fi

# 打印配置文件路径
echo "使用的配置文件: $config_file"

# 读取配置文件
source "$config_file"

# 检查配置文件中的必需参数
if [ -z "$id_file" ] || [ -z "$wallpaper_dir" ] || [ -z "$mode" ] || [ -z "$interval" ] || [ -z "$log_file" ] || [ -z "$wallpaperengine_log_file" ]; then
    echo "请确保配置文件中的所有参数都已设置。"
    exit 1
fi

# 打印配置项
echo "配置项:"
echo "壁纸id列表文件: $id_file"
echo "壁纸下载地址: $wallpaper_dir"
echo "播放模式: $mode"
echo "播放间隔: $interval"
echo "脚本日志文件: $log_file"
echo "linux-wallpaperengine日志文件: $wallpaperengine_log_file"

# 记录所有输出到log_file
exec > >(tee -a "$log_file") 2>&1

# 读取文件中的所有ID
mapfile -t ids <"$id_file"

# 检查文件夹和加载壁纸的函数
check_and_load_wallpaper() {
    local id="$1"
    local project_json="$wallpaper_dir/$id/project.json"

    if [ -d "$wallpaper_dir/$id" ]; then
        if [ -f "$project_json" ]; then
            type=$(jq -r '.type | ascii_downcase' "$project_json")
            if [ "$type" != "scene" ]; then
                echo "加载壁纸: $id"
                pgrep -f linux-wallpaperengine | xargs kill

                linux-wallpaperengine \
                    --scaling fill \
                    --screen-root HDMI-A-1 \
                    --screen-root eDP-1 \
                    "$id" \
                    --clamping border \
                    --fps 240 \
                    --volume 50 \
                    >"$wallpaperengine_log_file" \
                    2>&1 &

                sleep 2
                sleep "$interval"
            else
                echo "跳过壁纸: $id (暂不支持场景类型的壁纸)"
            fi
        else
            echo "项目文件不存在: $project_json"
        fi
    else
        echo "未找到文件夹: $id"
    fi
}

# 开始循环
while true; do
    if [ "$mode" -eq 1 ]; then
        # 顺序播放
        for id in "${ids[@]}"; do
            check_and_load_wallpaper "$id"
        done
    elif [ "$mode" -eq 2 ]; then
        # 随机播放
        id="${ids[RANDOM % ${#ids[@]}]}"
        check_and_load_wallpaper "$id"
    elif [ "$mode" -eq 3 ]; then
        # 随机不重复播放
        shuffled_ids=($(shuf -e "${ids[@]}"))
        for id in "${shuffled_ids[@]}"; do
            check_and_load_wallpaper "$id"
        done
    else
        echo "无效模式。请选择 1 (顺序播放)，2 (随机)，或 3 (随机不重复)。"
        exit 1
    fi
done
