![img](https://i.imgur.com/V9KLOJE.png)

# OxiSynth

`fluidsynth` soundfont synthesizer but in pure safe Rust.

### Showcase

OxiSynth running in browser, with Boomwhacker soundfont bundled:
[https://oxisynth.netlify.app](https://oxisynth.netlify.app)

### Project Structure

- `./src/core` - Safe but not rusty enough part of the project based on `fluidlite`, ideally it should be completely faded out over time.
- `./src` - Temporary public api surface of `core`
- `./soundfont-rs` - Pure Rust implementation of sf2 file parser (intended to be also usable outside `oxisynth`)
