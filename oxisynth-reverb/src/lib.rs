const DC_OFFSET: f32 = 1e-8;
const STEREO_SPREAD: usize = 23;

const COMBTUNING_L1: usize = 1116;
const COMBTUNING_R1: usize = 1116 + STEREO_SPREAD;
const COMBTUNING_L2: usize = 1188;
const COMBTUNING_R2: usize = 1188 + STEREO_SPREAD;
const COMBTUNING_L3: usize = 1277;
const COMBTUNING_R3: usize = 1277 + STEREO_SPREAD;
const COMBTUNING_L4: usize = 1356;
const COMBTUNING_R4: usize = 1356 + STEREO_SPREAD;
const COMBTUNING_L5: usize = 1422;
const COMBTUNING_R5: usize = 1422 + STEREO_SPREAD;
const COMBTUNING_L6: usize = 1491;
const COMBTUNING_R6: usize = 1491 + STEREO_SPREAD;
const COMBTUNING_L7: usize = 1557;
const COMBTUNING_R7: usize = 1557 + STEREO_SPREAD;
const COMBTUNING_L8: usize = 1617;
const COMBTUNING_R8: usize = 1617 + STEREO_SPREAD;
const ALLPASSTUNING_L1: usize = 556;
const ALLPASSTUNING_R1: usize = 556 + STEREO_SPREAD;
const ALLPASSTUNING_L2: usize = 441;
const ALLPASSTUNING_R2: usize = 441 + STEREO_SPREAD;
const ALLPASSTUNING_L3: usize = 341;
const ALLPASSTUNING_R3: usize = 341 + STEREO_SPREAD;
const ALLPASSTUNING_L4: usize = 225;
const ALLPASSTUNING_R4: usize = 225 + STEREO_SPREAD;

#[derive(Clone)]
struct Comb {
    feedback: f32,
    filterstore: f32,
    damp1: f32,
    damp2: f32,
    buffer: Vec<f32>,
    bufidx: usize,
}

impl Comb {
    fn new(size: usize) -> Self {
        Self {
            feedback: 0f32,
            filterstore: 0f32,
            damp1: 0f32,
            damp2: 0f32,
            buffer: vec![DC_OFFSET; size],
            bufidx: 0,
        }
    }

    fn set_damp(&mut self, val: f32) {
        self.damp1 = val;
        self.damp2 = 1f32 - val;
    }

    fn set_feedback(&mut self, val: f32) {
        self.feedback = val;
    }

    fn process(&mut self, input: f32) -> f32 {
        let mut _tmp = self.buffer[self.bufidx];
        self.filterstore = _tmp * self.damp2 + self.filterstore * self.damp1;
        self.buffer[self.bufidx] = input + self.filterstore * self.feedback;
        self.bufidx += 1;
        if self.bufidx >= self.buffer.len() {
            self.bufidx = 0
        }
        _tmp
    }
}

#[derive(Clone)]
struct AllPass {
    feedback: f32,
    buffer: Vec<f32>,
    bufidx: usize,
}

impl AllPass {
    fn new(size: usize, feedback: f32) -> Self {
        Self {
            feedback,
            buffer: vec![DC_OFFSET; size],
            bufidx: 0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let bufout: f32 = self.buffer[self.bufidx];
        let output: f32 = bufout - input;
        self.buffer[self.bufidx] = input + bufout * self.feedback;
        self.bufidx += 1;
        if self.bufidx >= self.buffer.len() {
            self.bufidx = 0
        }
        output
    }
}

#[derive(Clone)]
struct LRPair<T> {
    l: T,
    r: T,
}

#[derive(Clone)]
pub struct Reverb {
    active: bool,

