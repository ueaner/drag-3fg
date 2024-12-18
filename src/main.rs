extern crate regex;
extern crate signal_hook;
extern crate serde;
extern crate serde_json;

use regex::Regex;
use std::io::{self, BufRead};
use std::fs::read_to_string;
use std::iter::Iterator;
use std::process::{Command, Stdio};
//use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use serde::Deserialize;
use serde_json::from_str;

mod uinput_handler;

#[derive(Deserialize)]
struct Configuration {
    acceleration: f32,
    drag_end_delay: u64  // in milliseconds
}

// Configs are so optional that their absence should not crash the program,
// So if there is any issue with the JSON config file, 
// the following default values will be returned:
//
//      acceleration = 1.0
//      dragEndDelay = 0
//
// The user is also warned about this, so they can address the issues 
// if they want to configure the way the program runs.
fn parse_config_file(filepath: &str) -> Configuration {
    
    let Ok(jsonfile) = read_to_string(filepath) else {
        println!("WARNING: Unable to locate JSON file at {}; using defaults of:

            acceleration = 1.0
            dragEndDelay = 0
            ", filepath);

        return Configuration {
            acceleration: 1.0, 
            drag_end_delay: 0
        };
    };

    let Ok(config) = from_str::<Configuration>(&jsonfile) else {
        println!("WARNING: Bad formatting found in JSON file, falling back on defaults of:
    
            acceleration = 1.0
            dragEndDelay = 0
            ");
    
        return Configuration {
            acceleration: 1.0, 
            drag_end_delay: 0
        };
    };

    config
}
 
fn main() {

    // handling SIGINT and SIGTERM
    let sigterm_received = Arc::new(AtomicBool::new(false));
    let sigint_received  = Arc::new(AtomicBool::new(false));

    signal_hook::flag::register(
        signal_hook::consts::SIGTERM, 
        Arc::clone(&sigterm_received)
    ).unwrap();

    signal_hook::flag::register(
        signal_hook::consts::SIGINT, 
        Arc::clone(&sigint_received)
    ).unwrap();
    
    // let args: Vec<String> = env::args().collect();
    // let acceleration: f32;
    // if args.len() > 1 {
    //     acceleration = args[1].parse::<f32>().unwrap_or(1.0);
    // } else {
    //     acceleration = 1.0;
    // }

    let configs = parse_config_file("~/.config/linux-3-finger-drag/config.json");
    let mut vtrackpad = uinput_handler::start_handler();

    let output = Command::new("stdbuf")
        .arg("-o0")
        .arg("libinput")
        .arg("debug-events")
        .stdout(Stdio::piped())
        .spawn()
        .expect("You are not yet allowed to read libinput's debug events.
        Have you added yourself to the group \"input\" yet?
        (see installation section in README, step 3.2)
        ")
        .stdout
        .expect("libinput has no stdout");


    let mut xsum: f32 = 0.0;
    let mut ysum: f32 = 0.0;
    let pattern = Regex::new(r"[\s]+|/|\(").unwrap();

    for line in io::BufReader::new(output).lines() {
        
        // handle interrupts
        if sigterm_received.load(Ordering::Relaxed) || sigint_received.load(Ordering::Relaxed) {
            break;
        }

        let line = line.unwrap();
        if line.contains("GESTURE_") {
            // event10  GESTURE_SWIPE_UPDATE +3.769s	4  0.25/ 0.48 ( 0.95/ 1.85 unaccelerated)
            let parts: Vec<&str> = pattern.split(&line).filter(|c| !c.is_empty()).collect();
            let action = parts[1];
            let finger = parts[3];
            if finger != "3" && !action.starts_with("GESTURE_HOLD"){
                // mouse_down
                vtrackpad.mouse_down();
                continue;
            }
            let cancelled = parts.len() > 4 && parts[4] == "cancelled";

            match action {
                "GESTURE_SWIPE_BEGIN" => {
                    xsum = 0.0;
                    ysum = 0.0;
                    // mouse_down
                    vtrackpad.mouse_down();
                }
                "GESTURE_SWIPE_UPDATE" => {
                    let x: f32 = parts[4].parse().unwrap();
                    let y: f32 = parts[5].parse().unwrap();
                    xsum += x * configs.acceleration;
                    ysum += y * configs.acceleration;
                    if xsum.abs() > 1.0 || ysum.abs() > 1.0 {
                        // mouse_move_relative
                        vtrackpad.mouse_move_relative(xsum, ysum);
                        xsum = 0.0;
                        ysum = 0.0;
                    }
                }
                "GESTURE_SWIPE_END" => {
                    // mouse_move_relative
                    vtrackpad.mouse_move_relative(xsum, ysum);

                    if cancelled {
                        // mouse_up
                        vtrackpad.mouse_up();
                    } else {
                        // mouse_up, with 300ms delay
                        vtrackpad.mouse_up_delay(configs.drag_end_delay);
                    }
                }
                "GESTURE_HOLD_BEGIN" => {
                    // Ignore
                }
                "GESTURE_HOLD_END" => {
                    // Ignore accidental holds when repositioning
                    if !cancelled {
                        // mouse_up
                        vtrackpad.mouse_up();
                    }
                }
                _ => {
                    // GESTURE_PINCH_*,
                    // mouse_up
                    vtrackpad.mouse_up();
                }
            }
        } else {
            // mouse_up
        }
    }

    vtrackpad.mouse_up();       // just in case
    vtrackpad.dev_destroy();    // we don't need virtual devices cluttering the system

}
