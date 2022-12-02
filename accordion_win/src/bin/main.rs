#![allow(clippy::missing_safety_doc, clippy::needless_return)]

// enum of Keyboard keys
// f(key) -> GlovePIE keycode

// parse CSV of notes/chords mapped to keyboard(||stradella bass system?)

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
use winapi::um::processthreadsapi::{GetStartupInfoW, STARTUPINFOW};
use winapi::um::winbase::{FormatMessageW, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_IGNORE_INSERTS, LocalFree};
use winapi::um::winnt::{MAKELANGID, LANG_NEUTRAL, SUBLANG_DEFAULT, LPWSTR, HANDLE};
use winapi::um::winuser::{WNDCLASSEXW, RegisterClassExW, CreateWindowExW, WS_OVERLAPPEDWINDOW, CW_USEDEFAULT, GetMessageW, DispatchMessageW, MSG, WS_VISIBLE, PostQuitMessage, RAWINPUTDEVICE, RIDEV_NOLEGACY, RegisterRawInputDevices, RIDEV_INPUTSINK, RAWINPUT, WM_KEYDOWN, WM_SYSKEYDOWN, WM_KEYUP, WM_SYSKEYUP, WM_INPUT, HRAWINPUT, RID_INPUT, RAWINPUTHEADER, GetRawInputData, RIM_TYPEKEYBOARD};

type KeyMap = HashMap<(HANDLE, USHORT), Option<Vec<MidiNote>>>;

static mut GLB: Globals = Globals {
    verbose: false,
    raw_key_logs: null_mut(),
    key_map: null_mut(),
    dev_handles: null_mut(),
    midi_handler: null_mut(),
};

struct Globals {
    verbose: bool,
    raw_key_logs: *mut RawKeyLogs,
    key_map: *mut KeyMap,
    dev_handles: *mut DevHandles,
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
                print!("Please select output port: ");
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

    fn insert_key(&mut self, key: (HANDLE, USHORT)) {
        self.key_states.insert(key, KDir::Up);
    }

    fn process_msg(&mut self, r: RawKRecord, notes: &Vec<MidiNote>) {
        const NOTE_ON_MSG: u8 = 0x90;
        const NOTE_OFF_MSG: u8 = 0x80;
        if *self.key_states.get(&(r.h_dev, r.v_k_code)).unwrap() == r.k_dir {
            return; // return if key hasn't changed state
        }
        self.key_states.insert((r.h_dev, r.v_k_code), r.k_dir.to_owned());
        
        for mn in notes {
            match r.k_dir {
                KDir::Up => {
                    if self.note_states[mn.n as usize] > 0 {
                        self.note_states[mn.n as usize] -= 1;
                    }
                    if self.note_states[mn.n as usize] == 0 {
                        if unsafe {GLB.verbose} { println!("note: {} off", mn.n); }
                        let _ = self.conn_out.send(&[NOTE_OFF_MSG, mn.n, mn.vel]);
                    }
                },
                KDir::Down => {
                    if self.note_states[mn.n as usize] == 0 {
                        if unsafe {GLB.verbose} { println!("note: {} on", mn.n); }
                        let _ = self.conn_out.send(&[NOTE_ON_MSG, mn.n, mn.vel]);
                    }
                    self.note_states[mn.n as usize] += 1;
                },
            }
        }
    }
}

struct DevHandles {
    files: Vec<String>,
    devs: Vec<HANDLE>,
    amt: usize,
}

impl DevHandles {
    fn new(amt: usize) -> Self {
        DevHandles { devs: Vec::with_capacity(amt), files: Vec::with_capacity(amt), amt }
    }

    fn push_d(&mut self, dev: HANDLE) {
        self.devs.push(dev);
    }

    fn push_f(&mut self, fl: String) {
        self.files.push(fl);
    }

    fn populate_devs(&self, k_m: &mut KeyMap) {
        let old_k_m = k_m.to_owned();
        k_m.clear();
        for ((h, v_k), mn) in old_k_m {
            k_m.insert((self.devs[h as usize], v_k), mn);
        }
    }

    fn is_full(&self) -> bool {
        self.devs.len() == self.amt
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
    fn new(r: &RAWINPUT) -> RawKRecord {
        let k_dir = unsafe { match r.data.keyboard().Message {
            WM_KEYDOWN | WM_SYSKEYDOWN => KDir::Down,
            WM_KEYUP | WM_SYSKEYUP => KDir::Up,
            _ => unreachable!("recieved non key message from rawinput"),
        } };
        let h_dev = r.header.hDevice;
        let v_k_code = unsafe { r.data.keyboard().VKey };
        return RawKRecord {k_dir, h_dev, v_k_code};
    }
}

struct RawKeyLogs {
    bound_keys: Vec<(HANDLE, USHORT)>,
    toggle_keys: Vec<(HANDLE, USHORT)>,
}

impl RawKeyLogs {
    fn new() -> Self {
        RawKeyLogs {
            bound_keys: Vec::new(),
            toggle_keys: Vec::new(),
        }
    }

