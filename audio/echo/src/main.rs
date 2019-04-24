use libc::memalign;
use libnx_rs::{libnx::*, LibnxError};
use std::{self, ptr, result::Result};

// Example for audio capture and playback
// This example continuously records audio data from the default input device (see libnx audin.h)
// and sends it to the default audio output device (see libnx audout.h)

const SAMPLERATE: u32 = 48000;
const CHANNELCOUNT: u32 = 2;
const FRAMERATE: u32 = 1000 / 30;
const SAMPLECOUNT: u32 = SAMPLERATE / FRAMERATE;
const BYTESPERSAMPLE: u32 = 2;

fn main() -> Result<(), LibnxError> {
    // Initialize console interface
    // Using NULL as argument tells the console library to use the internal console structure as current one
    unsafe { consoleInit(ptr::null_mut()) };

    // Declare buffers for audio IO
    let mut audio_in_buf: AudioInBuffer;
    let mut audio_out_buf: AudioOutBuffer;

    // Prepare pointers and counters for released buffers
    let released_in_buffer: *mut *mut AudioInBuffer = &mut ptr::null_mut();
    let released_out_buffer: *mut *mut AudioOutBuffer = &mut ptr::null_mut();
    let released_in_count: *mut u32 = &mut 0;
    let released_out_count: *mut u32 = &mut 0;

    // Make sure the sample buffer size is aligned to 0x1000 bytes
    let data_size: u64 = u64::from(SAMPLECOUNT * CHANNELCOUNT * BYTESPERSAMPLE);
    let buffer_size: u64 = (data_size + 0xfff) & !0xfff;

    // Allocate the buffers
    let (in_buff_data, out_buff_data) = unsafe {
        (
            memalign(0x1000, buffer_size as usize) as *mut lang_items::c_void,
            memalign(0x1000, buffer_size as usize) as *mut lang_items::c_void,
        )
    };

    // Initialize the default audio input device
    let mut rc = unsafe { audinInitialize() };
    println!("audinInitialize() returned 0x{}", rc);

    // Initialize the default audio output device
    rc = unsafe { audoutInitialize() };
    println!("audioutnitialize() returned 0x{}", rc);

    // Start audio capture
    rc = unsafe { audinStartAudioIn() };
    println!("audinStartAudioIn() returned 0x{}", rc);

    // Start audio playback
    rc = unsafe { audoutStartAudioOut() };
    println!("audoutStartAudioOut() returned 0x{}", rc);

    // Prepare the input buffer
    audio_in_buf = AudioInBuffer {
        buffer: in_buff_data,
        buffer_size,
        data_offset: 0,
        data_size,
        next: ptr::null_mut(),
    };

    // Prepare the output buffer
    audio_out_buf = AudioOutBuffer {
        buffer: out_buff_data,
        buffer_size,
        data_offset: 0,
        data_size,
        next: ptr::null_mut(),
    };

    // Append the initial input buffer
    rc = unsafe { audinAppendAudioInBuffer(&mut audio_in_buf) };
    println!("audinAppendAudioInBuffer() returned 0x{}", rc);

    // Append the initial output buffer
    rc = unsafe { audoutAppendAudioOutBuffer(&mut audio_out_buf) };
    println!("audoutAppendAudioOutBuffer() returned 0x{}", rc);

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

        unsafe {
            // Wait for audio capture and playback to finish.
            audinWaitCaptureFinish(released_in_buffer, released_in_count, std::u64::MAX);
            audoutWaitPlayFinish(released_out_buffer, released_out_count, std::u64::MAX);

            // Copy the captured audio data into the playback buffer.
            ptr::copy_nonoverlapping(
                (*(*released_in_buffer)).buffer,
                (*(*released_out_buffer)).buffer,
                (*(*released_in_buffer)).data_size as usize,
            );

            // Append the released buffers again.
            audinAppendAudioInBuffer(*released_in_buffer);
            audoutAppendAudioOutBuffer(*released_out_buffer);

            // Print console output
            consoleUpdate(ptr::null_mut());
        }
    }

    // Exit console and return to `hbmenu` in an orderly fashion
    unsafe { consoleExit(ptr::null_mut()) };

    Ok(())
}
