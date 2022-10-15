// enum of Keyboard keys
// f(key) -> GlovePIE keycode

// parse CSV of notes/chords mapped to keyboard(||stradella bass system?)

use std::collections::HashMap;
use std::{env, fs};

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

struct MidiNote {
    i: u8,
    vel: u8,
}

impl Note {
    fn to_midi(&self, vel: u8) -> MidiNote {
        todo!("todo") // TODO todo (todo) [todo] {Middle C is C4}
    }

    fn new(s: &String) -> Note {
        todo!("todo") // TODO todo (todo) [todo] {Middle C is C4}
    }
}

fn main() {
    let args: Vec<String> = env::args().collect(); // TODO error if file does not end with .csv
    let file_path = args.first()
        .unwrap_or_else(|| panic!("Correct usage: ")); //TODO add correct usage text
    let csv_string = fs::read_to_string(file_path)
        .expect("Failed to read file: ");
    
    let parsed_csv: Vec<Vec<&str>> = csv_string.split('\n').map(
        |v| v.split(", ").collect()
    ).collect();

    let key_map: HashMap<KeyCode, MidiNote> = HashMap::new();

    for row in parsed_csv.iter() {
        for col in row.iter() {

        }
    }

    // parse CSV into a HashMap<KeyCode, MidiNote>
    

}
