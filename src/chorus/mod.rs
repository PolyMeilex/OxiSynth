mod public;
pub use public::*;

const MIN_SPEED_HZ: f32 = 0.29;

/* Length of one delay line in samples:
 * Set through MAX_SAMPLES_LN2.
 * For example:
 * MAX_SAMPLES_LN2=12
 * => MAX_SAMPLES=pow(2,12)=4096
 * => MAX_SAMPLES_ANDMASK=4095
 */
const MAX_SAMPLES_LN2: usize = 12;
const MAX_SAMPLES: usize = 1 << (MAX_SAMPLES_LN2 - 1);
const MAX_SAMPLES_ANDMASK: usize = MAX_SAMPLES - 1;

const INTERPOLATION_SUBSAMPLES_LN2: usize = 8;
const INTERPOLATION_SUBSAMPLES: usize = 1 << (INTERPOLATION_SUBSAMPLES_LN2 - 1);
const INTERPOLATION_SUBSAMPLES_ANDMASK: usize = INTERPOLATION_SUBSAMPLES - 1;

const INTERPOLATION_SAMPLES: usize = 5;

#[derive(Clone)]
pub struct Chorus {
    active: bool,

    type_0: ChorusMode,
    new_type: ChorusMode,
    depth_ms: f32,
    new_depth_ms: f32,
    level: f32,
    new_level: f32,
    speed_hz: f32,
    new_speed_hz: f32,
    number_blocks: u32,
    new_number_blocks: u32,
    chorusbuf: [f32; MAX_SAMPLES],
    counter: i32,
    phase: [isize; 99],
    modulation_period_samples: isize,
    lookup_tab: Vec<i32>,
    sample_rate: f32,
    sinc_table: [[f32; 128]; 5],
}

impl Chorus {
    pub(crate) fn new(sample_rate: f32, active: bool) -> Self {
        let mut chorus = Self {
            active,

            type_0: ChorusMode::Sine,
            new_type: ChorusMode::Sine,
            depth_ms: 0f32,
            new_depth_ms: 0f32,
            level: 0f32,
            new_level: 0f32,
            speed_hz: 0f32,
            new_speed_hz: 0f32,
            number_blocks: 0,
            new_number_blocks: 0,
            chorusbuf: [0.0; MAX_SAMPLES],
            counter: 0,
            phase: [0; 99],
            modulation_period_samples: 0,
            lookup_tab: vec![0; (sample_rate / MIN_SPEED_HZ) as usize],
            sample_rate,
            sinc_table: [[0f32; 128]; 5],
        };
        for i in 0..INTERPOLATION_SAMPLES {
            for ii in 0..INTERPOLATION_SUBSAMPLES {
                // Move the origin into the center of the table
                let i_shifted = i as f32 - INTERPOLATION_SAMPLES as f32 / 2.0
                    + ii as f32 / INTERPOLATION_SUBSAMPLES as f32;

                if i_shifted.abs() < 0.000001 {
                    // sinc(0) cannot be calculated straightforward (limit needed for 0/0)
                    chorus.sinc_table[i][ii] = 1.0;
                } else {
                    chorus.sinc_table[i][ii] = (i_shifted * std::f32::consts::PI).sin()
                        / (std::f32::consts::PI * i_shifted);
                    // Hamming window
                    chorus.sinc_table[i][ii] *=
                        0.5 * (1.0 + (2.0 * std::f32::consts::PI * i_shifted / 5.0).cos());
                }
            }
        }
        chorus.init();

        chorus
    }

    fn init(&mut self) {
        self.chorusbuf.fill(0.0);
        self.set_chorus(&Default::default());
        self.update();
    }

    fn update(&mut self) {
        if self.new_number_blocks > 99 {
            log::warn!(
                "chorus: number blocks larger than max. allowed! Setting value to {}.",
                99
            );
            self.new_number_blocks = 99;
        }
        if self.new_speed_hz < 0.29 {
            log::warn!(
                "chorus: speed is too low (min {})! Setting value to min.",
                0.29
            );
            self.new_speed_hz = 0.29;
        } else if self.new_speed_hz > 5.0 {
            log::warn!(
                "chorus: speed must be below {} Hz! Setting value to max.",
                5
            );
            self.new_speed_hz = 5.0;
        }
        if self.new_depth_ms < 0.0 {
            log::warn!("chorus: depth must be positive! Setting value to 0.",);
            self.new_depth_ms = 0.0;
        }
        if self.new_level < 0.0 {
            log::warn!("chorus: level must be positive! Setting value to 0.",);
            self.new_level = 0.0;
        } else if self.new_level > 10.0 {
            log::warn!(
                "chorus: level must be < 10. A reasonable level is << 1! Setting it to 0.1.",
            );
            self.new_level = 0.1;
        }
        self.modulation_period_samples = (self.sample_rate / self.new_speed_hz) as isize;

        let mut modulation_depth_samples = (self.new_depth_ms / 1000.0 * self.sample_rate) as i32;
        if modulation_depth_samples > MAX_SAMPLES as i32 {
            log::warn!(
                "chorus: Too high depth. Setting it to max ({}).",
                MAX_SAMPLES,
            );
            modulation_depth_samples = MAX_SAMPLES as i32;
        }
        if self.type_0 == ChorusMode::Sine {
            modulate_sine(
                &mut self.lookup_tab,
                self.modulation_period_samples as usize,
                modulation_depth_samples,
            );
        } else if self.type_0 == ChorusMode::Triangle {
            modulate_triangle(
                &mut self.lookup_tab,
                self.modulation_period_samples as i32,
                modulation_depth_samples,
            );
        } else {
            log::warn!("chorus: Unknown modulation type. Using sinewave.",);
            self.type_0 = ChorusMode::Sine;
            modulate_sine(
                &mut self.lookup_tab,
                self.modulation_period_samples as usize,
                modulation_depth_samples,
            );
        }
        for i in 0..(self.number_blocks as usize) {
            self.phase[i] = (self.modulation_period_samples as f64 * i as f64
                / self.number_blocks as f64) as isize;
        }
        self.counter = 0;
        self.type_0 = self.new_type;
        self.depth_ms = self.new_depth_ms;
        self.level = self.new_level;
        self.speed_hz = self.new_speed_hz;
        self.number_blocks = self.new_number_blocks;
    }

