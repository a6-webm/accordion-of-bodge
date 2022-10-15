// enum of Keyboard keys
// f(key) -> GlovePIE keycode

// parse CSV of notes/chords mapped to keyboard(||stradella bass system?)

use std::collections::HashMap;
use std::{env, fs};
use regex::Regex;

// TODO implement KeyCode

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

impl Note {
    fn to_midi(&self, vel: u8) -> MidiNote {
        todo!("todo") // TODO todo (todo) [todo] {Middle C is C4}
    }

    fn new(s: &str) -> Option<Note> {
        todo!("todo") // TODO todo (todo) [todo] {Middle C is C4}
    }
}

struct MidiNote {
    n: u8,
    vel: u8,
}

struct CSVParser {
    regex: Regex,
}

impl CSVParser {
    fn new() -> CSVParser {
        CSVParser { 
            regex: Regex::new("(?:(?:\"(.*?)\")|(.*?))(?:(?:,\\n)|,|\\n|$)").unwrap() 
        }
    }

    fn cells_as_vec(&self, s: &str) -> Vec<String>{
        let mut out: Vec<String> = Vec::new();
        for caps in self.regex.captures_iter(s) {
            for group in caps.iter() {
                if let Some(m) = group {
                    out.push(m.as_str().to_owned());
                }
            }
        }
        out
    }
}

fn main() {
    let csv_parser = CSVParser::new();
    let args: Vec<String> = env::args().collect(); // TODO error if file does not end with .csv
    let file_path = args.first().expect("Correct usage: "); //TODO add correct usage text
    let csv_string = fs::read_to_string(file_path).expect("Failed to read file: ");
    
    let parsed_csv = csv_parser.cells_as_vec(csv_string.as_str());

    // TODO parse keycode alias map
    // let key_map: HashMap<KeyCode, MidiNote> = HashMap::new();

    for s in parsed_csv.iter() {
        if let None = s.split_whitespace().next() { // Ignore strings of whitespace
            continue;
        }
        let mut iter = s.splitn(2, '=');
        let chord_str = iter.next().expect("Missing chord and key data in cell");
        let key_str = iter.next().expect("Missing chord or key data in cell");
        let chord: Vec<Note> = chord_str.split_whitespace().map(|n| Note::new(n).expect("Note could not be parsed")).collect();
        // let key = 
    }

    // parse CSV into a HashMap<KeyCode, MidiNote>
    

}
