fn main() {
    for _ in 0..100 {
        let mut file = std::fs::File::open("./testdata/sin.sf2").unwrap();

        // let start = std::time::Instant::now();
        // let data = SFData::load(&mut file).unwrap();
        // let _sf2 = SoundFont2::from_data(data);

        let mut synth = oxisynth::Synth::new(Default::default()).unwrap();

        let start = std::time::Instant::now();
        synth.sfload(&mut file, true).unwrap();
        println!("All: {:?}", start.elapsed().as_millis());
    }
}
