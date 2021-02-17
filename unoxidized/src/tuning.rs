#[derive(Clone)]
pub struct Tuning {
    name: Vec<u8>,
    pub(crate) bank: i32,
    pub(crate) prog: i32,
    pub(crate) pitch: [f64; 128],
}

impl Tuning {
    pub fn new(name: &[u8], bank: i32, prog: i32) -> Tuning {
        let mut tuning = Tuning {
            name: name.to_vec(),
            bank,
            prog,
            pitch: [0f64; 128],
        };
        for i in 0..128 {
            tuning.pitch[i] = i as f64 * 100.0f64;
        }
        return tuning;
    }

    pub fn set_name(&mut self, name: &[u8]) {
        self.name = name.to_vec();
    }

    pub fn get_name(&self) -> &[u8] {
        return &self.name;
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

    pub fn set_pitch(&mut self, key: i32, pitch: f64) {
        if key >= 0 as i32 && key < 128 as i32 {
            self.pitch[key as usize] = pitch
        };
    }
}
