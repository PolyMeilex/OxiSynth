use crate::error::{range_check, OxiError};

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
                range_check(0..=127, key, OxiError::KeyOutOfRange)?;
                range_check(0..=127, vel, OxiError::VelocityOutOfRange)?;
            }
            MidiEvent::NoteOff { key, .. } => {
                range_check(0..=127, key, OxiError::KeyOutOfRange)?;
            }
            MidiEvent::ControlChange { ctrl, value, .. } => {
                range_check(0..=127, ctrl, OxiError::CtrlOutOfRange)?;
                range_check(0..=127, value, OxiError::CCValueOutOfRange)?;
            }
            MidiEvent::AllNotesOff { .. } => {}
            MidiEvent::AllSoundOff { .. } => {}
            MidiEvent::PitchBend { value, .. } => {
                range_check(0..=16383, value, OxiError::PithBendOutOfRange)?;
            }
            MidiEvent::ProgramChange { program_id, .. } => {
                range_check(0..=127, program_id, OxiError::ProgramOutOfRange)?;
            }
            MidiEvent::ChannelPressure { value, .. } => {
                range_check(0..=127, value, OxiError::ChannelPressureOutOfRange)?;
            }
            MidiEvent::PolyphonicKeyPressure { key, value, .. } => {
                range_check(0..=127, key, OxiError::KeyOutOfRange)?;
                range_check(0..=127, value, OxiError::KeyPressureOutOfRange)?;
            }
            MidiEvent::SystemReset => {}
        };

        Ok(self)
    }
}

