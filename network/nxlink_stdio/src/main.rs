use libc;
use libnx_rs::{libnx::*, LibnxError};
use std::{mem, net::Ipv4Addr, ptr, result::Result, slice, str};

fn main() -> Result<(), LibnxError> {
    unsafe {
        // Initialize console interface
        consoleInit(ptr::null_mut());
        // Initialize sockets system using default configuration
        socketInitialize(socketGetDefaultInitConfig());
    };

    println!("Press (+) at any time to exit the demo\n");

    // Get arguments sent from `nxlink` and put them in a rusty vector
    let argv: Vec<&str> = unsafe {
        let argv_ptr: *mut lang_items::c_void = envGetArgv();
        let argv_slice = slice::from_raw_parts(argv_ptr as *mut u8, 512);

        str::from_utf8(argv_slice)
            .unwrap()
            .trim_matches('\0')
            .split(|c| c == ' ' || c == '"')
            .filter(|c| !c.is_empty())
            .collect()
    };

    // Display arguments sent from `nxlink`
    println!("{} arguments:", argv.len());
    for (i, arg) in argv.iter().enumerate() {
        println!("\targv[{}]: `{}`", i, arg);
    }

    unsafe {
        // Get and print the IP address of the host  where `nxlink` was launched
        let libc_address: libc::in_addr = mem::transmute_copy(&__nxlink_host);
        let ipv4_address = Ipv4Addr::from(libc_address.s_addr.swap_bytes());
        println!("\nnxlink host is {:?}\n", ipv4_address);
        // Redirect `stdout` & `stderr` over network to nxlink
        nxlinkStdio();
    }

    // This text should display on nxlink host
    println!("`println!` output now goes to `nxlink` server (you should not be seeing this line on your Switch)");
    println!("Press (+) at any time to exit the demo");
    println!("Press A or B in your console to see the event printed to `nxlink`");

    while unsafe { appletMainLoop() } {
        // Get the identifier of the last pressed key
        let key = unsafe {
            //Scan all the inputs. This should be done once for each frame
            hidScanInput();

            //hidKeysDown returns information about which buttons have been just pressed (and they weren't in the previous frame)
            hidKeysDown(HidControllerID::CONTROLLER_P1_AUTO) as u32
        };

        // Break the main loop if the last pressed key was (+)
        match HidControllerKeys(key) {
            HidControllerKeys::KEY_PLUS => break,
            HidControllerKeys::KEY_A => println!("A pressed!"),
            HidControllerKeys::KEY_B => println!("B pressed!"),
            _ => (),
        }

        // Print console output
        unsafe {
            consoleUpdate(ptr::null_mut());
        }
    }

    // Stop sockets, exit console and return to `hbmenu` in an orderly fashion
    unsafe {
        socketExit();
        consoleExit(ptr::null_mut());
    }

    Ok(())
}
