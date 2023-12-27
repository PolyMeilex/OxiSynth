use byte_slice_cast::AsByteSlice;
use std::{fs::File, io::Write};

use oxisynth::{MidiEvent, OxiError, SoundFont, Synth};

fn main() {
    use env_logger::Env;
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    synth_sf2().unwrap();
}

fn synth_sf2() -> Result<(), OxiError> {
    let mut pcm = File::create("Out.sf2.pcm").unwrap();

    let mut synth = Synth::default();

    let mut file = File::open("./testdata/sin.sf2").unwrap();
    let font = SoundFont::load(&mut file).unwrap();
    synth.add_font(font, true);

    let mut samples = [0f32; 44100 / 8];

    for n in 50..100 {
        synth.send_event(MidiEvent::NoteOn {
            channel: 0,
            key: n,
            vel: 127,
        })?;

        synth.write(samples.as_mut());
        pcm.write_all(samples.as_byte_slice()).unwrap();

        synth.send_event(MidiEvent::NoteOff { channel: 0, key: n })?;
    }
    for n in 0..50 {
        synth.send_event(MidiEvent::NoteOn {
            channel: 0,
            key: 100 - n,
            vel: 127,
        })?;

        synth.write(samples.as_mut());
        pcm.write_all(samples.as_byte_slice()).unwrap();

        synth.send_event(MidiEvent::NoteOff {
            channel: 0,
            key: 100 - n,
        })?;
    }

    Ok(())
}