macro_rules! u8_to_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($(#[$vmeta:meta])* $vname:ident $(= $val:expr)?,)*
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $($(#[$vmeta])* $vname $(= $val)?,)*
        }

        impl $name {
            #[allow(unused)]
            pub const fn const_try_from(v: u8) -> Option<Self> {
                match v {
                    $(x if x == $name::$vname as u8 => Some($name::$vname),)*
                    _ => None,
                }
            }
        }
    }
}

u8_to_enum!(
    #[allow(unused)]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
    pub enum ControlFunction {
        BankSelect = 0,
        ModulationWheel = 1,
        BreathController = 2,
        Undefined3 = 3,
        FootController = 4,
        PortamentoTime = 5,
        DataEntryMsb = 6,
        ChannelVolume = 7,
        Balance = 8,
        Undefined9 = 9,
        Pan = 10,
        ExpressionController = 11,
        EffectControl1 = 12,
        EffectControl2 = 13,
        Undefined14 = 14,
        Undefined15 = 15,
        GeneralPurposeController1 = 16,
        GeneralPurposeController2 = 17,
        GeneralPurposeController3 = 18,
        GeneralPurposeController4 = 19,
        Undefined20 = 20,
        Undefined21 = 21,
        Undefined22 = 22,
        Undefined23 = 23,
        Undefined24 = 24,
        Undefined25 = 25,
        Undefined26 = 26,
        Undefined27 = 27,
        Undefined28 = 28,
        Undefined29 = 29,
        Undefined30 = 30,
        Undefined31 = 31,
        BankSelectLsb = 32,
        ModulationWheelLsb = 33,
        BreathControllerLsb = 34,
        Undefined3Lsb = 35,
        FootControllerLsb = 36,
        PortamentoTimeLsb = 37,
        DataEntryLsb = 38,
        ChannelVolumeLsb = 39,
        BalanceLsb = 40,
        Undefined9Lsb = 41,
        PanLsb = 42,
        ExpressionControllerLsb = 43,
        EffectControl1Lsb = 44,
        EffectControl2Lsb = 45,
        Undefined14Lsb = 46,
        Undefined15Lsb = 47,
        GeneralPurposeController1Lsb = 48,
        GeneralPurposeController2Lsb = 49,
        GeneralPurposeController3Lsb = 50,
        GeneralPurposeController4Lsb = 51,
        Undefined20Lsb = 52,
        Undefined21Lsb = 53,
        Undefined22Lsb = 54,
        Undefined23Lsb = 55,
        Undefined24Lsb = 56,
        Undefined25Lsb = 57,
        Undefined26Lsb = 58,
        Undefined27Lsb = 59,
        Undefined28Lsb = 60,
        Undefined29Lsb = 61,
        Undefined30Lsb = 62,
        Undefined31Lsb = 63,
        DamperPedal = 64,
        PortamentoOnOff = 65,
        Sostenuto = 66,
        SoftPedal = 67,
        LegatoFootswitch = 68,
        Hold2 = 69,
        SoundController1 = 70,
        SoundController2 = 71,
        SoundController3 = 72,
        SoundController4 = 73,
        SoundController5 = 74,
        SoundController6 = 75,
        SoundController7 = 76,
        SoundController8 = 77,
        SoundController9 = 78,
        SoundController10 = 79,
        GeneralPurposeController5 = 80,
        GeneralPurposeController6 = 81,
        GeneralPurposeController7 = 82,
        GeneralPurposeController8 = 83,
        PortamentoControl = 84,
        Undefined85 = 85,
        Undefined86 = 86,
        Undefined87 = 87,
        Undefined88 = 88,
        Undefined89 = 89,
        Undefined90 = 90,
        Effects1Depth = 91,
        Effects2Depth = 92,
        Effects3Depth = 93,
        Effects4Depth = 94,
        Effects5Depth = 95,
        DataIncrement = 96,
        DataDecrement = 97,
        NonRegisteredParameterNumberLsb = 98,
        NonRegisteredParameterNumberMsb = 99,
        RegisteredParameterNumberLsb = 100,
        RegisteredParameterNumberMsb = 101,

        Undefined102 = 102,
        Undefined103 = 103,
        Undefined104 = 104,
        Undefined105 = 105,
        Undefined106 = 106,
        Undefined107 = 107,
        Undefined108 = 108,
        Undefined109 = 109,
        Undefined110 = 110,
        Undefined111 = 111,
        Undefined112 = 112,
        Undefined113 = 113,
        Undefined114 = 114,
        Undefined115 = 115,
        Undefined116 = 116,
        Undefined117 = 117,
        Undefined118 = 118,
        Undefined119 = 119,

        AllSoundOff = 120,
        ResetAllControllers = 121,
        LocalControl = 122,
        AllNotesOff = 123,
        OmniModeOn = 124,
        OmniModeOff = 125,
        MonoOperation = 126,
        PolyOperation = 127,
    }
);

#[allow(unused)]
impl ControlFunction {
    pub const MIN: Self = Self::BankSelect;
    pub const MAX: Self = Self::PolyOperation;

    pub fn iter() -> impl Iterator<Item = Self> {
        let mut range = ControlFunction::MIN as u8..=ControlFunction::MAX as u8;
        std::iter::from_fn(move || {
            let v = range.next()?;
            let v = ControlFunction::const_try_from(v).unwrap();

            Some(v)
        })
    }

    pub fn iter_range(v: impl std::ops::RangeBounds<Self>) -> impl Iterator<Item = Self> {
        let first = match v.start_bound() {
            std::ops::Bound::Included(v) => *v as u8,
            std::ops::Bound::Excluded(v) => *v as u8,
            std::ops::Bound::Unbounded => ControlFunction::MIN as u8,
        };
        let last = match v.end_bound() {
            std::ops::Bound::Included(v) => *v as u8 + 1,
            std::ops::Bound::Excluded(v) => *v as u8,
            std::ops::Bound::Unbounded => ControlFunction::MAX as u8,
        };

        let mut range = first..last;

        std::iter::from_fn(move || {
            let v = range.next()?;
            let v = ControlFunction::const_try_from(v).unwrap();

            Some(v)
        })
    }

    pub fn is_effects_n_depth(&self) -> bool {
        matches!(
            self,
            Self::Effects1Depth
                | Self::Effects2Depth
                | Self::Effects3Depth
                | Self::Effects4Depth
                | Self::Effects5Depth
        )
    }

    pub fn is_sound_controller_n(&self) -> bool {
        matches!(
            self,
            Self::SoundController1
                | Self::SoundController2
                | Self::SoundController3
                | Self::SoundController4
                | Self::SoundController5
                | Self::SoundController6
                | Self::SoundController7
                | Self::SoundController8
                | Self::SoundController9
                | Self::SoundController10
        )
    }
}

const _: () = {
    if ControlFunction::MIN as u8 != 0 {
        unreachable!();
    }

    if ControlFunction::MAX as u8 != 127 {
        unreachable!();
    }

    let mut i = 0;
    while i <= ControlFunction::MAX as u8 {
        let v = ControlFunction::const_try_from(i).unwrap();

        if v as u8 != i {
            unreachable!();
        }

        i += 1;
    }

    if i != ControlFunction::MAX as u8 + 1 {
        unreachable!();
    }

    if ControlFunction::const_try_from(i).is_some() {
        unreachable!();
    }
};

#[test]
fn abc() {
    for v in ControlFunction::iter() {
        println!("{v:?}");
    }
}
