![img](https://github.com/PolyMeilex/OxiSynth/assets/20758186/3773d917-920f-498b-94a1-a7504bd986f8)


# OxiSynth

`fluidsynth` soundfont synthesizer but in pure safe Rust.

### Showcase

OxiSynth running in [Neothesia](https://github.com/PolyMeilex/Neothesia)

https://github.com/PolyMeilex/OxiSynth/assets/20758186/e1d1c3c1-0869-4a8f-96c9-6f16a56767cb

https://github.com/PolyMeilex/OxiSynth/assets/20758186/d50d2e95-f316-472f-88e9-111ed2f52784

OxiSynth running in browser, with Boomwhacker soundfont bundled:
[https://oxisynth.netlify.app](https://oxisynth.netlify.app)

### Project Structure

- `./src` - OxiSynth
- `./soundfont-rs` - Pure Rust implementation of sf2/3 file parser (intended to be also usable outside of `oxisynth`)
