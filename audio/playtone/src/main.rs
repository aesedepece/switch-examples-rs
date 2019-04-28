#![allow(clippy::cast_ptr_alignment)]

use libc::{memalign, memset};
use libnx_rs::{libnx::*, LibnxError};
use std::{self, ptr, result::Result};

const SAMPLERATE: u32 = 48000;
const CHANNELCOUNT: u32 = 2;
const FRAMERATE: u32 = 1000 / 30;
const SAMPLECOUNT: u32 = SAMPLERATE / FRAMERATE;
const BYTESPERSAMPLE: u32 = 2;

const NOTES_FREQUENCIES: [u32; 12] = [
    220, 440, 880, 1760, 3520, 7040, 14080, 7040, 3520, 1760, 880, 440,
];

/// Fill an audio buffer with a sine waveform of chosen frequency.
fn fill_audio_buffer(
    audio_buffer: *mut lang_items::c_void,
    offset: usize,
    size: usize,
    frequency: u32,
) {
    if audio_buffer.is_null() {
        return;
    }

    let dest: &mut [u32] =
        unsafe { core::slice::from_raw_parts_mut(audio_buffer as *mut u32, size) };

    for (i, stereo_sample) in dest.iter_mut().enumerate().take(size) {
        // This is a simple sine wave, with a frequency of `frequency` Hz, and an amplitude 30% of maximum.
        let sample: i32 = (0.3
            * 32767.0
            * (frequency as f32 * (2f32 * std::f32::consts::PI) * (offset + i) as f32
                / SAMPLERATE as f32)
                .sin()) as i32;

        // Stereo samples are interleaved: left and right channels.
        *stereo_sample = ((sample << 16) | (sample & 0xffff)) as u32;
    }
}

/// Convert LibNX `rc` codes into rusty `Result`s.
fn result_from_rc(rc: u32) -> Result<u32, LibnxError> {
    if rc == 0 {
        Ok(rc)
    } else {
        Err(LibnxError::from_raw(rc))
    }
}

fn main() -> Result<(), LibnxError> {
    let mut play_tone: Option<u32> = None;

    // Initialize console interface
    // Using NULL as argument tells the console library to use the internal console structure as current one
    unsafe { consoleInit(ptr::null_mut()) };

    // Declare buffers for audio output
    let mut audio_out_buf: AudioOutBuffer;
    let released_out_buffer: *mut *mut AudioOutBuffer = &mut ptr::null_mut();

    // Make sure the sample buffer size is aligned to 0x1000 bytes
    let data_size: u64 = u64::from(SAMPLECOUNT * CHANNELCOUNT * BYTESPERSAMPLE);
    let buffer_size: u64 = (data_size + 0xfff) & !0xfff;

    // Allocate the buffers
    let out_buff_data = unsafe {
        let out_buff_data = memalign(0x1000, buffer_size as usize);
        memset(out_buff_data, 0, buffer_size as usize);

        out_buff_data
    };

    // Initialize the default audio output device
    let initialize_result = result_from_rc(unsafe { audoutInitialize() });
    println!("audioutnitialize() returned {:?}", initialize_result);

    unsafe {
        println!("Sample rate: {} Hz", audoutGetSampleRate());
        println!("Channel count: {}", audoutGetChannelCount());
        println!("PCM format: {}", audoutGetPcmFormat());
        println!("Device state: {}", audoutGetDeviceState());
    }

    // Start audio playback
    let start_result = result_from_rc(unsafe { audoutStartAudioOut() });
    println!("audoutStartAudioOut() returned {:?}", start_result);

    println!("Press A, B, Y, X, Left, Up, Right, Down, L, R, ZL or ZR to play a different tone.");

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

        play_tone = match HidControllerKeys(key) {
            HidControllerKeys::KEY_A => Some(NOTES_FREQUENCIES[0]),
            HidControllerKeys::KEY_B => Some(NOTES_FREQUENCIES[1]),
            HidControllerKeys::KEY_Y => Some(NOTES_FREQUENCIES[2]),
            HidControllerKeys::KEY_X => Some(NOTES_FREQUENCIES[3]),
            HidControllerKeys::KEY_DLEFT => Some(NOTES_FREQUENCIES[4]),
            HidControllerKeys::KEY_DUP => Some(NOTES_FREQUENCIES[5]),
            HidControllerKeys::KEY_DRIGHT => Some(NOTES_FREQUENCIES[6]),
            HidControllerKeys::KEY_DDOWN => Some(NOTES_FREQUENCIES[7]),
            HidControllerKeys::KEY_L => Some(NOTES_FREQUENCIES[8]),
            HidControllerKeys::KEY_R => Some(NOTES_FREQUENCIES[9]),
            HidControllerKeys::KEY_ZL => Some(NOTES_FREQUENCIES[10]),
            HidControllerKeys::KEY_ZR => Some(NOTES_FREQUENCIES[11]),
            _ => play_tone,
        };

        play_tone = match play_tone {
            Some(frequency) => {
                fill_audio_buffer(
                    out_buff_data as *mut lang_items::c_void,
                    0,
                    data_size as usize,
                    frequency,
                );

                // Prepare the output buffer
                audio_out_buf = AudioOutBuffer {
                    buffer: out_buff_data as *mut lang_items::c_void,
                    buffer_size,
                    data_offset: 0,
                    data_size,
                    next: ptr::null_mut(),
                };

                // Play the buffer.
                let play_result = result_from_rc(unsafe {
                    audoutPlayBuffer(&mut audio_out_buf, released_out_buffer)
                });
                if play_result.is_err() {
                    println!("audoutPlayBuffer() returned {:?}", play_result)
                };

                None
            }
            None => None,
        };

        // Print console output
        unsafe {
            consoleUpdate(ptr::null_mut());
        }
    }

    unsafe {
        // Stop audio playback.
        let stop_result = result_from_rc(audoutStopAudioOut());
        println!("audoutStopAudioOut() returned {:?}", stop_result);
        // Terminate the default audio output device.
        audoutExit();
        // Exit console and return to `hbmenu` in an orderly fashion
        consoleExit(ptr::null_mut());
    }

    Ok(())
}