    fn process_raw(&mut self, l_param: LPARAM) -> Option<RawKRecord> {
        let mut rid_size: UINT = 0;
        unsafe { GetRawInputData(l_param as HRAWINPUT, RID_INPUT, null_mut(), &mut rid_size, size_of::<RAWINPUTHEADER>() as UINT); }
        if rid_size == 0 { return None; } // not sure if this can happen, but microsoft docs do this
        let mut raw_data_buffer: Vec<u8> = Vec::with_capacity(rid_size.try_into().unwrap());

        unsafe {
            if rid_size != GetRawInputData(
                l_param as HRAWINPUT,
                RID_INPUT,
                raw_data_buffer.as_mut_ptr() as LPVOID,
                &mut rid_size,
                size_of::<RAWINPUTHEADER>() as UINT
            ) && GLB.verbose { println!("GetRawInputData does not return correct size!"); }
        }
        
        let raw: &RAWINPUT = unsafe { &*(raw_data_buffer.as_ptr() as *const RAWINPUT) };

        if raw.header.dwType == RIM_TYPEKEYBOARD {
            let rr = RawKRecord::new(raw);
            return Some(rr);
        }
        return None;
    }
}

fn main() {
    unsafe { GLB.verbose = false; }
    let mut midi_handler = MidiHandler::new().expect("error creating MidiHandler");
    unsafe{ GLB.midi_handler = &mut midi_handler; }
    let csv_parser = CsvParser::new();
    let args: Vec<String> = env::args().collect(); // TODO error if file does not end with .csv

    // ---------------------------------------------------------------------------------------------------------
    // Key map init vv------------------------------------------------------------------------------------------
    let mut key_aliases: HashMap<String, USHORT> = HashMap::new();
    let mut key_map: KeyMap = HashMap::new();
    unsafe { GLB.key_map = &mut key_map; }

    // Populate key_aliases
    {
        let key_aliases_fp = args.get(1).expect("Correct usage: "); //TODO allow ommission of this parameter // TODO finish Correct usage
        let key_aliases_string = fs::read_to_string(key_aliases_fp).expect("Failed to read file: ");
        let keyaliases_csv = csv_parser.cells_as_vec(key_aliases_string.as_str());
        for s in keyaliases_csv.iter() {
            if s.trim().is_empty() { // Ignore strings of whitespace
                continue;
            }
            let mut iter = s.splitn(2, '=');
            let alias = iter.next().expect("Missing alias and key data in alias CSV file").trim();
            let gp_key: USHORT = iter.next().expect("Missing alias or key data in alias CSV file").trim().parse().expect("Wrong syntax in alias CSV file");
            key_aliases.insert(alias.to_owned(), gp_key.to_owned());
        }
    }

    // Populate key_map
    let keymap_files: &[String] = &args[2..];
    if keymap_files.is_empty() { panic!("Correct usage: ") } // TODO finish Correct usage
    let mut dev_handles = DevHandles::new(keymap_files.len());
    unsafe { GLB.dev_handles = &mut dev_handles}
    for (i, keymap_fp) in keymap_files.iter().enumerate() {
        unsafe { (*GLB.dev_handles).push_f(keymap_fp.to_owned()); }
        
        let keymap_string = fs::read_to_string(keymap_fp).expect("Failed to read file: ");
        let keymap_csv = csv_parser.cells_as_vec(keymap_string.as_str());
        
        for s in keymap_csv.iter() {
            if s.trim().is_empty() { // Ignore strings of whitespace
                continue;
            }
            let mut iter = s.splitn(3, |c| c == '=' || c == '.');
            let chord_str = iter.next().expect("Wrong syntax in keymap CSV file").trim();
            let alias = iter.next().expect("Wrong syntax in keymap CSV file").trim();
            let key: USHORT = match key_aliases.get(alias) {
                Some(s) => *s,
                None => alias.parse().expect("alias not in aliases.csv and not a number\neither add to aliases.csv or use a correctly formatted number"),
            };
            if chord_str == "TOGGLE" {
                key_map.insert((i as HANDLE, key.to_owned()), None);
                break;
            }
            let vel: u8 = iter.next().expect("Wrong syntax in keymap CSV file").trim()
                .parse().expect("Wrong syntax in keymap CSV file"); // TODO More descriptive error messages?
            let chord = Chord::new(chord_str).expect("Wrong syntax in keymap CSV file")
                .to_midi_chord(vel).expect("Error creating chord");
            key_map.insert((i as HANDLE, key.to_owned()), Some(chord));
        }
    }

    println!("key_map: {:?}", key_map);
    // Key map init ^^------------------------------------------------------------------------------------------
    // ---------------------------------------------------------------------------------------------------------

    // ---------------------------------------------------------------------------------------------------------
    // Windows init vv------------------------------------------------------------------------------------------
    let h_instance = unsafe { GetModuleHandleW(null_mut()) };

    let mut startup_info: STARTUPINFOW;
    unsafe {
        startup_info = std::mem::zeroed();
        GetStartupInfoW(&mut startup_info); // Don't think I need to free this
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

    let rid_tlc = RAWINPUTDEVICE {
        usUsagePage: 1, // HID_USAGE_PAGE_GENERIC
        usUsage: 6, // HID_USAGE_GENERIC_KEYBOARD
        dwFlags: RIDEV_NOLEGACY | RIDEV_INPUTSINK, // ignores legacy keyboard messages and reads input when not focused
        hwndTarget: hwnd,
    };

    unsafe { 
        if RegisterRawInputDevices(&rid_tlc, 1, size_of::<RAWINPUTDEVICE>() as UINT) == 0 {
            panic!("failed to register raw input TLC");
        }
    }
    // Windows init ^^------------------------------------------------------------------------------------------
    // ---------------------------------------------------------------------------------------------------------

    let mut raw_key_logs = RawKeyLogs::new();
    unsafe{ GLB.raw_key_logs = &mut raw_key_logs; }

    unsafe {
        let mut lp_msg: MSG = std::mem::zeroed();
        println!("Msg loop started");
        println!("-------- Press any key to set device [0], to be assigned mappings from {} --------", (*GLB.dev_handles).files[0]);
        while GetMessageW(&mut lp_msg, 0 as HWND, 0, 0) > 0 {
            DispatchMessageW(&lp_msg);
        }
    }

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

unsafe extern "system" fn wnd_proc(h_wnd: HWND, i_message: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    use winapi::{um::winuser::{DefWindowProcW, WM_DESTROY}};

    match i_message {
        WM_DESTROY => {
            dbg!(PostQuitMessage(0));
            return 0;
        },
        WM_INPUT => {
            let timer = Instant::now();
            let r = match (*GLB.raw_key_logs).process_raw(l_param) {
                Some(rec) => rec,
                None => return 0,
            };
            if !(*GLB.dev_handles).is_full() {
                if r.k_dir == KDir::Down {
                    (*GLB.dev_handles).push_d(r.h_dev);
                    if (*GLB.dev_handles).is_full() {
                        println!("Resolving device handles...");
                        (*GLB.dev_handles).populate_devs(&mut *GLB.key_map);
                        for ((h, v_k), c) in &*GLB.key_map {
                            if c.is_none() {
                                (*GLB.raw_key_logs).toggle_keys.push((h.to_owned(), v_k.to_owned()));
                            }
                            (*GLB.raw_key_logs).bound_keys.push((h.to_owned(), v_k.to_owned()));
                            (*GLB.midi_handler).insert_key((h.to_owned(), v_k.to_owned()))
                        }
                        println!("-------- All devices set! --------");
                    } else {
                        let len = (*GLB.dev_handles).devs.len();
                        println!("-------- Press any key to set device [{}], to be assigned mappings from {} --------", len, (*GLB.dev_handles).files[len]);
                    }
                }
            } else if let Some(o_c) = (*GLB.key_map).get(&(r.h_dev, r.v_k_code)) {
                match o_c {
                    Some(c) => {
                        if GLB.verbose { println!("Playing chord: {:?}", c); }
                        (*GLB.midi_handler).process_msg(r, c);
                        if GLB.verbose { println!("Sent midi msg, took {:?} from input to send", timer.elapsed()); }
                    },
                    None => {
                        if r.k_dir == KDir::Down {
                            todo!("Implement toggling pressing keys");
                        }
                    }
                }
            }
            return 0;
        },
        _ => DefWindowProcW(h_wnd, i_message, w_param, l_param),
    }
}

fn win32_string( value : &str ) -> Vec<u16> {
    use std::{ffi::OsStr, os::windows::prelude::OsStrExt, iter::once};
    OsStr::new( value ).encode_wide().chain( once( 0 ) ).collect()
}