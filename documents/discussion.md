# A Player Tool for Linux-Wallpaperengine

Linux-wallpaperengine is a fantastic tool that makes it possible to play live wallpapers on GNU/Linux.

## But I am struggling with checking and picking wallpapers!

Every time I find a new live wallpaper from the Steam Workshop, I need to open Wallpaper Engine via Steam Proton, and locate the wallpaper ID using an awkward file manager running on Wine. It's even difficult to select and copy the wallpaper ID! **_Just as shown below_**

![image](https://github.com/user-attachments/assets/af393197-4100-4b23-bf30-1a3ea09df191)

After that, I need to run linux-wallpaperengine using Bash. **_All of this wastes a lot of my time._**

## Besides, I want to play a list of wallpapers just like I do in Windows!

So, I posted an issue:

[Is it possible to change the background from a playlist?](https://github.com/Almamu/linux-wallpaperengine/issues/221)

It was marked as an enhancement, which is great. I then tried to implement this enhancement.

Unfortunately, using C++ was too difficult for me...

So, I decided to create another tool to achieve this enhancement.

## Now, I have created a player tool named [linux-wallpaperengine-player](https://github.com/DI-HUO-MING-YI/linux-wallpaperengine-player)

The player is developed entirely in Python.

### Main features:

- Pick and check a single wallpaper using Wallpaper Engine's UI.
- Batch check wallpapers using Wallpaper Engine's folders.
- Play wallpapers using a playlist.

### Now this tool **_just works_**

### For more information, please visit the [REPO](https://github.com/DI-HUO-MING-YI/linux-wallpaperengine-player)
