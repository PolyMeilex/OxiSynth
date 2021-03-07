# OxiSynth

`fluidsynth` based soundfont synthesizer

Project Structure

- `./unoxidized` - Safe but not rusty enough part of the project based on `fluidlite`, ideally it should be completely faded out over time.
- `./src` - Temporary public api surface of `unoxidized`
- `./sf2` - Pure Rust implementation of sf2 file parser (intended to be also usable outside `oxisynth`)
