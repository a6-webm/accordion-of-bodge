#![allow(clippy::needless_return)]

use std::fmt::{Display, self};
use std::num::ParseIntError;
use regex::Regex;

pub struct CsvParser {
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

#[derive(Debug, Clone)]
pub enum ChordError {
    EmptyStr,
    InvNote(NoteError, String),
    MissingNotes,
    MissingOver,
    InvChordType(char, String),
    InvRoot {root: Note, over: Note, parse: String},
}

impl Display for ChordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChordError::EmptyStr => write!(f, "could not create Chord from empty string or whitespace"),
            ChordError::InvNote(e, s) => write!(f, "could not create chord, {e}, in '{s}'"),
            ChordError::MissingNotes => write!(f, "could not create Chord, no notes"),
            ChordError::MissingOver => write!(f, "could not create Chord, '/' present but no note"),
            ChordError::InvChordType(c, s) => write!(f, "could not create Chord, '{c}' not a valid chord type in '{s}'"),
            ChordError::InvRoot { root, over, parse } => write!(f, "could not create Chord, {root:?} is higher than {over:?} in '{parse}'"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum NoteError {
    EmptyStr,
    InvNoteLetter(String),
    MissingOctave(String),
    InvOctave(ParseIntError, String),
    InvMidiVel(u8),
    InvMidiNote,
}

impl Display for NoteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NoteError::EmptyStr => write!(f, "could not create Note from empty string or whitespace"),
            NoteError::InvNoteLetter(c) => write!(f, "could not create Note, invalid NoteLetter of '{c}'"),
            NoteError::MissingOctave(s) => write!(f, "could not create Note, missing Octave value in '{s}'"),
            NoteError::InvOctave(parse_int_error, s) => write!(f, "could not create Note, invalid Octave, {parse_int_error}, in '{s}'"),
            NoteError::InvMidiVel(v) => write!(f, "could not create MidiNote from Note, vel:{v} > 127"),
            NoteError::InvMidiNote => write!(f, "could not create MidiNote from Note, note value < 0 or > 127"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum NoteLetter {
    C=0,
    D=2,
    E=4,
    F=5,
    G=7,
    A=9,
    B=11,
}

#[derive(Debug, Clone)]
pub struct Note {
    octave: i8,
    note: NoteLetter,
    accidental: i8,
}

#[derive(Debug, Clone)]
pub enum Chord {
    Custom(Vec<Note>),
    Maj {root: Note, over: Option<Note>},
    Min {root: Note, over: Option<Note>},
    Mm7 {root: Note, over: Option<Note>},
    Dim {root: Note, over: Option<Note>},
}

impl Chord {
    pub fn new(s: &str) -> Result<Self, ChordError> {
        let notes_str: Vec<&str> = s.split_whitespace().collect();
        if notes_str.is_empty() {
            Err(ChordError::EmptyStr)
        } else if notes_str.len() == 1 {
            let (chord, over) = {
                let has_over = s.find('/') != None;
                let mut iter = s.splitn(2, '/');
                let chord = match iter.next() { Some(ch) => ch, None => return Err(ChordError::MissingNotes), };
                if let Ok(n) = Note::new(chord) {
                    if !has_over {
                        return Ok(Chord::Custom(vec![n]));
                    }
                }
                let over = match (iter.next(), has_over) {
                    (Some(ss), true) => Some(Note::new(ss).map_err(|e| ChordError::InvNote(e, s.to_owned()))?),
                    (_, false) => None,
                    (None, true) => return Err(ChordError::MissingOver),
                };
                (chord, over)
            };

            let root = Note::new(&chord[..chord.len()-1]).map_err(|e| ChordError::InvNote(e, s.to_owned()))?;
            
            if let Some(over) = &over {
                if over.pitch() > root.pitch() {
                    return Err(ChordError::InvRoot {
                        root,
                        over: over.to_owned(),
                        parse: s.to_owned(),
                    });
                }
            }
            
            return match chord.chars().last() {
                Some('M') => Ok(Chord::Maj {root, over}),
                Some('m') => Ok(Chord::Min {root, over}),
                Some('7') => Ok(Chord::Mm7 {root, over}),
                Some('o') => Ok(Chord::Dim {root, over}),
                Some(c) => Err(ChordError::InvChordType(c, s.to_owned())),
                None => unreachable!(),
            }
        } else {
            let mut notes: Vec<Note> = Vec::new();
            for ss in notes_str.iter() {
                notes.push(Note::new(ss).map_err(|e| ChordError::InvNote(e, s.to_owned()))?);
            }
            return Ok(Chord::Custom(notes));
        }
    }

    pub fn to_midi_chord(&self, vel: u8) -> Result<Vec<MidiNote>, ChordError> {
        let mut intervals: Vec<u8> = Vec::new();
        let mut out: Vec<MidiNote> = Vec::new();
        let ro;
        let ov;

        match self {
            Chord::Custom(v) => {
                for n in v {
                    out.push(n.to_midi(vel).map_err(|e| ChordError::InvNote(e, format!("{self:?}")))?);
                }
                return Ok(out);
            },
            Chord::Maj { root, over } => {
                intervals.push(4);
                intervals.push(7);
                ro = root;
                ov = over;
            },
            Chord::Min { root, over } => {
                intervals.push(3);
                intervals.push(7);
                ro = root;
                ov = over;
            },
            Chord::Mm7 { root, over } => {
                intervals.push(4);
                intervals.push(7);
                intervals.push(10);
                ro = root;
                ov = over;
            },
            Chord::Dim { root, over } => {
                intervals.push(3);
                intervals.push(6);
                ro = root;
                ov = over;
            },
        }

        if let Some(over) = ov {
            out.push(over.to_midi(vel).map_err(|e| ChordError::InvNote(e, format!("{self:?}")))?);
        }
        let root = ro.to_midi(vel).map_err(|e| ChordError::InvNote(e, format!("{self:?}")))?;
        out.push(root.to_owned());

        for inte in intervals {
            out.push(MidiNote::new(root.n as i32 + inte as i32, vel).map_err(|e| ChordError::InvNote(e, format!("{self:?}")))?);
        }

        return Ok(out);
    }
}

impl Note {
    pub fn new(s: &str) -> Result<Self, NoteError> {
        let mut s_iter = s.chars().peekable();
        let note = match s_iter.next() {
            Some('A') => NoteLetter::A,
            Some('B') => NoteLetter::B,
            Some('C') => NoteLetter::C,
            Some('D') => NoteLetter::D,
            Some('E') => NoteLetter::E,
            Some('F') => NoteLetter::F,
            Some('G') => NoteLetter::G,
            Some(_) => return Err(NoteError::InvNoteLetter(s.to_string())),
            None => return Err(NoteError::EmptyStr),
        };
        let mut accidental: i8 = 0;
        s_iter = match s_iter.peek() {
            Some('b') => {
                while s_iter.peek().copied() == Some('b') {
                    accidental -= 1;
                    s_iter.next();
                }
                s_iter
            },
            Some('#') => {
                while s_iter.peek().copied() == Some('#') {
                    accidental += 1;
                    s_iter.next();
                }
                s_iter
            },
            Some(_) => s_iter,
            None => return Err(NoteError::MissingOctave(s.to_owned())),
        };
        let octave:i8 = match s_iter.collect::<String>().parse() {
            Ok(o) => o,
            Err(e) => return Err(NoteError::InvOctave(e, s.to_owned())),
        };
        return Ok(Note {octave, note, accidental});
    }

    fn pitch(&self) -> i32 {
        (self.octave as i32 + 1) * 12 + self.note.to_owned() as i32 + self.accidental as i32
    }
    
    pub fn to_midi(&self, vel: u8) -> Result<MidiNote, NoteError> {
        MidiNote::new(self.pitch(), vel)
    }
}

#[derive(Debug, Clone)]
pub struct MidiNote {
    pub n: u8,
    pub vel: u8,
}

impl MidiNote {
    pub fn new(pitch: i32, vel:u8) -> Result<Self, NoteError> {
        if vel > 127 {return Err(NoteError::InvMidiVel(vel));}
        let n: u8 = {if pitch >= 0 {
            pitch as u8
        } else {
            return Err(NoteError::InvMidiNote);
        }};
        return Ok(MidiNote { n, vel });
    }
}