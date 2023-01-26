use chord_parser::{Chord, ChordError, CsvParser, MidiNote};
use evdev::{Device, EventType, Key};
use midir::{os::unix::VirtualOutput, MidiOutput, MidiOutputConnection};
use nix::{
    errno::Errno,
    fcntl::{fcntl, FcntlArg, OFlag},
};
use std::{
    collections::HashMap, env, error::Error, fmt::Display, fs, io, os::unix::prelude::AsRawFd,
    process::exit,
};

const NOTE_ON_MSG: u8 = 0x90;
const NOTE_OFF_MSG: u8 = 0x80;

type KCode = u16;
type Kmap = HashMap<KCode, Vec<MidiNote>>;

#[derive(Debug)]
enum AccError {
    ArgError(io::Error),
    MidiError(),
    ArgsNoKmap,
    DeviceFail(io::Error),
    DeviceOpenFail(io::Error),
    DeviceFDFail(Errno),
    InvChord(ChordError),
    InvKeyName(String),
    InvVelocity(String),
    NoArgs,
    NoKey(String),
    NoVel(String),
}
use AccError::*;

impl Display for AccError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl Error for AccError {}

struct MappedDev {
    hndl: Device,
    kmap: Kmap,
    toggles: Vec<KCode>,
}

struct MidiHandler {
    played_notes: Vec<u8>,
    conn_out: MidiOutputConnection,
}

impl MidiHandler {
    fn new() -> Result<MidiHandler, AccError> {
        let midi_out = MidiOutput::new("Accordion_of_Bodge_midi_out").map_err(|_| MidiError())?;
        let conn_out = midi_out
            .create_virtual("Accordion_of_Bodge")
            .map_err(|_| MidiError())?;
        Ok(MidiHandler {
            played_notes: vec![0; 128],
            conn_out,
        })
    }

    fn play(&mut self, chord: &Vec<MidiNote>) {
        for note in chord {
            self.played_notes[note.n as usize] += 1;
            if self.played_notes[note.n as usize] == 1 {
                self.conn_out.send(&[NOTE_ON_MSG, note.n, note.vel]).ok();
                println!("played note: {} with vel: {}", note.n, note.vel);
            }
        }
    }

    fn release(&mut self, chord: &Vec<MidiNote>) {
        for note in chord {
            if self.played_notes[note.n as usize] != 0 {
                self.played_notes[note.n as usize] -= 1;
            }
            if self.played_notes[note.n as usize] == 0 {
                self.conn_out.send(&[NOTE_OFF_MSG, note.n, note.vel]).ok();
                println!("released note: {} with vel: {}", note.n, note.vel);
            }
        }
    }
}

fn parse_to_kmap_and_toggles(kmap_csv: Vec<String>) -> Result<(Kmap, Vec<KCode>), AccError> {
    let mut out: (Kmap, Vec<KCode>) = Default::default();
    let trim_kmap_csv: Vec<&str> = kmap_csv
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    enum P {
        Chord,
        Key,
        Vel,
    }
    let mut loop_state = P::Chord;
    let mut chord_str = "";
    let mut code: KCode = 0;
    for s in trim_kmap_csv.iter() {
        match loop_state {
            P::Chord => {
                chord_str = s;
                loop_state = P::Key;
            }
            P::Key => {
                let key: Key = s.parse().map_err(|_| InvKeyName(s.to_string()))?;
                code = key.code();
                if chord_str == "TOGGLE" {
                    out.1.push(code);
                    loop_state = P::Chord;
                } else {
                    loop_state = P::Vel;
                }
            }
            P::Vel => {
                let vel: u8 = s.parse().map_err(|_| InvVelocity(s.to_string()))?;
                let midi_chord = Chord::new(chord_str)
                    .map_err(|e| InvChord(e))?
                    .to_midi_chord(vel)
                    .map_err(|e| InvChord(e))?;
                out.0.insert(code, midi_chord);
                loop_state = P::Chord;
            }
        }
    }
    match loop_state {
        P::Chord => Ok(out),
        P::Key => Err(NoKey(
            trim_kmap_csv
                .last()
                .map(|s| s.to_string())
                .unwrap_or_default(),
        )),
        P::Vel => Err(NoVel(format!(
            "{},{}",
            trim_kmap_csv
                .get(trim_kmap_csv.len() - 2)
                .map(|s| s.to_string())
                .unwrap_or_default(),
            trim_kmap_csv
                .last()
                .map(|s| s.to_string())
                .unwrap_or_default(),
        ))),
    }
}

