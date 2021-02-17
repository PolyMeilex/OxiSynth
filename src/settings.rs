use crate::{engine, Result};
use bitflags::bitflags;
use std::{
    marker::PhantomData,
    ops::{Bound, RangeBounds},
};

/**
The generic settings object
 */
#[repr(transparent)]
pub struct Settings {
    pub(crate) handle: engine::settings::Settings,
}

unsafe impl Send for Settings {}

/**
The settings reference
*/
#[repr(transparent)]
pub struct SettingsRef<'a> {
    pub(crate) handle: *mut engine::settings::Settings,
    phantom: PhantomData<&'a ()>,
}

impl Settings {
    pub fn new() -> Result<Self> {
        return Ok(Self {
            handle: unsafe { engine::settings::Settings::new() },
        });
    }
}

impl<'a> SettingsRef<'a> {
    pub(crate) fn from_ptr(handle: *mut engine::settings::Settings) -> Self {
        Self {
            handle,
            phantom: PhantomData,
        }
    }
}

/**
The settings interface
 */
pub trait IsSettings {
    fn pick<S, T>(&mut self, name: S) -> Option<Setting<'_, T>>
    where
        T: IsSetting + ?Sized,
        S: ToString;

    fn str_<S>(&mut self, name: S) -> Option<Setting<'_, str>>
    where
        S: ToString;

    fn num<S>(&mut self, name: S) -> Option<Setting<'_, f64>>
    where
        S: ToString;

    fn int<S>(&mut self, name: S) -> Option<Setting<'_, i32>>
    where
        S: ToString;
}

mod private {
    use crate::{
        engine, private::HasHandle, IsSetting, IsSettings, Setting, Settings, SettingsRef,
    };
    use std::marker::PhantomData;

    impl<X> IsSettings for X
    where
        X: HasHandle<Handle = engine::settings::Settings>,
    {
        fn pick<S, T>(&mut self, name: S) -> Option<Setting<'_, T>>
        where
            T: IsSetting + ?Sized,
            S: ToString,
        {
            let handle = self.get_mut_handle();

            if T::TYPE == unsafe { handle.as_ref().unwrap().get_type(name.to_string().as_str()) } {
                Some(Setting {
                    handle,
                    name: name.to_string(),
                    phantom: PhantomData,
                })
            } else {
                None
            }
        }

        fn str_<S>(&mut self, name: S) -> Option<Setting<'_, str>>
        where
            S: ToString,
        {
            self.pick(name)
        }

        fn num<S>(&mut self, name: S) -> Option<Setting<'_, f64>>
        where
            S: ToString,
        {
            self.pick(name)
        }

        fn int<S>(&mut self, name: S) -> Option<Setting<'_, i32>>
        where
            S: ToString,
        {
            self.pick(name)
        }
    }

    impl HasHandle for Settings {
        type Handle = engine::settings::Settings;

        fn get_handle(&self) -> *const Self::Handle {
            &self.handle as *const Self::Handle
        }

        fn get_mut_handle(&mut self) -> *mut Self::Handle {
            &mut self.handle as *mut Self::Handle
        }
    }

    impl<'a> HasHandle for SettingsRef<'a> {
        type Handle = engine::settings::Settings;

        fn get_handle(&self) -> *const Self::Handle {
            self.handle
        }

        fn get_mut_handle(&mut self) -> *mut Self::Handle {
            self.handle
        }
    }
}

/**
The single setting object interface
 */
pub trait IsSetting {
    const TYPE: engine::settings::SettingsType;
}

impl IsSetting for str {
    const TYPE: engine::settings::SettingsType = engine::settings::FLUID_STR_TYPE;
}

impl IsSetting for f64 {
    const TYPE: engine::settings::SettingsType = engine::settings::FLUID_NUM_TYPE;
}

impl IsSetting for i32 {
    const TYPE: engine::settings::SettingsType = engine::settings::FLUID_INT_TYPE;
}

impl IsSetting for () {
    const TYPE: engine::settings::SettingsType = engine::settings::FLUID_SET_TYPE;
}

