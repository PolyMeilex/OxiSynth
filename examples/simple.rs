use oxidized_fluid as fluid;
use std::{fs::File, io::Write, slice::from_raw_parts};

fn main() {
    use env_logger::Env;
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    synth_sf2();
}

fn synth_sf2() {
    let mut pcm = File::create("Out.sf2.pcm").unwrap();

    let settings = fluid::Settings::default();

    let mut synth = fluid::Synth::new(settings);

    synth.sfload("./testdata/sin.sf2", true).unwrap();

    let mut samples = [0f32; 44100 / 8];

    for _ in 0..5 {
        for n in 50..100 {
            synth.note_on(0, n, 127).unwrap();

            synth.write(samples.as_mut()).unwrap();
            pcm.write(unsafe {
                from_raw_parts(samples.as_ptr() as _, std::mem::size_of_val(&samples))
            })
            .unwrap();

            synth.note_off(0, n).unwrap();
        }
    }

    drop(synth);
}