    roomsize: f32,
    damp: f32,
    wet: f32,
    wet1: f32,
    wet2: f32,
    width: f32,
    gain: f32,
    comb: [LRPair<Comb>; 8],
    allpass: [LRPair<AllPass>; 4],
}

impl Reverb {
    pub fn new(active: bool) -> Self {
        let mut rev = Self {
            active,

            roomsize: 0.5 * 0.28 + 0.7,
            damp: 0.2 * 1.0,
            wet: 1.0 * 3.0,
            wet1: 0.0,
            wet2: 0.0,
            width: 1.0,
            gain: 0.015,
            comb: [
                LRPair {
                    l: Comb::new(COMBTUNING_L1),
                    r: Comb::new(COMBTUNING_R1),
                },
                LRPair {
                    l: Comb::new(COMBTUNING_L2),
                    r: Comb::new(COMBTUNING_R2),
                },
                LRPair {
                    l: Comb::new(COMBTUNING_L3),
                    r: Comb::new(COMBTUNING_R3),
                },
                LRPair {
                    l: Comb::new(COMBTUNING_L4),
                    r: Comb::new(COMBTUNING_R4),
                },
                LRPair {
                    l: Comb::new(COMBTUNING_L5),
                    r: Comb::new(COMBTUNING_R5),
                },
                LRPair {
                    l: Comb::new(COMBTUNING_L6),
                    r: Comb::new(COMBTUNING_R6),
                },
                LRPair {
                    l: Comb::new(COMBTUNING_L7),
                    r: Comb::new(COMBTUNING_R7),
                },
                LRPair {
                    l: Comb::new(COMBTUNING_L8),
                    r: Comb::new(COMBTUNING_R8),
                },
            ],
            allpass: [
                LRPair {
                    l: AllPass::new(ALLPASSTUNING_L1, 0.5f32),
                    r: AllPass::new(ALLPASSTUNING_R1, 0.5f32),
                },
                LRPair {
                    l: AllPass::new(ALLPASSTUNING_L2, 0.5f32),
                    r: AllPass::new(ALLPASSTUNING_R2, 0.5f32),
                },
                LRPair {
                    l: AllPass::new(ALLPASSTUNING_L3, 0.5f32),
                    r: AllPass::new(ALLPASSTUNING_R3, 0.5f32),
                },
                LRPair {
                    l: AllPass::new(ALLPASSTUNING_L4, 0.5f32),
                    r: AllPass::new(ALLPASSTUNING_R4, 0.5f32),
                },
            ],
        };
        rev.set_reverb(&Default::default());
        rev
    }

    pub fn reset(&mut self) {
        self.comb = [
            LRPair {
                l: Comb::new(COMBTUNING_L1),
                r: Comb::new(COMBTUNING_R1),
            },
            LRPair {
                l: Comb::new(COMBTUNING_L2),
                r: Comb::new(COMBTUNING_R2),
            },
            LRPair {
                l: Comb::new(COMBTUNING_L3),
                r: Comb::new(COMBTUNING_R3),
            },
            LRPair {
                l: Comb::new(COMBTUNING_L4),
                r: Comb::new(COMBTUNING_R4),
            },
            LRPair {
                l: Comb::new(COMBTUNING_L5),
                r: Comb::new(COMBTUNING_R5),
            },
            LRPair {
                l: Comb::new(COMBTUNING_L6),
                r: Comb::new(COMBTUNING_R6),
            },
            LRPair {
                l: Comb::new(COMBTUNING_L7),
                r: Comb::new(COMBTUNING_R7),
            },
            LRPair {
                l: Comb::new(COMBTUNING_L8),
                r: Comb::new(COMBTUNING_R8),
            },
        ];
        self.allpass = [
            LRPair {
                l: AllPass::new(ALLPASSTUNING_L1, 0.5f32),
                r: AllPass::new(ALLPASSTUNING_R1, 0.5f32),
            },
            LRPair {
                l: AllPass::new(ALLPASSTUNING_L2, 0.5f32),
                r: AllPass::new(ALLPASSTUNING_R2, 0.5f32),
            },
            LRPair {
                l: AllPass::new(ALLPASSTUNING_L3, 0.5f32),
                r: AllPass::new(ALLPASSTUNING_R3, 0.5f32),
            },
            LRPair {
                l: AllPass::new(ALLPASSTUNING_L4, 0.5f32),
                r: AllPass::new(ALLPASSTUNING_R4, 0.5f32),
            },
        ];
    }

