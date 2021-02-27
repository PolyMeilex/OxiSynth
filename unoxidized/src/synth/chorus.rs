#![forbid(unsafe_code)]

use crate::synth::ChorusMode;
use crate::synth::Synth;

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

impl Synth {
    /**
    Set up the chorus. It should be turned on with Synth::chorus_on().
    If faulty parameters are given, all new settings are discarded.
    Keep in mind, that the needed CPU time is proportional to `nr`.
     */
    pub fn set_chorus(&mut self, params: &ChorusParams) {
        self.set_chorus_params(
            params.nr as i32,
            params.level,
            params.speed,
            params.depth,
            params.mode,
        );
    }

    /**
    Set up the chorus. It should be turned on with Synth::chorus_on().
    If faulty parameters are given, all new settings are discarded.
    Keep in mind, that the needed CPU time is proportional to `nr`.
     */
    pub fn set_chorus_params(
        &mut self,
        nr: i32,
        level: f64,
        speed: f64,
        depth_ms: f64,
        type_0: ChorusMode,
    ) {
        self.chorus.set_nr(nr);
        self.chorus.set_level(level as f32);
        self.chorus.set_speed_hz(speed as f32);
        self.chorus.set_depth_ms(depth_ms as f32);
        self.chorus.set_mode(type_0);
        self.chorus.update();
    }

    /** Turn on/off the built-in chorus unit */
    pub fn set_chorus_on(&mut self, on: bool) {
        self.settings.synth.chorus_active = on;
    }

    /**
    Query the current chorus nr
     */
    pub fn get_chorus_nr(&self) -> i32 {
        self.chorus.get_nr()
    }

    /**
    Query the current chorus level
     */
    pub fn get_chorus_level(&self) -> f64 {
        self.chorus.get_level() as f64
    }

    /**
    Query the current chorus speed (Hz)
     */
    pub fn get_chorus_speed_hz(&self) -> f64 {
        self.chorus.get_speed_hz() as f64
    }

    /**
    Query the current chorus depth (mS)
     */
    pub fn get_chorus_depth_ms(&self) -> f64 {
        self.chorus.get_depth_ms() as f64
    }

    /**
    Query the current chorus mode
     */
    pub fn get_chorus_mode(&self) -> ChorusMode {
        self.chorus.get_mode()
    }

    /**
    Query the current chorus params
     */
    pub fn get_chorus(&self) -> ChorusParams {
        ChorusParams {
            nr: self.get_chorus_nr() as _,
            level: self.get_chorus_level(),
            speed: self.get_chorus_speed_hz() as _,
            depth: self.get_chorus_depth_ms() as _,
            mode: self.get_chorus_mode(),
        }
    }
}
