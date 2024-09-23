#!/bin/bash

# 检查是否提供了 JSON 文件的路径
if [ -z "$1" ]; then
    echo "请提供JSON文件的路径"
    exit 1
fi

# 使用第一个参数作为 JSON 文件的路径
json_file="$1"

# 检查文件是否存在
if [ ! -f "$json_file" ]; then
    echo "文件不存在: $json_file"
    exit 1
fi

# 解析 folders 数组中的每一项
jq -c '.Narci.general.browser.folders[]' "$json_file" | while read -r folder; do
    # 提取父节点的 title
    title=$(echo "$folder" | jq -r '.title')

    # 处理父节点 items，创建文件
    file_name="${title}.txt"
    touch "$file_name"

    # 提取父节点 items 中的所有键，写入父节点文件
    echo "$folder" | jq -r '.items | keys[]' | while read -r key; do
        echo "$key" >> "$file_name"
    done

    # 检查 subfolders
    subfolders=$(echo "$folder" | jq '.subfolders')
    if [ "$subfolders" != "[]" ]; then
        # 遍历 subfolders
        echo "$subfolders" | jq -c '.[]' | while read -r subfolder; do
            subfolder_title=$(echo "$subfolder" | jq -r '.title')
            combined_title="${title}_${subfolder_title}"

            # 创建以合并后的 title 命名的文件
            subfile_name="${combined_title}.ids"
            touch "$subfile_name"

            # 提取 subfolder items 中的所有键，写入文件
            echo "$subfolder" | jq -r '.items | keys[]' | while read -r key; do
                echo "$key" >> "$subfile_name"
            done
        done
    fi
done
