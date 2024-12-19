# Three Finger Drag for Wayland/KDE
This program builds off marsqing's [`libinput-three-finger-drag`](https://github.com/marsqing/libinput-three-finger-drag), adapting it for computers with touchpads running in Wayland sessions (notably KDE Plasma 6). It does this by substituing the `xdo` calls in marsqing's original for write-calls directly to [`/dev/uinput`](https://www.kernel.org/doc/html/v4.12/input/uinput.html) (via Rust's `input-linux` crate), which lies beneath the display server layer. This (in theory) allows the program to run in any desktop environment that has libinput installed, which includes both KDE Plasma and GNOME.

## What is three-finger dragging?

Three-finger dragging is a feature originally for trackpads on Mac devices: instead of holding down the left click on the pad to drag, you can simply rest three fingers on the trackpad to start a mouse hold, and move the fingers together to continue the drag in whatever direction you move them in. In short, it interprets three fingers on the trackpad as a mouse-down input, and motion with three fingers afterwards for mouse movement. It can be quite handy, as it will save your hand some effort for moving windows around and highlighting text. 

Here is [an example](https://www.youtube.com/watch?v=-Fy6imaiHWE) of three-finger dragging in action on a MacBook.

## Installation

### 1. Clone the repository
```
git clone https://github.com/lmr97/linux-3-finger-drag.git
cd linux-3-finger-drag
```

### 2. Disable 3 finger swipe gesture in libinput-gestures (if needed)

If you haven't installed `libinput-gestures`, you can skip this step. 

If you have, though, modify the config file `/etc/libinput-gestures.conf` or `~/.config/libinput-gestures.conf`. 
Add finger_count 4 to essentially disable 3 finger swipes.

change
``` 
gesture swipe up     xdotool key super+Page_Down 
```
to
```
gesture swipe up  4  xdotool key super+Page_Down
```
(The only difference is the 4 before "xdotool").

### 3. Update permissions

#### 3.1: For `/dev/uinput`
We need to make `/dev/uinput` accessible to all logged-in users, so the program doesn't require root permissions to run. For more info about what's being done here, see [this section](https://wiki.archlinux.org/title/Udev#Allowing_regular_users_to_use_devices) of the ArchWiki article on `udev`. 

```
# defining a variable here saves us from quote escape hell
RULES="KERNEL==\"uinput\", MODE=\"0660\", TAG+=\"uaccess\""
sudo sh -c "echo $RULES >> /etc/udev/rules.d/60-uinput.rules"
sudo udevadm control --reload
sudo udevadm trigger
```

(It's not important that the number prepended to `uinput.rules` is 60, any number less than 70 will do, since that's the number with which the rules for `uaccess` starts, and, with a lexical precidence, our uinput rules need to be read first).

#### 3.2: For `libinput`

`libinput` will only let members of the group `input` read its debug output, so add yourself to the group by running:
```
sudo gpasswd --add your_username_here input
```

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

`mkdir -p ~/.config/systemd/user`

After that copy the service file in this repo to the folder you just created (or just the folder if you already have one):

`cp three-finger-drag.service ~/.config/systemd/user/`

Now you just need to enable and start the service:

`systemctl --user enable --now three-finger-drag.service`

### You did it! Now you can 3-finger-drag!


## Configuration
There is a JSON configuration file, assumed to be in `~/.config/linux-3-finger-drag/` called `3fd-config.json`, which is read into the program at startup. You can specify an acceleration value (`acceleration`), which will be multiplied with all 3-finger gesture movements. You can also specify the time (in milliseconds) that the mouse hold will persist for after you lift your fingers (to give you a moment to reposition your fingers), with `drag_end_delay`. It's entirely optional: if the file cannot be read for any reason, the program will simply warn the user that the file could not be read (with the reason), and default to an acceleration multiplier of 1 and a drag end delay value of 0. 

## How it works
Much like `libinput-three-finger-drag`, this program reads from the `libinput debug-events` command and parses the raw text, but here is where this fork diverges from the original: it defines a virtual mouse through `/dev/uinput` and translates three finger touch events parsed from the debug output into mouse presses and mouse movement for the virtual mouse.
