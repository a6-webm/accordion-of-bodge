#![allow(clippy::missing_safety_doc, clippy::needless_return)]

use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsString;
use std::io::{stdout, stdin, Write};
use std::mem::size_of;
use std::os::windows::prelude::OsStringExt;
use std::ptr::null_mut;
use std::time::Instant;
use std::{env, fs};
use chord_parser::{MidiNote, Chord, CsvParser};
use midir::{MidiOutputConnection, MidiOutput, MidiOutputPort};
use winapi::shared::minwindef::{UINT, LRESULT, WPARAM, LPARAM, HLOCAL, USHORT, LPVOID};
use winapi::shared::windef::HWND;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winbase::{FormatMessageW, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_IGNORE_INSERTS, LocalFree};
use winapi::um::winnt::{MAKELANGID, LANG_NEUTRAL, SUBLANG_DEFAULT, LPWSTR, HANDLE};
use winapi::um::winuser::{WNDCLASSEXW, RegisterClassExW, CreateWindowExW, WS_OVERLAPPEDWINDOW, CW_USEDEFAULT, GetMessageW, DispatchMessageW, MSG, WS_VISIBLE, PostQuitMessage, RAWINPUTDEVICE, RIDEV_NOLEGACY, RegisterRawInputDevices, RAWINPUT, WM_KEYDOWN, WM_SYSKEYDOWN, WM_KEYUP, WM_SYSKEYUP, WM_INPUT, HRAWINPUT, RID_INPUT, RAWINPUTHEADER, GetRawInputData, RIM_TYPEKEYBOARD, RIDEV_NOHOTKEYS, WM_DESTROY, DefWindowProcW, DestroyWindow, WM_USER, PostMessageA, UnregisterClassW};

type Keymap = HashMap<(HANDLE, USHORT), Vec<MidiNote>>;

static mut GLB: Globals = Globals {
    keymap: null_mut(),
    keymap_builder: null_mut(),
    midi_handler: null_mut(),
};

struct Globals {
    keymap: *mut Keymap,
    keymap_builder: *mut KeymapBuilder,
    midi_handler: *mut MidiHandler,
}

struct MidiHandler {
    note_states: Vec<u8>,
    key_states: HashMap<(HANDLE, USHORT), KDir>,
    conn_out: MidiOutputConnection,
}

impl MidiHandler {
    fn new() -> Result<Self, Box<dyn Error>> {
        let midi_out = MidiOutput::new("Accordion_of_Bodge_midi_out")?;
        let out_ports = midi_out.ports();
        let out_port: &MidiOutputPort = match out_ports.len() {
            0 => return Err("no output port found".into()),
            1 => {
                println!("Choosing the only available output port: {}", midi_out.port_name(&out_ports[0]).unwrap());
                &out_ports[0]
            },
            _ => {
                println!("\nAvailable output ports:");
                for (i, p) in out_ports.iter().enumerate() {
                    println!("{}: {}", i, midi_out.port_name(p).unwrap());
                }
                print!("Please select output midi port: ");
                stdout().flush()?;
                let mut input = String::new();
                stdin().read_line(&mut input)?;
                out_ports.get(input.trim().parse::<usize>()?)
                         .ok_or("invalid output port selected")?
            }
        };
        println!("\nOpening connection");
        let conn_out = midi_out.connect(out_port, "midir-test")?;
        println!("Connection open. Listen!");
        return Ok(MidiHandler { note_states: vec!(0; 256), key_states: HashMap::new(), conn_out });
    }

    fn initialise_key_states(&mut self, keymap: &Keymap) {
        for (key, _) in keymap.iter() {
            self.key_states.insert(key.to_owned(), KDir::Up);
        }
    }

