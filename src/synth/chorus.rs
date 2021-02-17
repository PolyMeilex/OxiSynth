use crate::{engine, Synth};

type ChorusMode = engine::chorus::ChorusMode;

/**
Chorus parameters
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChorusParams {
    pub nr: u32,
    pub level: f64,
    /// Speed in Hz
    pub speed: f64,
    /// Depth in mS
    pub depth: f64,
    /// Mode
    pub mode: ChorusMode,
}

impl Default for ChorusParams {
    fn default() -> Self {
        Self {
            nr: 3,
            level: 2.0,
            speed: 0.3,
            depth: 8.0,
            mode: ChorusMode::default(),
        }
    }
}

/**
Chorus
 */
impl Synth {
    /**
    Set up the chorus. It should be turned on with Synth::chorus_on().
    If faulty parameters are given, all new settings are discarded.
    Keep in mind, that the needed CPU time is proportional to `nr`.
     */
    pub fn set_chorus_params(
        &mut self,
        nr: u32,
        level: f64,
        speed: f64,
        depth: f64,
        mode: ChorusMode,
    ) {
        self.handle
            .set_chorus_params(nr as i32, level, speed, depth, mode);
    }

    /**
    Set up the chorus. It should be turned on with Synth::chorus_on().
    If faulty parameters are given, all new settings are discarded.
    Keep in mind, that the needed CPU time is proportional to `nr`.
     */
    pub fn set_chorus(&mut self, params: &ChorusParams) {
        self.set_chorus_params(
            params.nr,
            params.level,
            params.speed,
            params.depth,
            params.mode,
        );
    }

    /** Turn on/off the built-in chorus unit */
    pub fn set_chorus_on(&mut self, on: bool) {
        self.handle.set_chorus_on(on as _);
    }

    /**
    Query the current chorus nr
     */
    pub fn get_chorus_nr(&self) -> u32 {
        self.handle.get_chorus_nr() as _
    }

    /**
    Query the current chorus level
     */
    pub fn get_chorus_level(&self) -> f64 {
        self.handle.get_chorus_level() as _
    }

    /**
    Query the current chorus speed (Hz)
     */
    pub fn get_chorus_speed(&self) -> f64 {
        self.handle.get_chorus_speed_hz() as _
    }

    /**
    Query the current chorus depth (mS)
     */
    pub fn get_chorus_depth(&self) -> f64 {
        self.handle.get_chorus_depth_ms() as _
    }

    /**
    Query the current chorus mode
     */
    pub fn get_chorus_mode(&self) -> ChorusMode {
        self.handle.get_chorus_type()
    }

    /**
    Query the current chorus params
     */
    pub fn get_chorus(&self) -> ChorusParams {
        ChorusParams {
            nr: self.get_chorus_nr(),
            level: self.get_chorus_level(),
            speed: self.get_chorus_speed(),
            depth: self.get_chorus_depth(),
            mode: self.get_chorus_mode(),
        }
    }
}
