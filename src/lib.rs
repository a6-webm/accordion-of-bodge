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
    InvRoot {root: Note, over: Note},
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

impl From<NoteError> for ChordError {
    fn from(e: NoteError) -> Self {
        return ChordError::InvNote(e);
    }
}

impl Display for NoteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NoteError::EmptyStr => write!(f, "could not create Note from empty string or whitespace"),
            NoteError::InvNoteLetter(c) => write!(f, "could not create Note, invalid NoteLetter of '{}'", c),
            NoteError::MissingOctave => write!(f, "could not create Note, missing Octave value"),
            NoteError::InvOctave(parse_int_error) => write!(f, "could not create Note, invalid Octave, {}", parse_int_error),
            NoteError::InvMidiVel(v) => write!(f, "could not create MidiNote from Note, vel:{} > 127", v),
            NoteError::InvMidiNote() => write!(f, "could not create MidiNote from Note, note value < 0 or > 127"),
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
        if notes_str.len() == 0 {
            return Err(ChordError::EmptyStr);
        } else if notes_str.len() == 1 {
            let (chord, over) = {
                let has_over = s.find('/') != None;
                let mut iter = s.splitn(2, '/');
                let chord = match iter.next() { Some(ch) => ch, None => return Err(ChordError::MissingNotes), };
                let over = match (iter.next(), has_over) {
                    (Some(s), true) => Some(Note::new(s)?),
                    (_, false) => None,
                    (None, true) => return Err(ChordError::MissingOver),
                };
                (chord, over)
            };

            let root = Note::new(&chord[..chord.len()-1])?;
            
            if let Some(over) = &over {
                if over.pitch() > root.pitch() {
                    return Err(ChordError::InvRoot { root: root.to_owned(), over: over.to_owned() });
                }
            }
            
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
                notes.push(Note::new(s)?);
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
                    out.push(n.to_midi(vel)?);
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
            out.push(over.to_midi(vel)?);
        }
        let root = ro.to_midi(vel)?;
        out.push(root.to_owned());

        for inte in intervals {
            out.push(MidiNote::new(root.n as i32 + inte as i32, vel)?);
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

    fn pitch(&self) -> i32 {
        return (self.octave as i32 + 1) * 12 + self.note as i32 + self.accidental as i32;
    }
    
    pub fn to_midi(&self, vel: u8) -> Result<MidiNote, NoteError> {
        return MidiNote::new(self.pitch(), vel);
    }
}

#[derive(Debug, Clone)]
pub struct MidiNote {
    n: u8,
    vel: u8,
}

impl MidiNote {
    pub fn new(pitch: i32, vel:u8) -> Result<Self, NoteError> {
        if vel > 127 {return Err(NoteError::InvMidiVel(vel));}
        let n: u8 = {if pitch >= 0 {
            pitch as u8
        } else {
            return Err(NoteError::InvMidiNote());
        }};
        return Ok(MidiNote { n, vel: vel });
    }
}