    fn process_msg(&mut self, r: RawKRecord, notes: &Vec<MidiNote>) {
        const NOTE_ON_MSG: u8 = 0x90;
        const NOTE_OFF_MSG: u8 = 0x80;
        match self.key_states.get(&(r.h_dev, r.v_k_code)) {
            None => return, // key not in keymap
            Some(k_dir) => {
                if *k_dir == r.k_dir {
                    return; // return if key hasn't changed state
                }
            },
        }
        self.key_states.insert((r.h_dev, r.v_k_code), r.k_dir.to_owned());
        
        for mn in notes {
            match r.k_dir {
                KDir::Up => {
                    if self.note_states[mn.n as usize] > 0 {
                        self.note_states[mn.n as usize] -= 1;
                    }
                    if self.note_states[mn.n as usize] == 0 {
                        println!("note: {} off", mn.n);
                        let _ = self.conn_out.send(&[NOTE_OFF_MSG, mn.n, mn.vel]);
                    }
                },
                KDir::Down => {
                    if self.note_states[mn.n as usize] == 0 {
                        println!("note: {} on", mn.n);
                        let _ = self.conn_out.send(&[NOTE_ON_MSG, mn.n, mn.vel]);
                    }
                    self.note_states[mn.n as usize] += 1;
                },
            }
        }
    }
}

struct KeymapBuilder {
    keymap: Keymap,
    files: Vec<String>,
    devs: Vec<HANDLE>,
}

impl KeymapBuilder {
    fn new() -> Self {
        KeymapBuilder { keymap: HashMap::new(), devs: Vec::new(), files: Vec::new()}
    }

    fn push_km(&mut self, dev_and_key: (HANDLE, USHORT), notes: Vec<MidiNote>) {
        self.keymap.insert(dev_and_key, notes);
    }

    fn push_d(&mut self, dev: HANDLE) {
        self.devs.push(dev);
    }

    fn push_f(&mut self, fl: String) {
        self.files.push(fl);
    }

    fn is_filled(&self) -> bool {
        self.devs.len() == self.files.len()
    }

    fn build(&self) -> Keymap {
        let mut out = Keymap::new();
        for ((h, v_k), mn) in &self.keymap {
            out.insert((self.devs[h.to_owned() as usize], v_k.to_owned()), mn.to_owned());
        }
        return out;
    }
}

#[derive(Debug, Clone, PartialEq)]
enum KDir {
    Up,
    Down,
}

#[derive(Debug, Clone)]
struct RawKRecord {
    k_dir: KDir,
    h_dev: HANDLE,
    v_k_code: USHORT,
}

impl RawKRecord {
    fn new(l_param: LPARAM) -> Option<RawKRecord> {
        let mut rid_size: UINT = 0;
        unsafe { GetRawInputData(l_param as HRAWINPUT, RID_INPUT, null_mut(), &mut rid_size, size_of::<RAWINPUTHEADER>() as UINT); }
        if rid_size == 0 { return None; } // not sure if this can happen, but microsoft documentation does this
        let mut raw_data_buffer: Vec<u8> = Vec::with_capacity(rid_size.try_into().unwrap());

        unsafe {
            if rid_size != GetRawInputData(
                l_param as HRAWINPUT,
                RID_INPUT,
                raw_data_buffer.as_mut_ptr() as LPVOID,
                &mut rid_size,
                size_of::<RAWINPUTHEADER>() as UINT
            ) { println!("GetRawInputData does not return correct size!"); }
        }
        
        let raw: &RAWINPUT = unsafe { &*(raw_data_buffer.as_ptr() as *const RAWINPUT) };

        if raw.header.dwType != RIM_TYPEKEYBOARD {
            return None;
        }
        let rawd_k = unsafe{ raw.data.keyboard() };

        let k_dir = match rawd_k.Message {
            WM_KEYDOWN | WM_SYSKEYDOWN => KDir::Down,
            WM_KEYUP | WM_SYSKEYUP => KDir::Up,
            _ => unreachable!("recieved non key message from rawinput"),
        };
        let h_dev = raw.header.hDevice;
        let v_k_code = rawd_k.VKey;
        return Some(RawKRecord {k_dir, h_dev, v_k_code});
    }
}

fn win32_string( value : &str ) -> Vec<u16> {
    use std::{ffi::OsStr, os::windows::prelude::OsStrExt, iter::once};
    OsStr::new( value ).encode_wide().chain( once( 0 ) ).collect()
}

