# Accordion of Bodge
Accordion of Bodge is a program that binds keys on your keyboard (or keyboards!) to play midi notes.
#### Windows info
#### Linux info

## On Windows
### Usage
// TODO

### Building
Download the source code and run `cargo build` in the `\accordion_win` folder.

You can add CSVs for as many keyboards as you want, but there must be at least one CSV for aliases and one CSV for a key presses to midi.

```powershell
cargo run -- aliases.csv cgriff.csv bass.csv
```
will make two keyboards act like a chromatic button accordion.

## On Linux
### Usage
// TODO

### Building
Download the source code and run `cargo build` in the `\accordion_linux` folder.
