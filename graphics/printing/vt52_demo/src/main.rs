use libnx_rs::{libnx::*, LibnxError};
use std::{ptr, result::Result};

fn esc(code: &str) -> String {
    format!("\x1b[{}", code)
}

fn esc_color(code: u8) -> String {
    esc(&format!("{};1m", code))
}

fn main() -> Result<(), LibnxError> {
    // Initialize console interface
    // Using NULL as argument tells the console library to use the internal console structure as current one
    unsafe { consoleInit(ptr::null_mut()) };

    // clear screen and home cursor
    println!("{}", esc("2J"));

    // Set print co-ordinates
    // /x1b[row;columnH
    println!("{}VT52 codes demo", esc("10;10H"));

    // move cursor up
    // /x1b[linesA
    println!("{}Line 0", esc("10A"));

    // move cursor left
    // /x1b[columnsD
    println!("{}Column 0", esc("28D"));

    // move cursor down
    // /x1b[linesB
    println!("{}Line 19", esc("19B"));

    // move cursor right
    // /x1b[columnsC
    println!("{}Column 20", esc("5C"));

    println!("\n");

    // Color codes and attributes
    for i in 30..38 {
        println!(
            "{}Default {}Bold {}Reversed {}{}{}Light {}Reversed {}{}{}Underline {}{}{}Strikethrough {}",
            esc_color(i),
            esc("1m"),
            esc("7m"),
            esc("0m"),
            esc_color(i),
            esc("2m"),
            esc("7m"),
            esc("0m"),
            esc_color(i),
            esc("4m"),
            esc("0m"),
            esc_color(i),
            esc("9m"),
            esc("0m"),
        );
    }

    while unsafe { appletMainLoop() } {
        // Get the identifier of the last pressed key
        let key = unsafe {
            // Scan all the inputs. This should be done once for each frame
            hidScanInput();

            // `hidKeysDown` returns information about which buttons have been just pressed (and they weren't in the previous frame)
            hidKeysDown(HidControllerID::CONTROLLER_P1_AUTO) as u32
        };

        // Break the main loop if the last pressed key was (+)
        if let HidControllerKeys::KEY_PLUS = HidControllerKeys(key) {
            break;
        }

        // Print console output
        unsafe { consoleUpdate(ptr::null_mut()) };
    }

    // Exit console and return to `hbmenu` in an orderly fashion
    unsafe { consoleExit(ptr::null_mut()) };

    Ok(())
}