fn make_window(name: &str, class: &str, wnd_proc: unsafe extern "system" fn (HWND, UINT, WPARAM, LPARAM) -> LRESULT, h_instance: *mut winapi::shared::minwindef::HINSTANCE__) -> HWND {
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
        lpszClassName: win32_string(class).as_ptr(),
        hIconSm: null_mut(),
    };

    unsafe { RegisterClassExW(&wc); }

    let hwnd: HWND = unsafe { CreateWindowExW(
            0,
            win32_string(class).as_ptr(),
            win32_string(name).as_ptr(),
            WS_VISIBLE | WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            600,
            600,
            null_mut(),
            null_mut(),
            h_instance,
            null_mut(),
    )};
    if hwnd.is_null() { panic!("failed to create window"); }

    let rid_tlc = RAWINPUTDEVICE {
        usUsagePage: 1, // HID_USAGE_PAGE_GENERIC
        usUsage: 6, // HID_USAGE_GENERIC_KEYBOARD
        dwFlags: RIDEV_NOLEGACY | RIDEV_NOHOTKEYS, // ignores legacy keyboard messages and prevents hotkeys from triggering // TODO check if RIDEV_NOHOTKEYS actually does this lmao
        hwndTarget: hwnd,
    };

    unsafe {
        if RegisterRawInputDevices(&rid_tlc, 1, size_of::<RAWINPUTDEVICE>() as UINT) == 0 {
            panic!("failed to register raw input TLC");
        }
    }

    return hwnd;
}

fn print_last_win_error() {
    unsafe {
        let message_buffer: LPWSTR = null_mut();
        let error_message_id = GetLastError();
        let size = FormatMessageW(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
            null_mut(), error_message_id, MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT).into(), &message_buffer as *const LPWSTR as LPWSTR, 0, null_mut());
        let str = OsString::from_wide(std::slice::from_raw_parts(message_buffer, size as usize));
        println!("GetLastError: {:?}", str);
        LocalFree(message_buffer as HLOCAL);
    }
}

unsafe extern "system" fn init_devices_wnd_proc(h_wnd: HWND, i_message: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    match i_message {
        WM_INPUT => {
            let r = match RawKRecord::new(l_param) {
                Some(rec) => rec,
                None => return 0,
            };
            if r.k_dir != KDir::Down { return 0;}
            if (*GLB.keymap_builder).is_filled() { return 0; }
            (*GLB.keymap_builder).push_d(r.h_dev);
            if (*GLB.keymap_builder).is_filled() {
                PostMessageA(null_mut(), WM_USER + 1, 0, 0);
                DestroyWindow(h_wnd);
                return 0;
            }
            let dev_i = (*GLB.keymap_builder).devs.len();
            println!("-------- Press any key on the device that should use {} for mappings --------", (*GLB.keymap_builder).files[dev_i]);
            return 0;
        },
        _ => return DefWindowProcW(h_wnd, i_message, w_param, l_param),
    }
}

