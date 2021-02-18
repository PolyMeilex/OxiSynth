use oxidized_fluid as fluid;
use std::{fs::File, io::Write, slice::from_raw_parts};

fn main() {
    use env_logger::Env;
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    synth_sf2();
}

fn synth_sf2() {
    let mut pcm = File::create("Boomwhacker.sf2.pcm").unwrap();

    let settings = fluid::Settings::default();

    let mut synth = fluid::Synth::new(settings).unwrap();

    synth.sfload("./testdata/Boomwhacker.sf2", true).unwrap();

    let mut samples = [0f32; 44100 / 4];

    for n in 60..70 {
        synth.note_on(0, n, 127).unwrap();

        synth.write(samples.as_mut()).unwrap();
        pcm.write(unsafe {
            from_raw_parts(samples.as_ptr() as _, std::mem::size_of_val(&samples))
        })
        .unwrap();

        synth.note_off(0, n).unwrap();

        synth.write(samples.as_mut()).unwrap();
        pcm.write(unsafe {
            from_raw_parts(samples.as_ptr() as _, std::mem::size_of_val(&samples))
        })
        .unwrap();
    }

    drop(synth);
}
