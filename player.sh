#!/bin/bash

cd /home/narcissus/Workspace/wallpaperengine/linux-wallpaperengine-player/ || exit
cargo run -- --config config.toml sddm --folder 妹子_picked
cargo run -- --config config.toml play --playlist test