unsafe extern "system" fn play_wnd_proc(h_wnd: HWND, i_message: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    match i_message {
        WM_DESTROY => {
            PostQuitMessage(0);
            return 0;
        },
        WM_INPUT => {
            let timer = Instant::now();
            let r = match RawKRecord::new(l_param) {
                Some(r) => r,
                None => return 0,
            };
            let notes = match (*GLB.keymap).get(&(r.h_dev, r.v_k_code)) {
                Some(notes) => notes,
                None => return 0,
            };
            println!("Playing chord: {:?}", notes);
            (*GLB.midi_handler).process_msg(r, notes);
            println!("Sent midi msg, took {:?} from input to send", timer.elapsed());
            return 0;
        },
        _ => DefWindowProcW(h_wnd, i_message, w_param, l_param),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    for arg in &args[1..] {
        if arg[arg.len()-4..] != *".csv" { panic!("file '{}' does not end with .csv", arg) }
    }
    let h_instance = unsafe { GetModuleHandleW(null_mut()) };

    // Init globals
    let mut keymap_builder  = KeymapBuilder::new();
    unsafe { GLB.keymap_builder = &mut keymap_builder; }
    let mut midi_handler = MidiHandler::new().expect("error creating MidiHandler");
    unsafe{ GLB.midi_handler = &mut midi_handler; }
    
    // Populate key_aliases
    let csv_parser = CsvParser::new();
    let mut key_aliases: HashMap<String, USHORT> = HashMap::new();
    {
        let key_aliases_fp = args.get(1).expect("Correct usage: "); // TODO finish Correct usage
        let key_aliases_string = fs::read_to_string(key_aliases_fp).expect("Failed to read file: ");
        let keyaliases_csv = csv_parser.cells_as_vec(key_aliases_string.as_str());
        let mut parse_alias = true; // alternates between parsing an alias and a key code
        let mut alias = "";
        for s in keyaliases_csv.iter() {
            if s.trim().is_empty() { // Ignore strings of whitespace
                continue;
            }
            if parse_alias {
                alias = s.trim();
            } else {
                let gp_key: USHORT = s.trim().parse().expect("Key code not a valid number in alias CSV file");
                key_aliases.insert(alias.to_owned(), gp_key.to_owned());
            }
            parse_alias = !parse_alias;
        }
        if !parse_alias {
            panic!("Alias '{}' has no key code in alias CSV file '{}'", alias, key_aliases_fp);
        }
    }

    // Populate key_map
    let keymap_files: &[String] = &args[2..];
    if keymap_files.is_empty() { panic!("Correct usage: ") } // TODO finish Correct usage
    for (i, keymap_fp) in keymap_files.iter().enumerate() {
        unsafe { (*GLB.keymap_builder).push_f(keymap_fp.to_owned()); }
        
        let keymap_string = fs::read_to_string(keymap_fp).expect("Failed to read file: ");
        let keymap_csv = csv_parser.cells_as_vec(keymap_string.as_str());

        enum Parse {
            Chord,
            Key,
            Vel,
        }
        let mut loop_state = Parse::Chord;
        let mut chord_str = "";
        let mut key: USHORT = 0;
        for s in keymap_csv.iter() {
            if s.trim().is_empty() { // Ignore strings of whitespace
                continue;
            }
            match loop_state {
                Parse::Chord => {
                    chord_str = s.trim();
                    loop_state = Parse::Key;
                },
                Parse::Key => {
                    let alias = s.trim();
                    key = match key_aliases.get(alias) {
                        Some(s) => *s,
                        None => alias.parse().unwrap_or_else(|_| panic!("alias '{}' not in aliases.csv and not a number\neither add to aliases.csv or use a correctly formatted number", alias)),
                    };
                    loop_state = Parse::Vel;
                },
                Parse::Vel => {
                    let vel: u8 = s.trim()
                        .parse().unwrap_or_else(|_| panic!("{} is not a valid velocity in {}", s, keymap_fp));

                    let chord = Chord::new(chord_str).expect("Wrong syntax in keymap CSV file")
                        .to_midi_chord(vel).expect("Error creating chord");
                    unsafe { (*GLB.keymap_builder).push_km((i as HANDLE, key.to_owned()), chord); };
                    loop_state = Parse::Chord;
                },
            }
        }
        match loop_state {
            Parse::Chord => (),
            Parse::Key => panic!("Chord '{}' has no key specified after it in {}", chord_str, keymap_fp),
            Parse::Vel => panic!("no velocity specified after '{},{}' in {}", chord_str, key, keymap_fp),
        }
    }

    let hwnd = make_window("Accordion of Bodge: Set devices", "AoB: Set devs", init_devices_wnd_proc, h_instance);

    unsafe {
        let mut lp_msg: MSG = std::mem::zeroed();
        println!("-------- Tab into the window and press any key on the device that should use {} for mappings --------", (*GLB.keymap_builder).files[0]);
        while GetMessageW(&mut lp_msg, hwnd, 0, 0) > 0 {
            if lp_msg.message == WM_USER + 1 { break; }
            DispatchMessageW(&lp_msg);
        }
    }

    unsafe { UnregisterClassW(win32_string("AoB: Set devs").as_ptr(), null_mut()); }

    if unsafe { !(*GLB.keymap_builder).is_filled() } {
        return; // if true, "Set devices" window closed prematurely
    }

    let mut keymap: Keymap = unsafe { (*GLB.keymap_builder).build() };
    unsafe { GLB.keymap = &mut keymap; }
    unsafe { (*GLB.midi_handler).initialise_key_states(&*GLB.keymap); }

    let hwnd = make_window("Accordion of Bodge", "AoB: Play notes", play_wnd_proc, h_instance);

    println!("All devices set! Tab into window and press keys to play");

    unsafe {
        let mut lp_msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut lp_msg, hwnd, 0, 0) > 0 {
            DispatchMessageW(&lp_msg);
        }
    }
}