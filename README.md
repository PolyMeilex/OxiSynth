![img](https://github.com/PolyMeilex/OxiSynth/assets/20758186/3773d917-920f-498b-94a1-a7504bd986f8)

# OxiSynth

OxiSynth is a pure safe Rust SoundFont™ synthesizer, inspired by the widely known `FluidSynth`.  

## Primary Use Case  
Initially designed for integration with [Neothesia](https://github.com/PolyMeilex/Neothesia), OxiSynth has expanded its utility to other innovative projects, including:  
- [microwave](https://github.com/Woyten/tune/tree/main/microwave): A microtonal synthesizer that makes creative use of OxiSynth's per-channel tuning APIs.  
- [xsynth](https://github.com/BlackMIDIDevs/xsynth): A Black MIDI synthesizer leveraging OxiSynth's `soundfont` parsing crate.

## Showcase

This is how OxiSynth sounds like, running in real-time in [Neothesia](https://github.com/PolyMeilex/Neothesia)

The chorus and reverb in the recording are also produced by `oxisynth-chorus` and `oxisynth-reverb` crates respectively, that don't have any dependency on `oxisynth` itself and can be used as generic efects outside of this project.

https://github.com/user-attachments/assets/fdfc669e-6d9d-48bd-8f50-11c133a346c0

https://github.com/PolyMeilex/OxiSynth/assets/20758186/d50d2e95-f316-472f-88e9-111ed2f52784

### WASM as first class target

OxiSynth was built with WASM in mind from the get go, so here it is running in the browser, with Boomwhacker soundfont prebundled:
[https://oxisynth.netlify.app](https://oxisynth.netlify.app)
