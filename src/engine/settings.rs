use super::synth::Synth;
use std::collections::HashMap;
pub type SettingsType = i32;
pub const FLUID_SET_TYPE: SettingsType = 3;
pub const FLUID_STR_TYPE: SettingsType = 2;
pub const FLUID_INT_TYPE: SettingsType = 1;
pub const FLUID_NUM_TYPE: SettingsType = 0;
pub const FLUID_NO_TYPE: SettingsType = -1;

pub struct Range<T> {
    pub min: T,
    pub max: T,
}

pub enum Setting {
    Int(IntSetting),
    Str(StrSetting),
    Num(NumSetting),
    Set(HashMap<String, Setting>),
}

#[derive(Clone)]
pub struct StrSetting {
    value: String,
    def: String,
    hints: i32,
    update: StrUpdateFn,
    data: *mut libc::c_void,
}
pub type StrUpdateFn = Option<unsafe fn(_: *mut libc::c_void, _: &str, _: String) -> i32>;
#[derive(Clone)]
pub struct IntSetting {
    value: i32,
    def: i32,
    min: i32,
    max: i32,
    hints: i32,
    update: IntUpdateFn,
    data: *mut libc::c_void,
}
pub type IntUpdateFn = Option<unsafe fn(_: *mut libc::c_void, _: &str, _: i32) -> i32>;
#[derive(Clone)]
pub struct NumSetting {
    value: f64,
    def: f64,
    min: f64,
    max: f64,
    hints: i32,
    update: NumUpdateFn,
    data: *mut libc::c_void,
}

pub type NumUpdateFn = Option<unsafe fn(_: *mut libc::c_void, _: &str, _: f64) -> i32>;

unsafe fn new_fluid_str_setting(
    value: &str,
    def: &str,
    hints: i32,
    fun: StrUpdateFn,
    data: *mut libc::c_void,
) -> StrSetting {
    return StrSetting {
        value: value.to_string(),
        def: def.to_string(),
        hints,
        update: fun,
        data,
    };
}

unsafe fn new_fluid_num_setting(
    min: f64,
    max: f64,
    def: f64,
    hints: i32,
    fun: NumUpdateFn,
    data: *mut libc::c_void,
) -> NumSetting {
    return NumSetting {
        value: def,
        def,
        min,
        max,
        hints,
        update: fun,
        data,
    };
}

unsafe fn new_fluid_int_setting(
    min: i32,
    max: i32,
    def: i32,
    hints: i32,
    fun: IntUpdateFn,
    data: *mut libc::c_void,
) -> IntSetting {
    return IntSetting {
        value: def,
        def,
        min,
        max,
        hints,
        update: fun,
        data,
    };
}

pub struct Settings {
    table: HashMap<String, Setting>,
}

impl Settings {
    pub unsafe fn new() -> Settings {
        let mut settings = Settings {
            table: HashMap::new(),
        };
        settings.init();
        return settings;
    }

    unsafe fn init(&mut self) {
        Synth::register_settings(self);
    }

