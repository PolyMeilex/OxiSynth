#![feature(test)]
extern crate test;

use test::Bencher;

#[bench]
fn only_parse(b: &mut Bencher) {
    b.iter(|| {
        let mut file = std::fs::File::open("./testdata/test.sf2").unwrap();
        let data = soundfont::data::SFData::load(&mut file).unwrap();
        let mut sf2 = soundfont::SoundFont2::from_data(data);
        sf2.sort_presets();
    });
}

#[bench]
fn full(b: &mut Bencher) {
    b.iter(|| {
        let mut file = std::fs::File::open("./testdata/test.sf2").unwrap();
        let _font = oxisynth::SoundFont::load(&mut file).unwrap();
    });
}
