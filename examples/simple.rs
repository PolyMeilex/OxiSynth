use std::{fs::File, io::Write, slice::from_raw_parts};

fn main() {
    use env_logger::Env;
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    synth_sf2();
}

fn synth_sf2() {
    let mut pcm = File::create("Out.sf2.pcm").unwrap();

    let settings = oxisynth::Settings::default();

    let mut synth = oxisynth::Synth::new(settings);

    synth.sfload("./testdata/sin.sf2", true).unwrap();

    let mut samples = [0f32; 44100 / 8];

    for _ in 0..5 {
        for n in 50..100 {
            synth.note_on(0, n, 127).unwrap();

            synth.write(samples.as_mut());

            pcm.write(unsafe {
                from_raw_parts(samples.as_ptr() as _, std::mem::size_of_val(&samples))
            })
            .unwrap();

            // Here is safe write (if you prefere)
            // {
            // let mut out = [0u8; (44100 / 8) * 4];
            // for (id, v) in samples.iter().enumerate() {
            //     let b = v.to_le_bytes();
            //     let id = id * 4;
            //     out[id] = b[0];
            //     out[id + 1] = b[1];
            //     out[id + 2] = b[2];
            //     out[id + 3] = b[3];
            // }
            // pcm.write(&out).unwrap();
            // }

            synth.note_off(0, n).unwrap();
        }
        for n in 0..50 {
            synth.note_on(0, 100 - n, 127).unwrap();

            synth.write(samples.as_mut());
            pcm.write(unsafe {
                from_raw_parts(samples.as_ptr() as _, std::mem::size_of_val(&samples))
            })
            .unwrap();

            synth.note_off(0, 100 - n).unwrap();
        }
    }

    drop(synth);
}