bitflags! {
    /**
    The setting hints
     */
    pub struct Hints: i32 {
        /**
        Hint BOUNDED_BELOW indicates that the LowerBound field
        of the FLUID_PortRangeHint should be considered meaningful. The
        value in this field should be considered the (inclusive) lower
        bound of the valid range. If SAMPLE_RATE is also
        specified then the value of LowerBound should be multiplied by the
        sample rate.
         */
        const BOUNDED_BELOW = 1;

        /**
        Hint BOUNDED_ABOVE indicates that the UpperBound field
        of the FLUID_PortRangeHint should be considered meaningful. The
        value in this field should be considered the (inclusive) upper
        bound of the valid range. If SAMPLE_RATE is also
        specified then the value of UpperBound should be multiplied by the
        sample rate.
         */
        const BOUNDED_ABOVE = 2;

        /**
        Hint TOGGLED indicates that the data item should be
        considered a Boolean toggle. Data less than or equal to zero should
        be considered `off' or `false,' and data above zero should be
        considered `on' or `true.' TOGGLED may not be used in
        conjunction with any other hint except DEFAULT_0 or
        DEFAULT_1.
         */
        const TOGGLED = 4;

        /**
        Hint SAMPLE_RATE indicates that any bounds specified
        should be interpreted as multiples of the sample rate. For
        instance, a frequency range from 0Hz to the Nyquist frequency (half
        the sample rate) could be requested by this hint in conjunction
        with LowerBound = 0 and UpperBound = 0.5. Hosts that support bounds
        at all must support this hint to retain meaning.
         */
        const SAMPLE_RATE = 8;

        /**
        Hint LOGARITHMIC indicates that it is likely that the
        user will find it more intuitive to view values using a logarithmic
        scale. This is particularly useful for frequencies and gains.
         */
        const LOGARITHMIC = 16;

        /**
        Hint INTEGER indicates that a user interface would
        probably wish to provide a stepped control taking only integer
        values. Any bounds set should be slightly wider than the actual
        integer range required to avoid floating point rounding errors. For
        instance, the integer set {0,1,2,3} might be described as [-0.1,
        3.1].
         */
        const INTEGER = 32;

        const FILENAME = 1;

        const OPTIONLIST = 2;
    }
}

/**
The single setting of specific type
 */
pub struct Setting<'a, T: ?Sized> {
    handle: *mut engine::settings::Settings,
    name: String,
    phantom: PhantomData<(&'a (), T)>,
}

impl<'a, T> Setting<'a, T>
where
    T: ?Sized,
{
    pub fn hints(&self) -> Hints {
        Hints::from_bits_truncate(unsafe {
            engine::settings::Settings::get_hints(&*self.handle, &self.name)
        })
    }

    /** Returns whether the setting is changeable in real-time
     */
    pub fn is_realtime(&self) -> bool {
        unsafe { engine::settings::Settings::is_realtime(&*self.handle, &self.name) }
    }
}

impl<'a> Setting<'a, str> {
    /**
    Set the value of a string setting

    Returns `true` if the value has been set, `false` otherwise
     */
    pub fn set<S: Into<String>>(&self, value: S) -> bool {
        let mut value = value.into();
        value.push('\0');
        0 < unsafe { engine::settings::Settings::setstr(&mut *self.handle, &self.name, &value) }
    }

    /**
    Get the value of a string setting

    Returns `Some("value")` if the value exists, `None` otherwise
     */
    pub fn get(&self) -> Option<String> {
        unsafe {
            return engine::settings::Settings::getstr(&*self.handle, &self.name);
        }
    }

    /**
    Get the default value of a string setting
     */
    pub fn default(&self) -> String {
        unsafe {
            return engine::settings::Settings::getstr_default(&*self.handle, &self.name);
        }
    }
}

impl<'a, S> PartialEq<S> for Setting<'a, str>
where
    S: AsRef<str>,
{
    fn eq(&self, other: &S) -> bool {
        engine::settings::Settings::str_equal(
            unsafe { &mut *self.handle },
            &self.name,
            other.as_ref(),
        )
    }
}