    pub(crate) fn process_mix(
        &mut self,
        in_0: &mut [f32; 64],
        left_out: &mut [f32; 64],
        right_out: &mut [f32; 64],
    ) {
        for sample_index in 0..64 {
            let d_in = in_0[sample_index];
            let mut d_out = 0.0;
            self.chorusbuf[self.counter as usize] = d_in;

            for i in 0..(self.number_blocks as usize) {
                // Calculate the delay in subsamples for the delay line of chorus block nr. */
                //
                // The value in the lookup table is so, that this expression
                // will always be positive.  It will always include a number of
                // full periods of MAX_SAMPLES*INTERPOLATION_SUBSAMPLES to
                // remain positive at all times. */
                let mut pos_subsamples = INTERPOLATION_SUBSAMPLES as i32 * self.counter
                    - self.lookup_tab[self.phase[i] as usize];

                let mut pos_samples = pos_subsamples / INTERPOLATION_SUBSAMPLES as i32;
                pos_subsamples &= INTERPOLATION_SUBSAMPLES_ANDMASK as i32;

                for ii in 0..5 {
                    d_out += self.chorusbuf[(pos_samples & MAX_SAMPLES_ANDMASK as i32) as usize]
                        * self.sinc_table[ii as usize][pos_subsamples as usize];
                    pos_samples -= 1;
                }

                self.phase[i] += 1;
                self.phase[i] %= self.modulation_period_samples;
            }

            d_out *= self.level;

            left_out[sample_index] += d_out;
            right_out[sample_index] += d_out;

            self.counter += 1;
            self.counter %= MAX_SAMPLES as i32;
        }
    }

    pub(crate) fn process_replace(&mut self, left_out: &mut [f32; 64], right_out: &mut [f32; 64]) {
        for sample_index in 0..64 {
            // Don't ask me why only left buf is considered an input...
            let d_in = left_out[sample_index];
            let mut d_out = 0.0;

            self.chorusbuf[self.counter as usize] = d_in;

            for i in 0..(self.number_blocks as usize) {
                let mut pos_subsamples = INTERPOLATION_SUBSAMPLES as i32 * self.counter
                    - self.lookup_tab[self.phase[i] as usize];

                let mut pos_samples = pos_subsamples / INTERPOLATION_SUBSAMPLES as i32;
                pos_subsamples &= INTERPOLATION_SUBSAMPLES_ANDMASK as i32;

                for ii in 0..INTERPOLATION_SAMPLES {
                    d_out += self.chorusbuf[(pos_samples & MAX_SAMPLES_ANDMASK as i32) as usize]
                        * self.sinc_table[ii][pos_subsamples as usize];
                    pos_samples -= 1;
                }

                self.phase[i] += 1;
                self.phase[i] %= self.modulation_period_samples;
            }
            d_out *= self.level;

            left_out[sample_index] = d_out;
            right_out[sample_index] = d_out;

            self.counter += 1;
            self.counter %= MAX_SAMPLES as i32;
        }
    }

    pub(crate) fn reset(&mut self) {
        self.init();
    }
}

fn modulate_sine(buf: &mut [i32], len: usize, depth: i32) {
    for (i, out) in buf.iter_mut().take(len).enumerate() {
        let val = f64::sin(i as f64 / len as f64 * 2.0 * std::f64::consts::PI);
        *out = ((1.0 + val) * depth as f64 / 2.0 * INTERPOLATION_SUBSAMPLES as f64) as i32;
        *out -= 3 * MAX_SAMPLES as i32 * INTERPOLATION_SUBSAMPLES as i32;
    }
}

fn modulate_triangle(buf: &mut [i32], len: i32, depth: i32) {
    let mut i = 0;
    let mut ii = len - 1;
    while i <= ii {
        let val = i as f64 * 2.0 / len as f64 * depth as f64 * INTERPOLATION_SUBSAMPLES as f64;
        let val2 = (val + 0.5) as i32 - 3 * MAX_SAMPLES as i32 * INTERPOLATION_SUBSAMPLES as i32;
        let fresh2 = i;
        i += 1;
        buf[fresh2 as usize] = val2;
        let fresh3 = ii;
        ii -= 1;
        buf[fresh3 as usize] = val2;
    }
}
