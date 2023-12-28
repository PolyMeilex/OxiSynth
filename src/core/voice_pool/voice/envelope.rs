#[derive(Copy, Clone, Default)]
pub struct Envelope([EnvelopePortion; 7]);

impl std::ops::Index<EnvelopeStep> for Envelope {
    type Output = EnvelopePortion;

    fn index(&self, index: EnvelopeStep) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl std::ops::IndexMut<EnvelopeStep> for Envelope {
    fn index_mut(&mut self, index: EnvelopeStep) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

#[derive(Copy, Default, Clone)]
pub struct EnvelopePortion {
    pub count: u32,
    pub coeff: f32,
    pub incr: f32,
    pub min: f32,
    pub max: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EnvelopeStep {
    Delay,
    Attack,
    Hold,
    Decay,
    Sustain,
    Release,
    Finished,
}

impl EnvelopeStep {
    pub fn next(&mut self) {
        *self = match self {
            Self::Delay => Self::Attack,
            Self::Attack => Self::Hold,
            Self::Hold => Self::Decay,
            Self::Decay => Self::Sustain,
            Self::Sustain => Self::Release,
            Self::Release => Self::Finished,
            Self::Finished => Self::Finished,
        }
    }
}
