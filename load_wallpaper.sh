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
echo "current_wallpaper_id: $current_wallpaper_id"

# 记录所有输出到log_file
exec > >(tee -a "$log_file") 2>&1

# 读取文件中的所有ID
mapfile -t ids < <(grep -v '#.*' "$id_file")

load_wallpaper() {
  local id="$1"

  if [ -d "$wallpaper_dir/$id" ]; then
    pgrep -f linux-wallpaperengine | xargs kill
    sleep 1
    echo "加载壁纸: $id"

    # 构建linux-wallpaperengine命令
    local base_command="linux-wallpaperengine"

    # 遍历所有 screen_root
    for i in "${!screen_root[@]}"; do
      local command="$base_command --screen-root ${screen_root[$i]}"

      # 如果是第一个 screen_root，添加配置文件中的参数
      if [ "$i" -eq 0 ]; then
        [[ -n "$silent" ]] && [[ "$silent" == true ]] && command+=" --silent"
        [[ -n "$volume" ]] && command+=" --volume $volume"
        [[ -n "$noautomute" ]] && [[ "$noautomute" == true ]] && command+=" --noautomute"
        [[ -n "$no_audio_processing" ]] && [[ "$no_audio_processing" == true ]] && command+=" --no-audio-processing"
      else
        # 后续的 screen_root 只添加 --silent
        command+=" --silent"
      fi

      [[ -n "$scaling" ]] && command+=" --scaling $scaling"
      [[ -n "$window" ]] && [[ "$window" == true ]] && command+=" --window"
      [[ -n "$fps" ]] && command+=" --fps $fps"
      [[ -n "$assets_dir" ]] && command+=" --assets-dir $assets_dir"
      [[ -n "$screenshot" ]] && [[ "$screenshot" == true ]] && command+=" --screenshot"
      [[ -n "$list_propertites" ]] && [[ "$list_propertites" == true ]] && command+=" --list-propertites"

      if [[ -n "$set_property" ]] && [[ "${#set_property[@]}" -gt 0 ]]; then
        for sp in "${set_property[@]}"; do
          command+=" --set-property $sp"
        done
      fi

      [[ -n "$no_fullscreen_pause" ]] && [[ "$no_fullscreen_pause" == true ]] && command+=" --no-fullscreen-pause"
      [[ -n "$disable_mouse" ]] && [[ "$disable_mouse" == true ]] && command+=" --disable-mouse"
      [[ -n "$clamping" ]] && command+=" --clamping $clamping"
      command+=" \"$id\""

      echo "执行命令: $command"
      eval "$command > \"${wallpaperengine_log_file}_in_${screen_root[$i]}\" 2>&1 &"
      sleep 1
    done

    sleep "$interval"
  else
    echo "未找到文件夹: $id"
  fi
}
