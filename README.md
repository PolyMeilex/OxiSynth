# OxiSynth

`fluidsynth` based soundfont synthesizer

Project Structure

- `./unoxidized` - Unsafe part of the project based on `fluidlite`, ideally it should be completely faded out over time.
- `./src` - Kinda safe wrapper around `unoxidized`
- `./sf2` - Pure Rust implementation of sf2 file parser (intended to be also usable outside `oxisynth`)
