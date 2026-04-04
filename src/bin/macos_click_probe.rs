//! Standalone macOS mouse-click probe (no macroquad).
//!
//! Polls raw mouse button state through `device_query` and prints transitions
//! (down/up) for left, right, and middle buttons to stdout.

#[cfg(target_os = "macos")]
use device_query::{DeviceQuery, DeviceState};
#[cfg(target_os = "macos")]
use std::thread;
#[cfg(target_os = "macos")]
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(target_os = "macos")]
fn timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis())
}

#[cfg(target_os = "macos")]
fn main() {
    let Some(device_state) = DeviceState::checked_new() else {
        eprintln!(
            "Accessibility permission is required.\n\
             Grant permission to your terminal app in:\n\
             System Settings -> Privacy & Security -> Accessibility"
        );
        std::process::exit(1);
    };

    println!("macOS click probe started (no macroquad).");
    println!("Press Ctrl+C to stop.");
    println!("Tracking left/right/middle transitions...");

    let mut prev_left = false;
    let mut prev_right = false;
    let mut prev_middle = false;

    loop {
        let mouse = device_state.get_mouse();
        let left = mouse.button_pressed.get(1).copied().unwrap_or(false);
        let right = mouse.button_pressed.get(2).copied().unwrap_or(false);
        let middle = mouse.button_pressed.get(3).copied().unwrap_or(false);
        let (x, y) = mouse.coords;

        if left != prev_left {
            let state = if left { "DOWN" } else { "UP" };
            println!("[{}] LEFT   {} at ({}, {})", timestamp_ms(), state, x, y);
            prev_left = left;
        }
        if right != prev_right {
            let state = if right { "DOWN" } else { "UP" };
            println!("[{}] RIGHT  {} at ({}, {})", timestamp_ms(), state, x, y);
            prev_right = right;
        }
        if middle != prev_middle {
            let state = if middle { "DOWN" } else { "UP" };
            println!("[{}] MIDDLE {} at ({}, {})", timestamp_ms(), state, x, y);
            prev_middle = middle;
        }

        thread::sleep(Duration::from_millis(1));
    }
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("This probe is only available on macOS.");
}
