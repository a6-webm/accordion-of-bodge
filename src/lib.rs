use std::fmt::{Display, self};
use std::num::ParseIntError;

pub type KeyCode = String;

#[derive(Debug, Clone)]
pub enum ChordError {
    EmptyStr,
    InvNote(NoteError),
    MissingNotes,
    MissingOver,
    InvChordType(char),
}

#[derive(Debug, Clone)]
pub enum NoteError {
    EmptyStr,
    InvNoteLetter(char),
    MissingOctave,
    InvOctave(ParseIntError),
    InvMidiVel(u8),
    InvMidiNote(),
}

impl Display for NoteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NoteError::EmptyStr => write!(f, "could not create Note from empty string or whitespace"),
            NoteError::InvNoteLetter(c) => write!(f, "could not create Note, invalid NoteLetter of '{}'", c),
            NoteError::MissingOctave => write!(f, "could not create Note, missing Octave value"),
            NoteError::InvOctave(ParseIntError) => write!(f, "could not create Note, invalid Octave, {}", ParseIntError),
            NoteError::InvMidiVel(v) => write!(f, "could not create MidiNote from Note, vel > 127"),
            NoteError::InvMidiNote() => write!(f, "could not create MidiNote from Note, note value < 0 or > 127"),
        }
    }
}

pub enum NoteLetter {
    C=0,
    D=2,
    E=4,
    F=5,
    G=7,
    A=9,
    B=11,
}

pub struct Note {
    octave: i8,
    note: NoteLetter,
    accidental: i8,
}

pub enum Chord {
    Custom(Vec<Note>),
    Maj {root: Note, over: Option<Note>},
    Min {root: Note, over: Option<Note>},
    Mm7 {root: Note, over: Option<Note>},
    Dim {root: Note, over: Option<Note>},
}

impl Chord {
    pub fn new(s: &str) -> Result<Chord, ChordError> {
        let notes_str: Vec<&str> = s.split_whitespace().collect();
        if notes_str.len() == 0 {
            return Err(ChordError::EmptyStr);
        } else if notes_str.len() == 1 {
            let (chord, over) = {
                let has_over = s.find('/') != None;
                let mut iter = s.splitn(2, '/');
                let chord = match iter.next() { Some(ch) => ch, None => return Err(ChordError::MissingNotes), };
                let over = match (iter.next(), has_over) {
                    (Some(s), true) => match Note::new(s) { Ok(n) => Some(n), Err(e) => return Err(ChordError::InvNote(e)), },
                    (_, false) => None,
                    (None, true) => return Err(ChordError::MissingOver),
                };
                (chord, over)
            };

            let root = match Note::new(&chord[..chord.len()-1]) { Ok(n) => n, Err(e) => return Err(ChordError::InvNote(e)), };

            match chord.chars().last() {
                Some('M') => return Ok(Chord::Maj {root, over}),
                Some('m') => return Ok(Chord::Min {root, over}),
                Some('7') => return Ok(Chord::Mm7 {root, over}),
                Some('o') => return Ok(Chord::Dim {root, over}),
                Some(c) => return Err(ChordError::InvChordType(c)),
                None => unreachable!(),
            }
        } else {
            let mut notes: Vec<Note> = Vec::new();
            for s in notes_str.iter() {
                match Note::new(s) {
                    Ok(n) => {notes.push(n)},
                    Err(e) => return Err(ChordError::InvNote(e)),
                }
            }
            return Ok(Chord::Custom(notes));
        }
    }

    pub fn to_midi_chord(&self, vel: u8) -> Vec<MidiNote> {
        todo!()
    }
}

impl Note {
    pub fn new(s: &str) -> Result<Note, NoteError> {
        let mut s_iter = s.chars().peekable();
        let note = match s_iter.next() {
            Some('A') => NoteLetter::A,
            Some('B') => NoteLetter::B,
            Some('C') => NoteLetter::C,
            Some('D') => NoteLetter::D,
            Some('E') => NoteLetter::E,
            Some('F') => NoteLetter::F,
            Some('G') => NoteLetter::G,
            Some(c) => return Err(NoteError::InvNoteLetter(c)),
            None => return Err(NoteError::EmptyStr),
        };
        let mut accidental: i8 = 0;
        s_iter = match s_iter.peek() {
            Some('b') => {
                while s_iter.peek().map(|o| *o) == Some('b') {
                    accidental -= 1;
                    s_iter.next();
                }
                s_iter
            },
            Some('#') => {
                while s_iter.peek().map(|o| *o) == Some('#') {
                    accidental += 1;
                    s_iter.next();
                }
                s_iter
            },
            Some(_) => s_iter,
            None => return Err(NoteError::MissingOctave),
        };
        let octave:i8 = match s_iter.collect::<String>().parse() {
            Ok(o) => o,
            Err(e) => return Err(NoteError::InvOctave(e)),
        };
        return Ok(Note {octave, note, accidental});
    }
    
    pub fn to_midi(&self, vel: u8) -> Result<MidiNote, NoteError> {
        if vel > 127 {return Err(NoteError::InvMidiVel(vel));}
        let pitch = (self.octave + 1) * 12 + (self.note as i8) + self.accidental;
        let n: u8 = if pitch >= 0 {
            pitch as u8
        } else {
            return Err(NoteError::InvMidiNote());
        };
        return Ok(MidiNote { n, vel: vel });
    }
}

pub struct MidiNote {
    n: u8,
    vel: u8,
}