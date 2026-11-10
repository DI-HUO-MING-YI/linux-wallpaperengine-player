# linux-wallpaperengine-player

A small CLI for controlling Waywallen and preparing Wallpaper Engine Workshop videos for SDDM and wallock.

## Features

- Switch Waywallen wallpapers with Next and Previous.
- Pause and resume Waywallen renderers.
- Restart the Waywallen daemon with Reload.
- Select a random Workshop video for SDDM or wallock.
- Export existing wallpaper play history to CSV.

## Requirements

- Waywallen 0.6.0
- Steam and Wallpaper Engine Workshop content for SDDM and wallock
- Rust and Cargo

The default Waywallen executable is `/home/narcissus/Application/waywallen-0.6.0-x86_64.AppImage`.
Set `WAYWALLEN_EXECUTABLE` to use a different installation.

## Build

```bash
cargo build --release
```

## Waywallen control

```bash
linux-wallpaperengine-player control --next
linux-wallpaperengine-player control --prev
linux-wallpaperengine-player control --pause
linux-wallpaperengine-player control --mute
linux-wallpaperengine-player control --reload
linux-wallpaperengine-player control --open-ui
```

`--pause` toggles between paused and playing states.

`--reload` performs a clean D-Bus shutdown, starts Waywallen with `--no-ui`, and waits for its D-Bus service and WebSocket port to become ready.

`--mute` toggles between muted and unmuted states.

The `--stop` and `--continue` arguments are reserved for future playlist activation controls and currently have no implementation.

## SDDM and wallock

Both commands read Wallpaper Engine folders from the configured Workshop library and select a random video.

```bash
linux-wallpaperengine-player --config config.toml sddm --folder picked
linux-wallpaperengine-player --config config.toml wallock --folder picked
```

## Export play history

```bash
linux-wallpaperengine-player --config config.toml export --file history.csv
```
