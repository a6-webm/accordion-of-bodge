// enum of Keyboard keys
// f(key) -> GlovePIE keycode

// parse CSV of notes/chords mapped to keyboard(||stradella bass system?)

use std::collections::HashMap;
use std::mem::size_of;
use std::ptr::{null_mut, null};
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs, thread};
use regex::Regex;
use winapi::shared::minwindef::{UINT, LRESULT, WPARAM, LPARAM};
use winapi::shared::windef::{HWND};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::processthreadsapi::{GetStartupInfoW, STARTUPINFOA, STARTUPINFOW};
use winapi::um::winuser::{WNDCLASSEXW, RegisterClassExW, CreateWindowExW, WS_OVERLAPPEDWINDOW, CW_USEDEFAULT, GetMessageW, DispatchMessageW, MSG, ShowWindow, WS_VISIBLE, CS_HREDRAW, CS_VREDRAW, PostQuitMessage, RAWINPUTDEVICE, RIDEV_NOLEGACY, RegisterRawInputDevices};

mod lib;
use crate::lib::{Chord, KeyCode, MidiNote};

struct CsvParser {
    regex: Regex,
}

impl CsvParser {
    fn new() -> CsvParser {
        CsvParser {
            regex: Regex::new(r#"(?:(?:"(.*?)")|(.*?))(?:(?:,\r?\n)|,|(?:\r?\n)|$)"#).unwrap(),
        }
    }

    fn cells_as_vec(&self, s: &str) -> Vec<String>{
        let mut out: Vec<String> = Vec::new();
        for caps in self.regex.captures_iter(s) {
            for m in caps.iter().skip(1).flatten() {
                out.push(m.as_str().to_owned());
            }
        }
        out
    }
}

fn main() {
    // let csv_parser = CsvParser::new();
    // let args: Vec<String> = env::args().collect(); // TODO error if file does not end with .csv

    // let keymap_fp = args.get(1).expect("Correct usage: "); //TODO add correct usage text
    // let key_aliases_fp = args.get(2).expect("Correct usage: "); //TODO allow ommission of 2nd parameter

    // let keymap_string = fs::read_to_string(keymap_fp).expect("Failed to read file: ");
    // let key_aliases_string = fs::read_to_string(key_aliases_fp).expect("Failed to read file: ");

    // let keymap_csv = csv_parser.cells_as_vec(keymap_string.as_str());
    // let keyaliases_csv = csv_parser.cells_as_vec(key_aliases_string.as_str());

    // let mut key_aliases: HashMap<String, KeyCode> = HashMap::new();
    // let mut key_map: HashMap<KeyCode, Vec<MidiNote>> = HashMap::new();

    // // Populate key_aliases
    // for s in keyaliases_csv.iter() {
    //     if s.trim().is_empty() { // Ignore strings of whitespace
    //         continue;
    //     }
    //     let mut iter = s.splitn(2, '=');
    //     let alias = iter.next().expect("Missing alias and key data in alias CSV file").trim();
    //     let gp_key = iter.next().expect("Missing alias or key data in alias CSV file").trim();
    //     key_aliases.insert(alias.to_owned(), gp_key.to_owned());
    // }

    // // Populate key_map
    // for s in keymap_csv.iter() {
    //     if s.trim().is_empty() { // Ignore strings of whitespace
    //         continue;
    //     }
    //     let mut iter = s.splitn(3, |c| c == '=' || c == '.');
    //     let chord_str = iter.next().expect("Wrong syntax in keymap CSV file").trim();
    //     let alias = iter.next().expect("Wrong syntax in keymap CSV file").trim();
    //     let vel: u8 = iter.next().expect("Wrong syntax in keymap CSV file").trim()
    //         .parse().expect("Wrong syntax in keymap CSV file");
        
    //     let chord = Chord::new(chord_str).expect("Wrong syntax in keymap CSV file")
    //         .to_midi_chord(vel).expect("Error creating chord");
    //     let key = match key_aliases.get(alias) {
    //         Some(s) => s,
    //         None => {
    //             println!("\"{}\" not in aliases.csv. Using alias, but it may not be a valid GlovePIE key", alias);
    //             alias
    //         },
    //     };
    //     key_map.insert(key.to_owned(), chord);
    // }

    // println!("key_map: {:?}", key_map);

    let rid_tlc = RAWINPUTDEVICE {
        usUsagePage: 1, // HID_USAGE_PAGE_GENERIC
        usUsage: 6, // HID_USAGE_GENERIC_KEYBOARD
        dwFlags: RIDEV_NOLEGACY, // adds keyboard and also ignores legacy keyboard messages // TODO might not want this to ignore legacy
        hwndTarget: null_mut(), // follows keyboard focus
    };

    unsafe { 
        if RegisterRawInputDevices(&rid_tlc, 1, size_of::<RAWINPUTDEVICE>() as UINT) == 0 {
            panic!("failed to register raw input TLC");
        }
    }

    let h_instance = unsafe { GetModuleHandleW(null_mut()) };

    let mut startup_info: STARTUPINFOW;
    unsafe {
        startup_info = std::mem::zeroed();
        GetStartupInfoW(&mut startup_info);
    };

    let class_name = win32_string("my_first_window");

    let wc = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        style: 0,
        lpfnWndProc: Some(wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: h_instance,
        hIcon: null_mut(),
        hCursor: null_mut(),
        hbrBackground: null_mut(),
        lpszMenuName: null_mut(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: null_mut(),
    };

    unsafe { RegisterClassExW(&wc); }

    let hwnd: HWND = unsafe { CreateWindowExW(
            0,
            class_name.as_ptr(),
            win32_string("the bruh window").as_ptr(),
            WS_VISIBLE | WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            null_mut(),
            null_mut(),
            h_instance,
            null_mut(),
    )};
    if hwnd.is_null() { panic!("failed to create window"); }

    unsafe {
        let mut lp_msg: MSG = std::mem::zeroed();
        println!("Thread started");
        while dbg!(GetMessageW(&mut lp_msg, 0 as HWND, 0, 0) > 0) {
            dbg!(DispatchMessageW(&lp_msg));
        }
    }

}

#[cfg(windows)]
unsafe extern "system" fn wnd_proc(h_wnd: HWND, i_message: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    use winapi::{um::winuser::{DefWindowProcW, WM_DESTROY, WM_INPUT, RAWINPUT, GetRawInputData, HRAWINPUT, RID_INPUT, RAWINPUTHEADER, RIM_TYPEKEYBOARD}, shared::{minwindef::LPVOID, winerror::FAILED}};

    match dbg!(i_message) {
        WM_DESTROY => {
            dbg!(PostQuitMessage(0));
            return 0;
        },
        WM_INPUT => {
            let mut rid_size: UINT = 0;
            GetRawInputData(l_param as HRAWINPUT, RID_INPUT, null_mut(), &mut rid_size, size_of::<RAWINPUTHEADER>() as UINT);
            if rid_size == 0 { return 0; } // not sure if this can happen, but microsoft docs do this
            let mut raw_data_buffer: Vec<u8> = Vec::with_capacity(rid_size.try_into().unwrap());

            if rid_size != 
                GetRawInputData(l_param as HRAWINPUT, RID_INPUT, raw_data_buffer.as_mut_ptr() as LPVOID, &mut rid_size, size_of::<RAWINPUTHEADER>() as UINT) {
                    println!("GetRawInputData does not return correct size!");
            }

            let raw: &RAWINPUT = &*(raw_data_buffer.as_ptr() as *const RAWINPUT);

            if (raw.header.dwType == RIM_TYPEKEYBOARD) {
                dbg!(raw.data.keyboard().MakeCode);
            }

            return 0;
        },
        _ => DefWindowProcW(h_wnd, i_message, w_param, l_param),
    }
}

#[cfg(windows)]
fn win32_string( value : &str ) -> Vec<u16> {
    use std::{ffi::OsStr, os::windows::prelude::OsStrExt, iter::once};

    OsStr::new( value ).encode_wide().chain( once( 0 ) ).collect()
}