    unsafe fn get<'a>(&'a self, name: &[String]) -> Option<&'a Setting> {
        let mut table = &self.table;
        for n in 0..name.len() - 1 {
            match table.get(&name[n]) {
                Some(Setting::Set(t)) => table = t,
                _ => return None,
            }
        }
        return table.get(&name[name.len() - 1]);
    }

    unsafe fn get_mut<'a>(&'a mut self, name: &[String]) -> Option<&'a mut Setting> {
        let mut table = &mut self.table;
        for n in 0..name.len() - 1 {
            match table.get_mut(&name[n]) {
                Some(Setting::Set(t)) => table = t,
                _ => return None,
            }
        }
        return table.get_mut(&name[name.len() - 1]);
    }

    unsafe fn set(&mut self, name: &[String], value: Setting) -> i32 {
        let mut table = &mut self.table;
        for n in 0..name.len() - 1 {
            if table.get(&name[n]).is_none() {
                let t = HashMap::new();
                table.insert(name[n].to_string(), Setting::Set(t));
            }
            table = match table.get_mut(&name[n].to_string()) {
                Some(Setting::Set(t)) => t,
                _ => return 0,
            };
        }
        table.insert(name[name.len() - 1].to_string(), value);
        return 1;
    }

    pub unsafe fn register_str(
        &mut self,
        name: &str,
        def: &str,
        hints: i32,
        fun: StrUpdateFn,
        data: *mut libc::c_void,
    ) -> i32 {
        let tokens: Vec<String> = name.split(".").map(|x| x.to_string()).collect();
        match self.get_mut(&tokens) {
            None => {
                return self.set(
                    &tokens,
                    Setting::Str(new_fluid_str_setting(def, def, hints, fun, data)),
                );
            }
            Some(Setting::Str(setting)) => {
                setting.update = fun;
                setting.data = data;
                setting.def = def.to_string();
                setting.hints = hints;
                return 1 as i32;
            }
            _ => {
                fluid_log!(FLUID_WARN, "Type mismatch on setting \'{}\'", name);
                return 1 as i32;
            }
        }
    }

    pub fn register_num(
        &mut self,
        name: &str,
        def: f64,
        min: f64,
        max: f64,
        hints: i32,
        fun: NumUpdateFn,
        data: *mut libc::c_void,
    ) -> i32 {
        unsafe {
            let tokens: Vec<String> = name.split(".").map(|x| x.to_string()).collect();
            match self.get_mut(&tokens) {
                None => {
                    return self.set(
                        &tokens,
                        Setting::Num(new_fluid_num_setting(min, max, def, hints, fun, data)),
                    );
                }
                Some(Setting::Num(setting)) => {
                    setting.update = fun;
                    setting.data = data;
                    setting.min = min;
                    setting.max = max;
                    setting.def = def;
                    setting.hints = hints;
                    return 1 as i32;
                }
                _ => {
                    fluid_log!(FLUID_WARN, "Type mismatch on setting \'{}\'", name);
                    return 0 as i32;
                }
            };
        }
    }

    pub fn register_int(
        &mut self,
        name: &str,
        def: i32,
        min: i32,
        max: i32,
        hints: i32,
        fun: IntUpdateFn,
        data: *mut libc::c_void,
    ) -> i32 {
        unsafe {
            let tokens: Vec<String> = name.split(".").map(|x| x.to_string()).collect();
            match self.get_mut(&tokens) {
                Some(Setting::Int(setting)) => {
                    setting.update = fun;
                    setting.data = data;
                    setting.min = min;
                    setting.max = max;
                    setting.def = def;
                    setting.hints = hints;
                    return 1 as i32;
                }
                None => {
                    return self.set(
                        &tokens,
                        Setting::Int(new_fluid_int_setting(min, max, def, hints, fun, data)),
                    );
                }
                _ => {
                    return 0;
                }
            }
        }
    }

    pub unsafe fn get_hints(&self, name: &str) -> i32 {
        let tokens: Vec<String> = name.split(".").map(|x| x.to_string()).collect();
        return match self.get(&tokens) {
            Some(Setting::Num(s)) => s.hints,
            Some(Setting::Str(s)) => s.hints,
            _ => 0,
        };
    }

    pub unsafe fn is_realtime(&self, name: &str) -> bool {
        let tokens: Vec<String> = name.split(".").map(|x| x.to_string()).collect();
        return match self.get(&tokens) {
            Some(Setting::Num(s)) => s.update.is_some(),
            Some(Setting::Str(s)) => s.update.is_some(),
            _ => false,
        };
    }

    pub unsafe fn setstr(&mut self, name: &str, str: &str) -> i32 {
        let tokens: Vec<String> = name.split(".").map(|x| x.to_string()).collect();
        match self.get_mut(&tokens) {
            Some(Setting::Str(setting)) => {
                setting.value = str.to_string();
                if setting.update.is_some() {
                    Some(setting.update.expect("non-null function pointer"))
                        .expect("non-null function pointer")(
                        setting.data,
                        name,
                        setting.value.clone(),
                    );
                }
                return 1;
            }
            None => {
                let setting;
                setting = new_fluid_str_setting(str, "", 0 as i32, None, 0 as *mut libc::c_void);
                return self.set(&tokens, Setting::Str(setting));
            }
            _ => return 0,
        }
    }

    pub unsafe fn getstr(&self, name: &str) -> Option<String> {
        let tokens: Vec<String> = name.split(".").map(|x| x.to_string()).collect();
        return match self.get(&tokens) {
            Some(Setting::Str(s)) => Some(s.value.clone()),
            _ => None,
        };
    }

    pub fn str_equal(&self, name: &str, s: &str) -> bool {
        unsafe {
            let tokens: Vec<String> = name.split(".").map(|x| x.to_string()).collect();
            return match self.get(&tokens) {
                Some(Setting::Str(setting)) => setting.value == s,
                _ => false,
            };
        }
    }

    pub unsafe fn getstr_default(&self, name: &str) -> String {
        let tokens: Vec<String> = name.split(".").map(|x| x.to_string()).collect();
        return match self.get(&tokens) {
            Some(Setting::Str(s)) => s.def.clone(),
            _ => String::new(),
        };
    }

    pub unsafe fn setnum(&mut self, name: &str, mut val: f64) -> i32 {
        let tokens: Vec<String> = name.split(".").map(|x| x.to_string()).collect();
        match self.get_mut(&tokens) {
            Some(Setting::Num(setting)) => {
                if val < setting.min {
                    val = setting.min
                } else if val > setting.max {
                    val = setting.max
                }
                setting.value = val;
                if setting.update.is_some() {
                    Some(setting.update.expect("non-null function pointer"))
                        .expect("non-null function pointer")(
                        setting.data, name, val
                    );
                }
                return 1;
            }
            None => {
                let mut setting;
                setting = new_fluid_num_setting(
                    -1e10f64,
                    1e10f64,
                    0.0f32 as f64,
                    0 as i32,
                    None,
                    0 as *mut libc::c_void,
                );
                setting.value = val;
                return self.set(&tokens, Setting::Num(setting));
            }
            _ => return 0,
        }
    }

    pub fn getnum(&self, name: &str) -> Option<f64> {
        unsafe {
            let tokens: Vec<String> = name.split(".").map(|x| x.to_string()).collect();
            return match self.get(&tokens) {
                Some(Setting::Num(s)) => Some(s.value),
                _ => None,
            };
        }
    }

    pub unsafe fn getnum_range(&self, name: &str) -> Option<Range<f64>> {
        let tokens: Vec<String> = name.split(".").map(|x| x.to_string()).collect();
        return match self.get(&tokens) {
            Some(Setting::Num(s)) => Some(Range {
                min: s.min,
                max: s.max,
            }),
            _ => None,
        };
    }

    pub unsafe fn getnum_default(&self, name: &str) -> f64 {
        let tokens: Vec<String> = name.split(".").map(|x| x.to_string()).collect();
        return match self.get(&tokens) {
            Some(Setting::Num(s)) => s.def,
            _ => 0f64,
        };
    }

    pub fn setint(&mut self, name: &str, mut val: i32) -> i32 {
        unsafe {
            let tokens: Vec<String> = name.split(".").map(|x| x.to_string()).collect();
            match self.get_mut(&tokens) {
                Some(Setting::Int(setting)) => {
                    if val < (*setting).min {
                        val = (*setting).min
                    } else if val > (*setting).max {
                        val = (*setting).max
                    }
                    setting.value = val;
                    if setting.update.is_some() {
                        Some(setting.update.expect("non-null function pointer"))
                            .expect("non-null function pointer")(
                            setting.data, name, val
                        );
                    }
                    return 1;
                }
                None => {
                    let mut setting;
                    setting = new_fluid_int_setting(
                        -(2147483647 as i32) - 1 as i32,
                        2147483647 as i32,
                        0 as i32,
                        0 as i32,
                        None,
                        0 as *mut libc::c_void,
                    );
                    setting.value = val;
                    return self.set(&tokens, Setting::Int(setting));
                }
                _ => {
                    return 0;
                }
            }
        }
    }

    pub fn getint(&self, name: &str) -> Option<i32> {
        unsafe {
            let tokens: Vec<String> = name.split(".").map(|x| x.to_string()).collect();
            return match self.get(&tokens) {
                Some(Setting::Int(s)) => Some(s.value),
                _ => None,
            };
        }
    }

    pub unsafe fn getint_range(&self, name: &str) -> Option<Range<i32>> {
        let tokens: Vec<String> = name.split(".").map(|x| x.to_string()).collect();
        return match self.get(&tokens) {
            Some(Setting::Int(s)) => Some(Range {
                min: s.min,
                max: s.max,
            }),
            _ => None,
        };
    }

    pub unsafe fn getint_default(&self, name: &str) -> i32 {
        let tokens: Vec<String> = name.split(".").map(|x| x.to_string()).collect();
        return match self.get(&tokens) {
            Some(Setting::Int(s)) => s.def,
            _ => 0,
        };
    }

    pub fn get_type(&self, name: &str) -> SettingsType {
        unsafe {
            let tokens: Vec<String> = name.split(".").map(|x| x.to_string()).collect();
            return match self.get(&tokens) {
                Some(Setting::Num(_)) => FLUID_SET_TYPE,
                Some(Setting::Str(_)) => FLUID_STR_TYPE,
                Some(Setting::Int(_)) => FLUID_INT_TYPE,
                Some(Setting::Set(_)) => FLUID_NUM_TYPE,
                _ => FLUID_NO_TYPE,
            };
        }
    }
}