/**
The range of setting value
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Range<T> {
    /// Below limit
    pub min: Option<T>,
    /// Above limit
    pub max: Option<T>,
}

impl<T> Range<T> {
    pub fn new(min: Option<T>, max: Option<T>) -> Self {
        Self { min, max }
    }
}

impl<T> RangeBounds<T> for Range<T> {
    fn start_bound(&self) -> Bound<&T> {
        if let Some(value) = &self.min {
            Bound::Included(value)
        } else {
            Bound::Unbounded
        }
    }

    fn end_bound(&self) -> Bound<&T> {
        if let Some(value) = &self.max {
            Bound::Included(value)
        } else {
            Bound::Unbounded
        }
    }
}

impl<'a> Setting<'a, f64> {
    /**
    Set the value of a numeric setting

    Returns `true` if the value has been set, `false` otherwise
     */
    pub fn set(&self, value: f64) -> bool {
        0 < unsafe { engine::settings::Settings::setnum(&mut *self.handle, &self.name, value) }
    }

    /**
    Get the value of a numeric setting

    Returns `Some(value)` if the value exists, `None` otherwise
     */
    pub fn get(&self) -> Option<f64> {
        return engine::settings::Settings::getnum(unsafe { &*self.handle }, &self.name);
    }

    /**
    Get the default value of a numeric setting
     */
    pub fn default(&self) -> f64 {
        unsafe { engine::settings::Settings::getnum_default(&*self.handle, &self.name) }
    }

    /**
    Get the range of values of a numeric setting
     */
    pub fn range(&self) -> Range<f64> {
        unsafe {
            let hints = self.hints();
            return match engine::settings::Settings::getnum_range(&*self.handle, &self.name) {
                Some(range) => Range::new(
                    if hints.contains(Hints::BOUNDED_BELOW) {
                        Some(range.min)
                    } else {
                        None
                    },
                    if hints.contains(Hints::BOUNDED_ABOVE) {
                        Some(range.max)
                    } else {
                        None
                    },
                ),
                None => Range::new(None, None),
            };
        }
    }
}

impl<'a> Setting<'a, i32> {
    /**
    Set the value of a integer setting

    Returns `true` if the value has been set, `false` otherwise
     */
    pub fn set(&self, value: i32) -> bool {
        0 < engine::settings::Settings::setint(unsafe { &mut *self.handle }, &self.name, value)
    }

    /**
    Get the value of a integer setting

    Returns `Some(value)` if the value exists, `None` otherwise
     */
    pub fn get(&self) -> Option<i32> {
        return engine::settings::Settings::getint(unsafe { &*self.handle }, &self.name);
    }

    /**
    Get the default value of a integer setting
     */
    pub fn default(&self) -> i32 {
        unsafe { engine::settings::Settings::getint_default(&*self.handle, &self.name) }
    }

    /**
    Get the range of values of a integer setting
     */
    pub fn range(&self) -> Range<i32> {
        unsafe {
            let hints = self.hints();
            return match engine::settings::Settings::getint_range(&*self.handle, &self.name) {
                Some(range) => Range::new(
                    if hints.contains(Hints::BOUNDED_BELOW) {
                        Some(range.min)
                    } else {
                        None
                    },
                    if hints.contains(Hints::BOUNDED_ABOVE) {
                        Some(range.max)
                    } else {
                        None
                    },
                ),
                None => Range::new(None, None),
            };
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn settings() {
        let settings = Settings::new().unwrap();

        drop(settings);
    }

    #[test]
    fn num_setting() {
        let mut settings = Settings::new().unwrap();
        let gain = settings.num("synth.gain").unwrap();

        assert_eq!(gain.default(), 0.2f32 as f64);
        //assert_eq!(gain.range().min, Some(0.0));
        //assert_eq!(gain.range().max, Some(10.0));

        assert_eq!(gain.get(), Some(0.2f32 as f64));
        assert!(gain.set(0.5));
        assert_eq!(gain.get(), Some(0.5));
    }

    #[test]
    fn int_setting() {
        let mut settings = Settings::new().unwrap();
        let polyphony = settings.int("synth.polyphony").unwrap();

        assert_eq!(polyphony.default(), 256);
        //assert_eq!(polyphony.range().min, Some(1));
        //assert_eq!(polyphony.range().max, Some(65535));

        assert_eq!(polyphony.get(), Some(256));
        assert!(polyphony.set(512));
        assert_eq!(polyphony.get(), Some(512));
    }

    #[test]
    fn str_setting() {
        let mut settings = Settings::new().unwrap();
        let active = settings.str_("synth.drums-channel.active").unwrap();

        assert_eq!(active.default(), "yes");

        assert_eq!(active.get(), Some("yes".to_string()));
        assert!(active.set("no"));
        assert_eq!(active.get(), Some("no".to_string()));
    }
}