static USAGE: &'static str = "
Usage:
  acc-bodge [-h | --help] (<evdev-device-file> <keymap-file>...)
  
Options:
  -h, --help   Show this screen
";

fn program() -> Result<(), AccError> {
    let args: Vec<String> = env::args().collect();

    let mut midi_handler = MidiHandler::new()?;

    let mut devs_and_kmap_paths: Vec<(String, String)> = Vec::new();
    let mut iter = args.iter();
    let mut next = iter.nth(1);
    match next {
        Some(s) if s == "-h" || s == "--help" => {
            println!("{}", USAGE);
            exit(0);
        }
        None => return Err(NoArgs),
        _ => (),
    }
    while next.is_some() {
        let dev_path = next.unwrap();
        next = iter.next();
        let kmap_path = next.ok_or(ArgsNoKmap)?;
        devs_and_kmap_paths.push((dev_path.to_owned(), kmap_path.to_owned()));
        next = iter.next();
    }

    let csv_parser = CsvParser::new();

    let mut mapped_devs: Vec<MappedDev> = Vec::new();
    for (dev_path, kmap_path) in devs_and_kmap_paths {
        let device = Device::open(dev_path).map_err(|e| DeviceOpenFail(e))?;
        let raw_fd = device.as_raw_fd();
        fcntl(raw_fd, FcntlArg::F_SETFL(OFlag::O_NONBLOCK)).map_err(|e| DeviceFDFail(e))?;
        let kmap_str = fs::read_to_string(kmap_path).map_err(|e| ArgError(e))?;
        let kmap_csv = csv_parser.cells_as_vec(&kmap_str);
        let (kmap, toggles) = parse_to_kmap_and_toggles(kmap_csv)?;
        mapped_devs.push(MappedDev {
            hndl: device,
            kmap,
            toggles,
        });
    }

    let mut prev_toggle_key: Option<KCode> = None;
    loop {
        'off: loop {
            for dev in mapped_devs.iter_mut() {
                match dev.hndl.fetch_events() {
                    Ok(ev_iter) => {
                        for ev in ev_iter {
                            if ev.event_type() == EventType::KEY && dev.toggles.contains(&ev.code())
                            {
                                if ev.value() == 1 {
                                    prev_toggle_key = Some(ev.code());
                                } else if ev.value() == 0 {
                                    if prev_toggle_key == Some(ev.code()) {
                                        prev_toggle_key = None;
                                        break 'off;
                                    }
                                    prev_toggle_key = None;
                                }
                            }
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => return Err(DeviceFail(e)),
                }
            }
        }
        for dev in mapped_devs.iter_mut() {
            dev.hndl.grab().map_err(|e| DeviceFail(e))?;
        }
        println!("devices grabbed, playing on");
        'on: loop {
            for dev in mapped_devs.iter_mut() {
                match dev.hndl.fetch_events() {
                    Ok(ev_iter) => {
                        for ev in ev_iter {
                            if ev.event_type() != EventType::KEY {
                                continue;
                            }
                            if dev.toggles.contains(&ev.code()) {
                                if ev.value() == 1 {
                                    prev_toggle_key = Some(ev.code());
                                } else if ev.value() == 0 {
                                    if prev_toggle_key == Some(ev.code()) {
                                        prev_toggle_key = None;
                                        break 'on;
                                    }
                                    prev_toggle_key = None;
                                }
                            }

                            if let Some(chord) = dev.kmap.get(&ev.code()) {
                                if ev.value() == 1 {
                                    midi_handler.play(chord);
                                }
                                if ev.value() == 0 {
                                    midi_handler.release(chord);
                                }
                            }
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => return Err(DeviceFail(e)),
                }
            }
        }
        for dev in mapped_devs.iter_mut() {
            dev.hndl.ungrab().map_err(|e| DeviceFail(e))?;
        }
        println!("devices released, playing off");
    }
}

fn main() {
    if let Err(e) = program() {
        println!("{:?}", e);
        exit(1);
    }
}
