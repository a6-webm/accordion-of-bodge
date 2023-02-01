# Accordion of Bodge
Accordion of Bodge is a program that binds keys on your keyboard (or keyboards!) to play midi notes.
#### [Windows info](https://github.com/a6-webm/accordion-of-bodge#on-windows)
#### [Linux info](https://github.com/a6-webm/accordion-of-bodge#on-linux)

## On Windows
### Usage
```
Usage:
  acc-bodge [-h | --help] <alias-file> (<keymap-file>...)

Options:
  -h, --help   Show this screen
```

### Building
Download the source code and run `cargo build` in the `\accordion_win` folder.

## On Linux
### Usage
```
Usage:
  acc-bodge [-h | --help] (<evdev-device-file> <keymap-file>...)
  
Options:
  -h, --help   Show this screen
```

### Building
Download the source code and run `cargo build` in the `\accordion_linux` folder.
