// enum of Keyboard keys
// f(key) -> GlovePIE keycode

// parse CSV of notes/chords mapped to keyboard(||stradella bass system?)

use std::collections::HashMap;
use std::ffi::c_uint;
use std::mem::size_of;
use std::{env, fs};
use regex::Regex;
use winapi::shared::basetsd::PUINT32;
use winapi::shared::minwindef::UINT;
use winapi::shared::ntdef::NULL;
use winapi::shared::winerror::ERROR_INSUFFICIENT_BUFFER;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winuser::{GetRawInputDeviceList, PRAWINPUTDEVICELIST, RAWINPUTDEVICE, RAWINPUTDEVICELIST};
use winapi::vc::limits::UINT_MAX;
use crate::lib::{Chord, KeyCode, MidiNote};

mod lib;



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
    let csv_parser = CsvParser::new();
    let args: Vec<String> = env::args().collect(); // TODO error if file does not end with .csv

    let keymap_fp = args.get(1).expect("Correct usage: "); //TODO add correct usage text
    let key_aliases_fp = args.get(2).expect("Correct usage: "); //TODO allow ommission of 2nd parameter

    let keymap_string = fs::read_to_string(keymap_fp).expect("Failed to read file: ");
    let key_aliases_string = fs::read_to_string(key_aliases_fp).expect("Failed to read file: ");

    let keymap_csv = csv_parser.cells_as_vec(keymap_string.as_str());
    let keyaliases_csv = csv_parser.cells_as_vec(key_aliases_string.as_str());

    let mut key_aliases: HashMap<String, KeyCode> = HashMap::new();
    let mut key_map: HashMap<KeyCode, Vec<MidiNote>> = HashMap::new();

    // Populate key_aliases
    for s in keyaliases_csv.iter() {
        if s.trim().is_empty() { // Ignore strings of whitespace
            continue;
        }
        let mut iter = s.splitn(2, '=');
        let alias = iter.next().expect("Missing alias and key data in alias CSV file").trim();
        let gp_key = iter.next().expect("Missing alias or key data in alias CSV file").trim();
        key_aliases.insert(alias.to_owned(), gp_key.to_owned());
    }

    // Populate key_map
    for s in keymap_csv.iter() {
        if s.trim().is_empty() { // Ignore strings of whitespace
            continue;
        }
        let mut iter = s.splitn(3, |c| c == '=' || c == '.');
        let chord_str = iter.next().expect("Wrong syntax in keymap CSV file").trim();
        let alias = iter.next().expect("Wrong syntax in keymap CSV file").trim();
        let vel: u8 = iter.next().expect("Wrong syntax in keymap CSV file").trim()
            .parse().expect("Wrong syntax in keymap CSV file");
        
        let chord = Chord::new(chord_str).expect("Wrong syntax in keymap CSV file")
            .to_midi_chord(vel).expect("Error creating chord");
        let key = match key_aliases.get(alias) {
            Some(s) => s,
            None => {
                println!("\"{}\" not in aliases.csv. Using alias, but it may not be a valid GlovePIE key", alias);
                alias
            },
        };
        key_map.insert(key.to_owned(), chord);
    }

    println!("key_map: {:?}", key_map);

    let mut rid_list: Vec<RAWINPUTDEVICELIST>;
    loop {
        let mut num_dev: UINT = 0;
        unsafe {
            let err = GetRawInputDeviceList(NULL as PRAWINPUTDEVICELIST, &mut num_dev, size_of::<RAWINPUTDEVICELIST>() as UINT);
            if err != 0 { panic!("idk windows or something"); }
        }
        rid_list = Vec::with_capacity(num_dev as usize);
        if num_dev == 0 { return; }
        unsafe {
            num_dev = GetRawInputDeviceList(rid_list.as_mut_ptr(), &mut num_dev, size_of::<RAWINPUTDEVICELIST>() as UINT);
            if num_dev == UINT_MAX {
                if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
                    panic!("idk windows or something");
                }
                continue; // Devices were added since last check, rerun
            }
            rid_list.set_len(num_dev as usize);
        }
        break;
    }
    
    for (i, ridl) in rid_list.iter().enumerate() {
        println!("{i}: {}", ridl.dwType);
    }

    



    // let rid: RAWINPUTDEVICE = RAWINPUTDEVICE { 
    //     usUsagePage: 1, // HID_USAGE_PAGE_GENERIC
    //     usUsage: 2, // HID_USAGE_GENERIC_KEYBOARD
    //     dwFlags: RIDEV_NOLEGACY, // adds keyboard and also ignores legacy keyboard messages
    //     hwndTarget: HWND_DESKTOP, // no target window
    // };

    // unsafe {
    //     RegisterRawInputDevices(&rid, uiNumDevices, cbSize);
    // }

}
