# chip-8-emu-rs
```
chip-8-emu 0.1.0
Amy Y
Interpretting Emulator for Chip-8

USAGE:
    chip-8-emu.exe [OPTIONS] <rom_path>

ARGS:
    <rom_path>    The path of the ROM that is to be loaded into the emulator. If a '.state' file
                  is loaded, the emulator will resume from that save state.

OPTIONS:
    -b, --bg <background_color>    The color in Hex that will be the background color. [default:
                                   000000]
    -f, --fg <foreground_color>    The color in Hex that will be the foreground color. [default:
                                   FFFFFF]
    -h, --hz <hz>                  The amount of loops that the emulator runs in one second.
                                   [default: 500]
        --help                     Print help information
    -i, --invert-colors            Invert colors of the screen of the emulator.
    -v, --volume <volume>          Volume of the beep as a float. [default: 0.2]
    -V, --version                  Print version information
```
