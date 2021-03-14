mod public;
pub use public::*;

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
    pub fn new(size: usize) -> Self {
        return Self {
            feedback: 0f32,
            filterstore: 0f32,
            damp1: 0f32,
            damp2: 0f32,
            buffer: vec![DC_OFFSET; size],
            bufidx: 0,
        };
    }

    pub fn set_damp(&mut self, val: f32) {
        self.damp1 = val;
        self.damp2 = 1f32 - val;
    }

    pub fn set_feedback(&mut self, val: f32) {
        self.feedback = val;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let mut _tmp = self.buffer[self.bufidx];
        self.filterstore = _tmp * self.damp2 + self.filterstore * self.damp1;
        self.buffer[self.bufidx] = input + self.filterstore * self.feedback;
        self.bufidx += 1;
        if self.bufidx >= self.buffer.len() {
            self.bufidx = 0
        }
        return _tmp;
    }
}

#[derive(Clone)]
struct AllPass {
    feedback: f32,
    buffer: Vec<f32>,
    bufidx: usize,
}

impl AllPass {
    pub fn new(size: usize, feedback: f32) -> Self {
        return Self {
            feedback,
            buffer: vec![DC_OFFSET; size],
            bufidx: 0,
        };
    }

    pub fn process(self: &mut Self, input: f32) -> f32 {
        let bufout: f32 = self.buffer[self.bufidx];
        let output: f32 = bufout - input;
        self.buffer[self.bufidx] = input + bufout * self.feedback;
        self.bufidx += 1;
        if self.bufidx >= self.buffer.len() {
            self.bufidx = 0
        }
        return output;
    }
}

#[derive(Clone)]
struct LRPair<T> {
    pub l: T,
    pub r: T,
}

#[derive(Clone)]
pub struct Reverb {
    pub(crate) active: bool,

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
    pub(crate) fn new(active: bool) -> Self {
        let mut rev = Self {
            active,

            roomsize: 0.5f32 * 0.28f32 + 0.7f32,
            damp: 0.2f32 * 1.0f32,
            wet: 1f32 * 3.0f32,
            wet1: 0f32,
            wet2: 0f32,
            width: 1f32,
            gain: 0.015f32,
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
        rev.update();
        return rev;
    }

    pub(crate) fn reset(self: &mut Self) {
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

    pub(crate) fn process_replace(&mut self, left_out: &mut [f32; 64], right_out: &mut [f32; 64]) {
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

    pub(crate) fn process_mix(
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

    pub(crate) fn update(&mut self) {
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
