use oxidized_fluid as fluid;

use fluid::IsSettings;

fn main() {
    let mut settings = fluid::Settings::new().unwrap();
    // let gain = settings.num("synth.gain").unwrap();

    let rate = settings.pick::<_, f64>("synth.sample-rate").unwrap();

    let g = rate.get().unwrap();
    println!("{}", g);

    rate.set(0.0);

    let g = rate.get().unwrap();
    println!("{}", g);

    // assert_eq!(gain.default(), 0.2f32 as f64);
    // //assert_eq!(gain.range().min, Some(0.0));
    // //assert_eq!(gain.range().max, Some(10.0));

    // assert_eq!(gain.get(), Some(0.2f32 as f64));
    // assert!(gain.set(0.5));
    // assert_eq!(gain.get(), Some(0.5));
}
