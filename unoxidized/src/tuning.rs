#![forbid(unsafe_code)]

#[derive(Clone)]
pub struct Tuning {
    name: String,
    pub(crate) bank: u32,
    pub(crate) prog: u32,
    pub(crate) pitch: [f64; 128],
}

impl Tuning {
    pub fn new(name: String, bank: u32, prog: u32) -> Tuning {
        let mut tuning = Tuning {
            name,
            bank,
            prog,
            pitch: [0f64; 128],
        };
        for i in 0..128 {
            tuning.pitch[i] = i as f64 * 100.0f64;
        }
        return tuning;
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_octave(&mut self, pitch_deriv: &[f64; 12]) {
        let mut i;
        i = 0 as i32;
        while i < 128 as i32 {
            self.pitch[i as usize] = i as f64 * 100.0f64 + pitch_deriv[i as usize % 12];
            i += 1
        }
    }

    pub fn set_all(&mut self, pitch: &[f64; 128]) {
        for i in 0..128 {
            self.pitch[i] = pitch[i];
        }
    }

    pub fn set_pitch(&mut self, key: u32, pitch: f64) {
        if key < 128 {
            self.pitch[key as usize] = pitch
        };
    }
}