    pub fn process_replace(&mut self, left_out: &mut [f32; 64], right_out: &mut [f32; 64]) {
        for k in 0..64 {
            let mut out_r = 0f32;
            let mut out_l = 0f32;

            // Don't ask me why only left buf is considered an input...
            let input = (2.0 * left_out[k] + DC_OFFSET) * self.gain;

            for comb in self.comb.iter_mut() {
                out_l += comb.l.process(input);
                out_r += comb.r.process(input);
            }

            for allpass in self.allpass.iter_mut() {
                out_l = allpass.l.process(out_l);
                out_r = allpass.r.process(out_r);
            }

            out_l -= DC_OFFSET;
            out_r -= DC_OFFSET;

            left_out[k] = out_l * self.wet1 + out_r * self.wet2;
            right_out[k] = out_r * self.wet1 + out_l * self.wet2;
        }
    }

    pub fn process_mix(
        &mut self,
        in_0: &mut [f32; 64],
        left_out: &mut [f32; 64],
        right_out: &mut [f32; 64],
    ) {
        for k in 0..64 {
            let mut out_r = 0f32;
            let mut out_l = out_r;
            let input = (2.0 * in_0[k] + DC_OFFSET) * self.gain;

            for comb in self.comb.iter_mut() {
                out_l += comb.l.process(input);
                out_r += comb.r.process(input);
            }

            for allpass in self.allpass.iter_mut() {
                out_l = allpass.l.process(out_l);
                out_r = allpass.r.process(out_r);
            }

            out_l -= DC_OFFSET;
            out_r -= DC_OFFSET;

            left_out[k] += out_l * self.wet1 + out_r * self.wet2;
            right_out[k] += out_r * self.wet1 + out_l * self.wet2;
        }
    }

    fn update(&mut self) {
        self.wet1 = self.wet * (self.width / 2f32 + 0.5f32);
        self.wet2 = self.wet * ((1f32 - self.width) / 2f32);
        for comb in self.comb.iter_mut() {
            comb.l.set_feedback(self.roomsize);
            comb.r.set_feedback(self.roomsize);
            comb.l.set_damp(self.damp);
            comb.r.set_damp(self.damp);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReverbParams {
    pub roomsize: f32,
    pub damp: f32,
    pub width: f32,
    pub level: f32,
}

impl Default for ReverbParams {
    fn default() -> Self {
        Self {
            roomsize: 0.2,
            damp: 0.0,
            width: 0.5,
            level: 0.9,
        }
    }
}

impl Reverb {
    /// Turn on/off the built-in Reverb unit
    pub fn set_active(&mut self, on: bool) {
        self.active = on;
    }

    /// Check if Reverb is on/off
    pub fn active(&self) -> bool {
        self.active
    }

    /// Set the current reverb room size
    fn set_room_size(&mut self, value: f32) {
        self.roomsize = value * 0.28 + 0.7;
    }

    /// Query the current reverb room size
    pub fn room_size(&self) -> f32 {
        (self.roomsize - 0.7) / 0.28
    }

    /// Set the current reverb dumping
    fn set_damp(&mut self, value: f32) {
        self.damp = value * 1.0;
    }

    /// Query the current reverb dumping
    pub fn damp(&self) -> f32 {
        self.damp / 1.0
    }

    /// Set the current reverb level
    fn set_level(&mut self, value: f32) {
        let value = if value < 0.0 {
            0.0
        } else if value > 1.0 {
            1.0
        } else {
            value
        };
        self.wet = value * 3.0;
    }

    /// Query the current reverb level
    pub fn level(&self) -> f32 {
        self.wet / 3.0
    }

    /// Set the current reverb width
    fn set_width(&mut self, value: f32) {
        self.width = value;
    }

    /// Query the current reverb width
    pub fn width(&self) -> f32 {
        self.width
    }
}

impl Reverb {
    /// Set the parameters for the built-in reverb unit
    pub fn set_reverb(&mut self, params: &ReverbParams) {
        self.set_reverb_params(params.roomsize, params.damp, params.width, params.level);
    }

    /// Set the parameters for the built-in reverb unit
    pub fn set_reverb_params(&mut self, roomsize: f32, damping: f32, width: f32, level: f32) {
        self.set_room_size(roomsize);
        self.set_damp(damping);
        self.set_width(width);
        self.set_level(level);
        self.update();
    }

    /// Query the current reverb params
    pub fn reverb(&self) -> ReverbParams {
        ReverbParams {
            roomsize: self.room_size(),
            damp: self.damp(),
            level: self.level(),
            width: self.width(),
        }
    }
}
