use std::{collections::HashMap, env, fmt::Display, fs, process::exit};

use chord_parser::CsvParser;

#[derive(Debug)]
enum AccErr {
    NoArgs,
    ArgsNoKmap,
}

impl Display for AccErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccErr::NoArgs => write!(f, "uh idk lol"),
            AccErr::ArgsNoKmap => todo!(),
        }
    }
}

fn main() -> Result<(), AccErr> {
    let correct_usage = "------ Correct usage: accordionbodge <alias file> <keymap file>... ------";
    let args: Vec<String> = env::args().collect();

    let devs_and_kmap_paths: Vec<(String, String)> = Vec::new();
    let iter = args.iter();
    let next = iter.next();
    if next.is_none() {
        return Err(AccErr::NoArgs);
    }
    while (next.is_some()) {
        let dev_path = next.unwrap();
        next = iter.next();
        let kmap_path = next.ok_or(AccErr::ArgsNoKmap)?;
        devs_and_kmap_paths.push((dev_path, kmap_path));
        next = iter.next();
    }

    let csv_parser = CsvParser::new();

    // Populate key_map
    // let keymap_files: &[String] = &args[1..];
    // if keymap_files.is_empty() {
    //     println!("{}", correct_usage);
    //     exit(1);
    // }
    // for (i, keymap_fp) in keymap_files.iter().enumerate() {
    //     unsafe {
    //         (*GLB.keymap_builder).push_f(keymap_fp.to_owned());
    //     }

    //     let keymap_string = fs::read_to_string(keymap_fp).unwrap_or_else(|e| {
    //         println!("Error, failed to read file: {}", e);
    //         exit(1);
    //     });
    //     let keymap_csv = csv_parser.cells_as_vec(keymap_string.as_str());

    //     enum Parse {
    //         Chord,
    //         Key,
    //         Vel,
    //     }
    //     let mut loop_state = Parse::Chord;
    //     let mut chord_str = "";
    //     let mut s_code: u32 = 0;
    //     for s in keymap_csv.iter() {
    //         if s.trim().is_empty() {
    //             // Ignore strings of whitespace
    //             continue;
    //         }
    //         match loop_state {
    //             Parse::Chord => {
    //                 chord_str = s.trim();
    //                 loop_state = Parse::Key;
    //             }
    //             Parse::Key => {
    //                 s_code = s.trim();
    //                 loop_state = Parse::Vel;
    //             }
    //             Parse::Vel => {
    //                 let vel: u8 = s.trim().parse().unwrap_or_else(|_| {
    //                     println!("Error: {} is not a valid velocity in {}", s, keymap_fp);
    //                     exit(1);
    //                 });
    //                 let chord = Chord::new(chord_str)
    //                     .unwrap_or_else(|e| {
    //                         println!("Error, wrong syntax in keymap file {}: {}", keymap_fp, e);
    //                         exit(1);
    //                     })
    //                     .to_midi_chord(vel)
    //                     .unwrap_or_else(|e| {
    //                         println!("Error, wrong syntax in keymap file {}: {}", keymap_fp, e);
    //                         exit(1);
    //                     });
    //                 unsafe {
    //                     (*GLB.keymap_builder).push_km((i as HANDLE, s_code.to_owned()), chord);
    //                 };
    //                 loop_state = Parse::Chord;
    //             }
    //         }
    //     }
    //     match loop_state {
    //         Parse::Chord => (),
    //         Parse::Key => {
    //             println!(
    //                 "Error: Chord '{}' has no scancode specified after it in {}",
    //                 chord_str, keymap_fp
    //             );
    //             exit(1);
    //         }
    //         Parse::Vel => {
    //             println!(
    //                 "Error: No velocity specified after '{},{}' in {}",
    //                 chord_str, key, keymap_fp
    //             );
    //             exit(1);
    //         }
    //     }
    // }

    loop {}
}
