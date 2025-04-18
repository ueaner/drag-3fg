// this file is basically copied and rearranged from arcnmx's GitHub example
// in the input-linux-rs repo (a translation of an example
// on the Linux kernel's uinput module, actually). 
// The Rust example can be found here: 
// https://github.com/arcnmx/input-linux-rs/blob/main/examples/mouse-movements.rs

use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::{thread, time};

use input_linux::{
    EventKind, EventTime, 
    InputEvent, InputId, 
    Key, KeyEvent, KeyState, 
    RelativeAxis, RelativeEvent, 
    SynchronizeEvent, SynchronizeKind, 
    UInputHandle
};
use nix::libc::O_NONBLOCK;

// so I can attach mouse specific methods to this 
// UInputHandle meant to model a mouse
pub struct VirtualTrackpad {
    handle: UInputHandle<File>,
    mouse_is_down: bool
}

pub fn start_handler() -> VirtualTrackpad {
    let uinput_file = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(O_NONBLOCK)
        .open("/dev/uinput")
        .expect("
            You are not yet allowed to write to /dev/uinput.
            Some things to try:
            - Update the udev rules for uinput (see installation guide in README.md, step 3.1)
            - Log out and log in again
            - Restart your computer
            - FOR ARCH: make sure the uinput kernel module is loaded on boot
            ");
    let uhandle = UInputHandle::new(uinput_file);

    uhandle.set_evbit(EventKind::Key).unwrap();
    uhandle.set_keybit(input_linux::Key::ButtonLeft).unwrap();

    uhandle.set_evbit(EventKind::Relative).unwrap();
    uhandle.set_relbit(RelativeAxis::X).unwrap();
    uhandle.set_relbit(RelativeAxis::Y).unwrap();

    let input_id = InputId {
        bustype: input_linux::sys::BUS_USB,
        vendor: 0x1234,
        product: 0x5678,  // iykyk
        version: 0,
    };
    let device_name = b"Virtual trackpad (made by linux-three-finger-drag)";
    uhandle.create(&input_id, device_name, 0, &[]).unwrap();

    // needed to let the system catch up
    thread::sleep(time::Duration::from_secs(1));

    VirtualTrackpad {
        handle: uhandle,
        mouse_is_down: false
    }

}

impl VirtualTrackpad
{
    pub const ZERO: EventTime = EventTime::new(0, 0);

    pub fn mouse_down(&mut self) {
        let events = [
            InputEvent::from(
                KeyEvent::new(
                    VirtualTrackpad::ZERO, 
                    Key::ButtonLeft, 
                    KeyState::pressed(true))
                ).into_raw(),
            InputEvent::from(
                SynchronizeEvent::new(
                    VirtualTrackpad::ZERO, 
                    SynchronizeKind::Report, 
                    0)
                ).into_raw(),
        ];
        self.handle.write(&events).unwrap();
        self.mouse_is_down = true;
    }

    pub fn mouse_up(&mut self) {    
        let events = [
            InputEvent::from(
                KeyEvent::new(
                    VirtualTrackpad::ZERO, 
                    Key::ButtonLeft, 
                    KeyState::pressed(false))
                ).into_raw(),
            InputEvent::from(
                SynchronizeEvent::new(
                    VirtualTrackpad::ZERO, 
                    SynchronizeKind::Report, 
                    0)
                ).into_raw(),
        ];
        self.handle.write(&events).unwrap();
        self.mouse_is_down = false;
    }

    // delay is in milliseconds
    pub fn mouse_up_delay(&mut self, delay: u64) {
        thread::sleep(time::Duration::from_millis(delay));

        let events = [
            InputEvent::from(
                KeyEvent::new(
                    VirtualTrackpad::ZERO, 
                    Key::ButtonLeft, 
                    KeyState::pressed(false))
                ).into_raw(),
            InputEvent::from(
                SynchronizeEvent::new(
                    VirtualTrackpad::ZERO, 
                    SynchronizeKind::Report, 
                    0)
                ).into_raw(),
        ];
        self.handle.write(&events).unwrap();
        self.mouse_is_down = false;
    }

    pub fn mouse_move_relative(&self, x_rel: f32, y_rel:f32) {
        // RelativeEvent::new() can only take integers, 
        // so some precision must be lost

        let x_rel_int = x_rel.ceil() as i32;
        let y_rel_int = y_rel.ceil() as i32;

        let events = [
            InputEvent::from(
                RelativeEvent::new(
                    VirtualTrackpad::ZERO, 
                    RelativeAxis::X, 
                    x_rel_int)
                ).into_raw(),
            InputEvent::from(
                RelativeEvent::new(
                    VirtualTrackpad::ZERO, 
                    RelativeAxis::Y, 
                    y_rel_int)
                ).into_raw(),
            InputEvent::from(
                SynchronizeEvent::new(
                    VirtualTrackpad::ZERO, 
                    SynchronizeKind::Report, 
                    0)
                ).into_raw(),
        ];
        self.handle.write(&events).unwrap();
    }

    pub fn dev_destroy(&self) {
        self.handle.dev_destroy().unwrap();
    }
}