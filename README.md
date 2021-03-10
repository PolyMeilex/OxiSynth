# OxiSynth

`fluidsynth` based soundfont synthesizer

### Showcase
OxiSynth running in browser, with Boomwhacker soundfont bundled:
[https://oxisynth.netlify.app](https://oxisynth.netlify.app)


### Project Structure

- `./unoxidized` - Safe but not rusty enough part of the project based on `fluidlite`, ideally it should be completely faded out over time.
- `./src` - Temporary public api surface of `unoxidized`
- `./soundfont-rs` - Pure Rust implementation of sf2 file parser (intended to be also usable outside `oxisynth`)
