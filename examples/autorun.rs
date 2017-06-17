extern crate rusty_hotkey;

use rusty_hotkey::{intercept_input, get_toggle_state, send_input};
use rusty_hotkey::Input::{KeybdRelease, KeybdPress};
use std::time::Duration;
use std::thread::sleep;

fn main() {
    while let Some(input) = intercept_input() {
        match input {
            //An autorun for videogames activated when NumLock is on
            KeybdRelease(144) => {
                while get_toggle_state(0x90) {
                    send_input(KeybdPress(0x10));
                    send_input(KeybdPress(0x57));
                    sleep(Duration::from_millis(50));
                    send_input(KeybdRelease(0x10));
                    send_input(KeybdRelease(0x57));
                }
            },
            _ => {}
        }
    }
}