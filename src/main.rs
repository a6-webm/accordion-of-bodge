// enum of Keyboard keys
// f(key) -> GlovePIE keycode

// parse CSV of notes/chords mapped to keyboard(||stradella bass system?)

use std::collections::HashMap;
use std::{env, fs};
use regex::Regex;

type KeyCode = String;

enum NoteLetter {
    C=0,
    D=2,
    E=4,
    F=5,
    G=7,
    A=9,
    B=11,
}

struct Note {
    octave: u8,
    note: NoteLetter,
    accidental: i8,
}

enum Chord {
    Custom(Vec<Note>),
    Maj {root: Note, over: Option<Note>},
    Min {root: Note, over: Option<Note>},
    Mm7 {root: Note, over: Option<Note>},
    Dim {root: Note, over: Option<Note>},
}

impl Chord {
    fn new(s: &str) -> Option<Chord> {
        todo!()
    }

    fn to_midi_chord(&self, vel: u8) -> Vec<MidiNote> {
        todo!()
    }
}

impl Note {
    fn new(s: &str) -> Option<Note> {
        let note = match s.chars().next() {
            Some('A') => NoteLetter::A,
            Some('B') => NoteLetter::B,
            Some('C') => NoteLetter::C,
            Some('D') => NoteLetter::D,
            Some('E') => NoteLetter::E,
            Some('F') => NoteLetter::F,
            Some('G') => NoteLetter::G,
            _ => return None,
        };
        let mut accidental: u8 = 0;
        let mut iter = s.chars().skip(1).peekable();
        match iter.peek() {
            Some('b') => {
                while iter.peek().map(|o| *o) == Some('b') {
                    accidental -= 1;
                    iter.next();
                }
            },
            Some('#') => {
                while iter.peek().map(|o| *o) == Some('#') {
                    accidental += 1;
                    iter.next();
                }
            },
            Some(_) => (),
            None => return None
        } // TODO redo this to give you an index of where the accidentals end, then parse a slice from then on to get octave ----------------------------
        
        todo!()
    }
    
    fn to_midi(&self, vel: u8) -> MidiNote {
        todo!("todo") // TODO todo (todo) [todo] {Middle C is C4}
    }
}

struct MidiNote {
    n: u8,
    vel: u8,
}

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
            for group in caps.iter().skip(1) {
                if let Some(m) = group {
                    out.push(m.as_str().to_owned());
                }
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
            .to_midi_chord(vel);
        let key = match key_aliases.get(alias) {
            Some(s) => s,
            None => {
                println!("\"{}\" not in aliases.csv. Using alias, but it may not be a valid GlovePIE key", alias);
                alias
            },
        };
        key_map.insert(key.to_owned(), chord);
    }

}
