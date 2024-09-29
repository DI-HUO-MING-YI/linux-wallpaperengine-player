import os
import sys
import json

if len(sys.argv) < 2:
    print("请提供JSON文件的路径")
    sys.exit(1)

json_file = sys.argv[1]

if not os.path.isfile(json_file):
    print(f"文件不存在: {json_file}")
    sys.exit(1)

with open(json_file, 'r') as file:
    data = json.load(file)
    folders = data['Narci']['general']['browser']['folders']

    for folder in folders:
        title = folder['title']
        file_name = f"{title}.txt"
        with open(file_name, 'w') as f:
            for key in folder['items'].keys():
                f.write(f"{key}\n")

        subfolders = folder.get('subfolders', [])
        if subfolders:
            for subfolder in subfolders:
                subfolder_title = subfolder['title']
                combined_title = f"{title}_{subfolder_title}"
                subfile_name = f"{combined_title}.ids"
                with open(subfile_name, 'w') as subf:
                    for key in subfolder['items'].keys():
                        subf.write(f"{key}\n")