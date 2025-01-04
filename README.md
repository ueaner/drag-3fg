# Three Finger Drag for Wayland/KDE
This program builds off marsqing's [`libinput-three-finger-drag`](https://github.com/marsqing/libinput-three-finger-drag), expanding it to support computers with touchpads running in Wayland sessions (notably KDE Plasma 6). It only depends on `libinbut` and `uinput`, so the program should run in any desktop environment that has `libinput` installed, reagrdless of whether it uses X or Wayland.

## Tested on...

OS | Version | Desktop Enviroment | Supported
---|---|---|---
**Kubuntu** | 24.10 | KDE Plasma 6| ✅ <sup>1</sup>
**Pop_OS!** | (unknown) | COSMIC | ✅
**Arch** | 2025-01-01 | GNOME | ✅

---
<sup>1</sup> Developed on this setup.

## What is three-finger dragging?

Three-finger dragging is a feature originally for trackpads on Mac devices: instead of holding down the left click on the pad to drag, you can simply rest three fingers on the trackpad to start a mouse hold, and move the fingers together to continue the drag in whatever direction you move them in. In short, it interprets three fingers on the trackpad as a mouse-down input, and motion with three fingers afterwards for mouse movement. It can be quite handy, as it will save your hand some effort for moving windows around and highlighting text. 

