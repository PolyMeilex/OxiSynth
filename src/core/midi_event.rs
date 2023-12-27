use super::{utils::RangeCheck, OxiError};

pub type U7 = u8;
pub type U14 = u16;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MidiEvent {
    /// Send a noteon message.
    NoteOn {
        channel: u8,
        key: U7,
        vel: U7,
    },
    /// Send a noteoff message.
    NoteOff {
        channel: u8,
        key: U7,
    },
    /// Send a control change message.
    ControlChange {
        channel: u8,
        ctrl: U7,
        value: U7,
    },
    AllNotesOff {
        channel: u8,
    },
    AllSoundOff {
        channel: u8,
    },
    /// Send a pitch bend message.
    PitchBend {
        channel: u8,
        value: U14,
    },
    /// Send a program change message.
    ProgramChange {
        channel: u8,
        program_id: U7,
    },
    /// Set channel pressure
    ChannelPressure {
        channel: u8,
        value: U7,
    },
    /// Set key pressure (aftertouch)
    PolyphonicKeyPressure {
        channel: u8,
        key: U7,
        value: U7,
    },
    /// Send a reset.
    ///
    /// A reset turns all the notes off and resets the controller values.
    ///
    /// Purpose:
    /// Respond to the MIDI command 'system reset' (0xFF, big red 'panic' button)
    SystemReset,
}

impl MidiEvent {
    pub fn check(self) -> Result<Self, OxiError> {
        match &self {
            MidiEvent::NoteOn { key, vel, .. } => {
                RangeCheck::check(0..=127, key, OxiError::KeyOutOfRange)?;
                RangeCheck::check(0..=127, vel, OxiError::VelocityOutOfRange)?;
            }
            MidiEvent::NoteOff { key, .. } => {
                RangeCheck::check(0..=127, key, OxiError::KeyOutOfRange)?;
            }
            MidiEvent::ControlChange { ctrl, value, .. } => {
                RangeCheck::check(0..=127, ctrl, OxiError::CtrlOutOfRange)?;
                RangeCheck::check(0..=127, value, OxiError::CCValueOutOfRange)?;
            }
            MidiEvent::AllNotesOff { .. } => {}
            MidiEvent::AllSoundOff { .. } => {}
            MidiEvent::PitchBend { value, .. } => {
                RangeCheck::check(0..=16383, value, OxiError::PithBendOutOfRange)?;
            }
            MidiEvent::ProgramChange { program_id, .. } => {
                RangeCheck::check(0..=127, program_id, OxiError::ProgramOutOfRange)?;
            }
            MidiEvent::ChannelPressure { value, .. } => {
                RangeCheck::check(0..=127, value, OxiError::ChannelPressureOutOfRange)?;
            }
            MidiEvent::PolyphonicKeyPressure { key, value, .. } => {
                RangeCheck::check(0..=127, key, OxiError::KeyOutOfRange)?;
                RangeCheck::check(0..=127, value, OxiError::KeyPressureOutOfRange)?;
            }
            MidiEvent::SystemReset => {}
        };

        Ok(self)
    }
}
