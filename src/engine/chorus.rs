/**
Chorus type
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ChorusMode {
    Sine = 0,
    Triangle = 1,
}

impl Default for ChorusMode {
    fn default() -> Self {
        ChorusMode::Sine
    }
}

#[derive(Copy, Clone)]
pub struct Chorus {
    type_0: ChorusMode,
    new_type: ChorusMode,
    depth_ms: f32,
    new_depth_ms: f32,
    level: f32,
    new_level: f32,
    speed_hz: f32,
    new_speed_hz: f32,
    number_blocks: i32,
    new_number_blocks: i32,
    chorusbuf: *mut f32,
    counter: i32,
    phase: [isize; 99],
    modulation_period_samples: isize,
    lookup_tab: *mut i32,
    sample_rate: f32,
    sinc_table: [[f32; 128]; 5],
}

impl Chorus {
    pub fn new(sample_rate: f32) -> Self {
        unsafe {
            let mut chorus = Self {
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
                chorusbuf: libc::malloc(
                    (((1 as i32) << 12 as i32 - 1 as i32) as libc::size_t)
                        .wrapping_mul(::std::mem::size_of::<f32>() as libc::size_t),
                ) as *mut f32,
                counter: 0,
                phase: [0; 99],
                modulation_period_samples: 0,
                lookup_tab: libc::malloc(
                    ((sample_rate as f64 / 0.29f64) as i32 as libc::size_t)
                        .wrapping_mul(::std::mem::size_of::<i32>() as libc::size_t),
                ) as *mut i32,
                sample_rate,
                sinc_table: [[0f32; 128]; 5],
            };
            let mut i;
            i = 0 as i32;
            while i < 5 as i32 {
                let mut ii;
                ii = 0 as i32;
                while ii < (1 as i32) << 8 as i32 - 1 as i32 {
                    let i_shifted: f64 = i as f64 - 5 as i32 as f64 / 2.0f64
                        + ii as f64 / ((1 as i32) << 8 as i32 - 1 as i32) as f64;
                    if f64::abs(i_shifted) < 0.000001f64 {
                        chorus.sinc_table[i as usize][ii as usize] = 1.0f32
                    } else {
                        chorus.sinc_table[i as usize][ii as usize] =
                            (f64::sin(i_shifted * std::f64::consts::PI) as f32 as f64
                                / (std::f64::consts::PI * i_shifted))
                                as f32;
                        chorus.sinc_table[i as usize][ii as usize] =
                            (chorus.sinc_table[i as usize][ii as usize] as f64
                                * (0.5f64
                                    * (1.0f64
                                        + f64::cos(
                                            2.0f64 * std::f64::consts::PI * i_shifted
                                                / 5 as i32 as f32 as f64,
                                        )))) as f32
                    }
                    ii += 1
                }
                i += 1
            }
            if chorus.lookup_tab.is_null() {
                panic!("out of memory");
            } else if chorus.chorusbuf.is_null() {
                panic!("out of memory");
            }
            chorus.init();
            return chorus;
        }
    }

    pub fn init(&mut self) {
        let mut i: i32 = 0;
        unsafe {
            while i < (1 as i32) << 12 as i32 - 1 as i32 {
                *self.chorusbuf.offset(i as isize) = 0.0f32;
                i += 1
            }
        }
        self.set_nr(3 as i32);
        self.set_level(2.0f32);
        self.set_speed_hz(0.3f32);
        self.set_depth_ms(8.0f32);
        self.set_type(ChorusMode::Sine);
        self.update();
    }

    pub fn set_nr(&mut self, nr: i32) {
        self.new_number_blocks = nr;
    }

    pub fn get_nr(&self) -> i32 {
        return self.number_blocks;
    }

    pub fn set_level(&mut self, level: f32) {
        self.new_level = level;
    }

    pub fn get_level(&self) -> f32 {
        return self.level;
    }

    pub fn set_speed_hz(&mut self, speed_hz: f32) {
        self.new_speed_hz = speed_hz;
    }

    pub fn get_speed_hz(&self) -> f32 {
        return self.speed_hz;
    }

    pub fn set_depth_ms(&mut self, depth_ms: f32) {
        self.new_depth_ms = depth_ms;
    }

    pub fn get_depth_ms(&self) -> f32 {
        return self.depth_ms;
    }

    pub fn set_type(&mut self, type_0: ChorusMode) {
        self.new_type = type_0;
    }

    pub fn get_type(&self) -> ChorusMode {
        return self.type_0;
    }

    pub fn delete(&mut self) {
        if !self.chorusbuf.is_null() {
            unsafe {
                libc::free(self.chorusbuf as *mut libc::c_void);
            }
        }
        if !self.lookup_tab.is_null() {
            unsafe {
                libc::free(self.lookup_tab as *mut libc::c_void);
            }
        }
    }

    pub fn update(&mut self) {
        let mut i: i32;
        let mut modulation_depth_samples: i32;
        if self.new_number_blocks < 0 as i32 {
            fluid_log!(
                FLUID_WARN,
                "chorus: number blocks must be >=0! Setting value to 0.",
            );
            self.new_number_blocks = 0 as i32
        } else if self.new_number_blocks > 99 as i32 {
            fluid_log!(
                FLUID_WARN,
                "chorus: number blocks larger than max. allowed! Setting value to {}.",
                99
            );
            self.new_number_blocks = 99 as i32
        }
        if (self.new_speed_hz as f64) < 0.29f64 {
            fluid_log!(
                FLUID_WARN,
                "chorus: speed is too low (min {})! Setting value to min.",
                0.29f64
            );
            self.new_speed_hz = 0.29f32
        } else if self.new_speed_hz > 5 as i32 as f32 {
            fluid_log!(
                FLUID_WARN,
                "chorus: speed must be below {} Hz! Setting value to max.",
                5
            );
            self.new_speed_hz = 5 as i32 as f32
        }
        if (self.new_depth_ms as f64) < 0.0f64 {
            fluid_log!(
                FLUID_WARN,
                "chorus: depth must be positive! Setting value to 0.",
            );
            self.new_depth_ms = 0.0f32
        }
        if (self.new_level as f64) < 0.0f64 {
            fluid_log!(
                FLUID_WARN,
                "chorus: level must be positive! Setting value to 0.",
            );
            self.new_level = 0.0f32
        } else if self.new_level > 10 as i32 as f32 {
            fluid_log!(
                FLUID_WARN,
                "chorus: level must be < 10. A reasonable level is << 1! Setting it to 0.1.",
            );
            self.new_level = 0.1f32
        }
        self.modulation_period_samples = (self.sample_rate / self.new_speed_hz) as isize;
        modulation_depth_samples =
            (self.new_depth_ms as f64 / 1000.0f64 * self.sample_rate as f64) as i32;
        if modulation_depth_samples > (1 as i32) << 12 as i32 - 1 as i32 {
            fluid_log!(
                FLUID_WARN,
                "chorus: Too high depth. Setting it to max ({}).",
                (1) << 12 - 1
            );
            modulation_depth_samples = (1 as i32) << 12 as i32 - 1 as i32
        }
        if self.type_0 == ChorusMode::Sine {
            modulate_sine(
                self.lookup_tab,
                self.modulation_period_samples as i32,
                modulation_depth_samples,
            );
        } else if self.type_0 == ChorusMode::Triangle {
            modulate_triangle(
                self.lookup_tab,
                self.modulation_period_samples as i32,
                modulation_depth_samples,
            );
        } else {
            fluid_log!(
                FLUID_WARN,
                "chorus: Unknown modulation type. Using sinewave.",
            );
            self.type_0 = ChorusMode::Sine;
            modulate_sine(
                self.lookup_tab,
                self.modulation_period_samples as i32,
                modulation_depth_samples,
            );
        }
        i = 0 as i32;
        while i < self.number_blocks {
            self.phase[i as usize] = (self.modulation_period_samples as f64 * i as f64
                / self.number_blocks as f64) as i32 as isize;
            i += 1
        }
        self.counter = 0 as i32;
        self.type_0 = self.new_type;
        self.depth_ms = self.new_depth_ms;
        self.level = self.new_level;
        self.speed_hz = self.new_speed_hz;
        self.number_blocks = self.new_number_blocks;
    }

    pub fn process_mix(&mut self, in_0: *mut f32, left_out: *mut f32, right_out: *mut f32) {
        unsafe {
            let mut sample_index: i32;
            let mut i: i32;
            let mut d_in: f32;
            let mut d_out: f32;
            sample_index = 0 as i32;
            while sample_index < 64 as i32 {
                d_in = *in_0.offset(sample_index as isize);
                d_out = 0.0f32;
                *self.chorusbuf.offset(self.counter as isize) = d_in;
                i = 0 as i32;
                while i < self.number_blocks {
                    let mut ii: i32;
                    let mut pos_subsamples: i32 = ((1 as i32) << 8 as i32 - 1 as i32)
                        * self.counter
                        - *self.lookup_tab.offset(self.phase[i as usize] as isize);
                    let mut pos_samples: i32 = pos_subsamples / ((1 as i32) << 8 as i32 - 1 as i32);
                    pos_subsamples &= ((1 as i32) << 8 as i32 - 1 as i32) - 1 as i32;
                    ii = 0 as i32;
                    while ii < 5 as i32 {
                        d_out += *self.chorusbuf.offset(
                            (pos_samples & ((1 as i32) << 12 as i32 - 1 as i32) - 1 as i32)
                                as isize,
                        ) * self.sinc_table[ii as usize][pos_subsamples as usize];
                        pos_samples -= 1;
                        ii += 1
                    }
                    self.phase[i as usize] += 1;
                    self.phase[i as usize] %= self.modulation_period_samples;
                    i += 1
                }
                d_out *= self.level;
                let ref mut fresh0 = *left_out.offset(sample_index as isize);
                *fresh0 += d_out;
                let ref mut fresh1 = *right_out.offset(sample_index as isize);
                *fresh1 += d_out;
                self.counter += 1;
                self.counter %= (1 as i32) << 12 as i32 - 1 as i32;
                sample_index += 1
            }
        }
    }

    pub fn process_replace(&mut self, in_0: *mut f32, left_out: *mut f32, right_out: *mut f32) {
        unsafe {
            let mut sample_index: i32;
            let mut i: i32;
            let mut d_in: f32;
            let mut d_out: f32;
            sample_index = 0 as i32;
            while sample_index < 64 as i32 {
                d_in = *in_0.offset(sample_index as isize);
                d_out = 0.0f32;
                *self.chorusbuf.offset(self.counter as isize) = d_in;
                i = 0 as i32;
                while i < self.number_blocks {
                    let mut ii: i32;
                    let mut pos_subsamples: i32 = ((1 as i32) << 8 as i32 - 1 as i32)
                        * self.counter
                        - *self.lookup_tab.offset(self.phase[i as usize] as isize);
                    let mut pos_samples: i32 = pos_subsamples / ((1 as i32) << 8 as i32 - 1 as i32);
                    pos_subsamples &= ((1 as i32) << 8 as i32 - 1 as i32) - 1 as i32;
                    ii = 0 as i32;
                    while ii < 5 as i32 {
                        d_out += *self.chorusbuf.offset(
                            (pos_samples & ((1 as i32) << 12 as i32 - 1 as i32) - 1 as i32)
                                as isize,
                        ) * self.sinc_table[ii as usize][pos_subsamples as usize];
                        pos_samples -= 1;
                        ii += 1
                    }
                    self.phase[i as usize] += 1;
                    self.phase[i as usize] %= self.modulation_period_samples;
                    i += 1
                }
                d_out *= self.level;
                *left_out.offset(sample_index as isize) = d_out;
                *right_out.offset(sample_index as isize) = d_out;
                self.counter += 1;
                self.counter %= (1 as i32) << 12 as i32 - 1 as i32;
                sample_index += 1
            }
        }
    }

    pub fn reset(&mut self) {
        self.init();
    }
}

fn modulate_sine(buf: *mut i32, len: i32, depth: i32) {
    unsafe {
        let mut i: i32;
        let mut val: f64;
        i = 0 as i32;
        while i < len {
            val = f64::sin(i as f64 / len as f64 * 2.0f64 * std::f64::consts::PI);
            *buf.offset(i as isize) = ((1.0f64 + val) * depth as f64 / 2.0f64
                * ((1 as i32) << 8 as i32 - 1 as i32) as f64)
                as i32;
            *buf.offset(i as isize) -= 3 as i32
                * ((1 as i32) << 12 as i32 - 1 as i32)
                * ((1 as i32) << 8 as i32 - 1 as i32);
            i += 1
        }
    }
}

fn modulate_triangle(buf: *mut i32, len: i32, depth: i32) {
    unsafe {
        let mut i: i32 = 0 as i32;
        let mut ii: i32 = len - 1 as i32;
        let mut val: f64;
        let mut val2: f64;
        while i <= ii {
            val = i as f64 * 2.0f64 / len as f64
                * depth as f64
                * ((1 as i32) << 8 as i32 - 1 as i32) as f64;
            val2 = ((val + 0.5f64) as i32
                - 3 as i32
                    * ((1 as i32) << 12 as i32 - 1 as i32)
                    * ((1 as i32) << 8 as i32 - 1 as i32)) as f64;
            let fresh2 = i;
            i = i + 1;
            *buf.offset(fresh2 as isize) = val2 as i32;
            let fresh3 = ii;
            ii = ii - 1;
            *buf.offset(fresh3 as isize) = val2 as i32
        }
    }
}