Here is [an example](https://www.youtube.com/watch?v=-Fy6imaiHWE) of three-finger dragging in action on a MacBook.

## Installation

### 0. Install `libinput` helper tools (you may have it already)

If you are using GNOME or KDE Plasma for your desktop environment, you already have `libinput` installed (it's a dependency of those environments). There is also a set of helper tools for `libinput` accessible from the command line that `linux-3-finger-drag` needs to function properly, which may also be already installed. Regardless, you can confirm whether you have these helper tools by running:
```
libinput --version
```
If you get a version number from this command, you can proceed to the next step.

On Debian/Ubuntu-based distributions, you may need to install the `libinput-tools` package to get the helper tools.

For other distributions, or for building from the source (which will include the helper tools), see the [`libinput`'s build instructions](https://wayland.freedesktop.org/libinput/doc/latest/building.html#building). With other distros, you may need to install the helper tools separately.


### 1. Clone the repository
```
git clone https://github.com/lmr97/linux-3-finger-drag.git
cd linux-3-finger-drag
```

### 2. Change any existing 3-finger gestures to 4-finger gestures

#### For GNOME users

GNOME users will need to install the Window Gestures Shell Extension. Once installed, you'll be able to change the finger number for swipe gestures from your settings. You can get it from either the [GNOME Extensions website](https://extensions.gnome.org/extension/6343/window-gestures/) or the [GitHub repository](https://github.com/amarullz/windowgestures). Once installed, disable all three finger gestures. 

#### For `libinput-gestures` (if needed)

If you haven't installed `libinput-gestures`, you can skip to the next step. 

If you have, though, modify the config file `/etc/libinput-gestures.conf` or `~/.config/libinput-gestures.conf`. 
Add 4 in the finger_count column to convert 3 finger swipes to 4 finger swipes, to prevent confusion for the desktop environment and frustration for yourself.

change
``` 
gesture swipe up     xdotool key super+Page_Down 
```
to
```
gesture swipe up  4  xdotool key super+Page_Down
```
(The only difference is the 4 before "xdotool").

#### For other extensions/programs (like [wzmach](https://github.com/maurges/wzmach))

The process is essentially the same: there is typically a configuration file somewhere that includes the number of fingers for swipe gestures, and if there are any responding to 3-finger swipes, increase the finger count to 4. Consult your program's documentation for the specifics.

If there's enough interest, I'll add an option to configure the number of fingers this program responds to to start the drag. 

### 3. Update permissions

This programs reads from `libinput`, and writes to `/dev/uinput`, and it requires an adjustment of permissions to accomplish both. 

#### 3.1: For `/dev/uinput`
We need to alter the rules for `/dev/uinput` to make it accessible to all logged-in users, so the program doesn't require root permissions to run. For more info about what's being done here, see [this section](https://wiki.archlinux.org/title/Udev#Allowing_regular_users_to_use_devices) of the ArchWiki article on `udev`. 
You may need to create the folder `rules.d` in `/etc/udev`.

<u>**For Arch users**</u>: You will need to set the `uinput` kernel module to load on boot, if you haven't already. For instructions on this, see the [relevant ArchWiki page](https://wiki.archlinux.org/title/Kernel_module#Automatic_module_loading).
```
sudo cp ./60-uinput.rules /etc/udev/rules.d
```

#### 3.2: For `libinput`

`libinput` will only let members of the group `input` read its debug output, so add yourself to the group by running:
```
sudo gpasswd --add your_username_here input
```
You will need to **reboot** to update the groups. 

### 4. Build with Cargo
```
cargo build --release
```

### 5. Verify funtionality
Check to make sure the executable works by running
```
./target/release/linux-3-finger-drag
```

You will see a warning about not being able to find a configuration file. Just ignore that for now, as it will not impact functionality; we'll get to configuration in step 7. 

I've tried to make error messages as informative as I can, with recommended actions included to remedy them, so look to those error messages for help first. But if they don't help, please submit an issue here, with information about your OS, desktop environment, and any error output you get, and I'll get on it as soon as I can. 

### 6. Install into `/usr/bin`
Once you've got it working, copy it into `/usr/bin` for ease and consistency of access:
```
sudo cp ./target/release/linux-3-finger-drag /usr/bin
```

### 7. Set up configuration file (optional)

See the Configuration section below about the included `3fd-config.json` file. If you'd like to configure the behavior of this program, run the following:
```
mkdir ~/.config/linux-3-finger-drag
cp 3fd-config.json ~/.config/linux-3-finger-drag
```
Now you can configure to your heart's content!

### 8. Add program to Autostart (KDE)
This is a part of the graphical interface. You can find the Autostart menu in System Settings > Autostart (near the bottom). Once there, click the "+ Add..." button in the upper right of the window, and select "Add Application" from the dropdown menu. Then, in text bar in the window that pops up, paste
```
/usr/bin/linux-3-finger-drag
```
and click OK. 

Now select the program in the Autostart menu, and press Start in the upper right-hand corner of the window to start using it in the current session. It will automatically start in the next session you log into.

### 8b. Add program to Autostart (systemd, works distro and desktop agnostic)

Alternatively you can add this program to the autostart in any linux desktop and autostart it via systemd. To do this copy this file create the local systemd folder if not already created:

```
mkdir -p ~/.config/systemd/user
```

After that copy the service file in this repo to the folder you just created (or just the folder if you already have one):

```
cp three-finger-drag.service ~/.config/systemd/user/
```

Now you just need to enable and start the service:

```
systemctl --user enable --now three-finger-drag.service
```

### You did it! Now you can 3-finger-drag!


## Configuration
There is a JSON configuration file, assumed to be in `~/.config/linux-3-finger-drag/` called `3fd-config.json`, which is read into the program at startup. You can specify an acceleration value (`acceleration`), which will be multiplied with all 3-finger gesture movements. You can also specify the time (in milliseconds) that the mouse hold will persist for after you lift your fingers (to give you a moment to reposition your fingers), with `drag_end_delay`. It's entirely optional: if the file cannot be read for any reason, the program will simply warn the user that the file could not be read (with the reason), and default to an acceleration multiplier of 1 and a drag end delay value of 0. 

## How it works
This program uses the regex parsing structure of marsqing's `libinput-three-finger-drag` to detect three-finger gestures, and translating them into write-calls to [`/dev/uinput`](https://www.kernel.org/doc/html/v4.12/input/uinput.html) via a virtual mouse. This flow of control bypasses the display server layer entirely, which ensures compatability with any desktop environment (at least with some modifications).