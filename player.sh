#!/bin/bash

cd /home/narcissus/Workspace/wallpaperengine/linux-wallpaperengine-player/ || exit
cargo run -- --config config.toml play --playlist test
