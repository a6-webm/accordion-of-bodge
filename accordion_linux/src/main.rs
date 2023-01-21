use chord_parser::{Chord, ChordError, CsvParser, MidiNote};
use std::{collections::HashMap, env, error::Error, fmt::Display, fs, io, process::exit};

type Kmap = HashMap<u16, Vec<MidiNote>>;

#[derive(Debug)]
enum AccError {
    ArgsNoKmap,
    IOError(io::Error),
    InvChord(ChordError),
    InvKeyName(String),
    InvVelocity(String),
    NoArgs,
    NoKey(String),
    NoVel(String),
}

use evdev::Key;
use AccError::*;

impl Display for AccError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl Error for AccError {}

fn parse_to_kmap(kmap_csv: Vec<String>) -> Result<Kmap, AccError> {
    let mut out: Kmap = Default::default();
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
    let mut code: u16 = 0;
    for s in trim_kmap_csv.iter() {
        match loop_state {
            P::Chord => {
                chord_str = s;
                loop_state = P::Key;
            }
            P::Key => {
                let key: Key = s.parse().map_err(|_| InvKeyName(s.to_string()))?;
                code = key.code();
                loop_state = P::Vel;
            }
            P::Vel => {
                let vel: u8 = s.parse().map_err(|_| InvVelocity(s.to_string()))?;
                let midi_chord = Chord::new(chord_str)
                    .map_err(|e| InvChord(e))?
                    .to_midi_chord(vel)
                    .map_err(|e| InvChord(e))?;
                out.insert(code, midi_chord);
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
        P::Vel => Err(NoKey(format!(
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

fn program() -> Result<(), AccError> {
    let correct_usage = "------ Correct usage: accordionbodge <alias file> <keymap file>... ------";
    let args: Vec<String> = env::args().collect();

    let mut devs_and_kmap_paths: Vec<(String, String)> = Vec::new();
    let mut iter = args.iter();
    let mut next = iter.next();
    if next.is_none() {
        Err(NoArgs)?;
    }
    while next.is_some() {
        let dev_path = next.unwrap();
        next = iter.next();
        let kmap_path = next.ok_or(ArgsNoKmap)?;
        devs_and_kmap_paths.push((dev_path.to_owned(), kmap_path.to_owned()));
        next = iter.next();
    }

    let csv_parser = CsvParser::new();

    let mut kmaps: Vec<Kmap> = Vec::new();
    for (_, kmap_path) in devs_and_kmap_paths {
        let kmap_str = fs::read_to_string(kmap_path).map_err(|e| IOError(e))?;
        let kmap_csv = csv_parser.cells_as_vec(&kmap_str);
        let kmap = parse_to_kmap(kmap_csv)?;
        kmaps.push(kmap);
    }

    loop {}
}

fn main() {
    // if let Err(e) = program() {
    //     match e {
    //         NoArgs => todo!(), // write error messages to stderr
    //         ArgsNoKmap => todo!(),
    //         InvKeyName => todo!(),
    //         IOError(_) => todo!(),
    //     }
    //     exit(1);
    // }
    program(); // ONLY HERE SO I CAN DEBUG, REMOVE LATER
}
