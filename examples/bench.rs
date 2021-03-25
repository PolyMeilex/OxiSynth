use oxisynth::{SoundFont, Synth};

fn main() {
    for _ in 0..100 {
        let mut file = std::fs::File::open("./testdata/sin.sf2").unwrap();

        // let start = std::time::Instant::now();
        // let data = SFData::load(&mut file).unwrap();
        // let _sf2 = SoundFont2::from_data(data);

        let mut synth = Synth::new(Default::default()).unwrap();

        let start = std::time::Instant::now();

        let font = SoundFont::load(&mut file).unwrap();
        synth.add_font(font, true);

        println!("All: {:?}", start.elapsed().as_millis());
    }
}
