# Accordion of Bodge
With a system that *sometimes* deletes input ~~stolen~~ *ported* from LuaMacros, Accordion of Bodge is a windows program that can let you bind most keys on your keyboard to play any number of midi notes.

Full disclosure, this code certainly accomplishes something, but that something is not polished even a little bit, so be warned.

# Building
Download the source code and run `cargo build` in that folder.

# Running
You can add CSVs for as many keyboards as you want, but there must be at least one CSV for aliases and one CSV for a key presses to midi.

```powershell
cargo run --bin main -- aliases.csv cgriff.csv bass.csv
```
will make two keyboards act like a chromatic button accordion.

# Use
*exercise left for the reader*
