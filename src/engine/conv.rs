lazy_static! {
    static ref CT2HZ_TAB: [f32; 1200] = {
        let mut init = [0f32; 1200];
        for i in 0..1200 {
            init[i] = f64::powf(2.0f64, i as f64 / 1200.0f64) as f32;
        }
        init
    };
    static ref CB2AMP_TAB: [f32; 961] = {
        let mut init = [0f32; 961];
        for i in 0..961 {
            init[i] = f64::powf(10.0f64, i as f64 / -200.0f64) as f32;
        }
        init
    };
    static ref ATTEN2AMP_TAB: [f32; 1441] = {
        let mut init = [0f32; 1441];
        for i in 0..1441 {
            init[i] = f64::powf(10.0f64, i as f64 / -200.0f64) as f32;
        }
        init
    };
    static ref CONCAVE_TAB: [f32; 128] = {
        let mut init = [0f32; 128];
        init[0] = 0.0f32;
        init[127] = 1.0f32;
        let mut x: f64;
        for i in 1..127 {
            x = -20.0f64 / 96.0f64 * f64::ln((i * i) as f64 / (127.0f64 * 127.0f64))
                / f64::ln(10.0f64);
            init[127 - i] = x as f32;
        }
        init
    };
    static ref CONVEX_TAB: [f32; 128] = {
        let mut init = [0f32; 128];
        init[0] = 0 as i32 as f32;
        init[127] = 1.0f32;
        let mut x: f64;
        for i in 1..127 {
            x = -20.0f64 / 96.0f64 * f64::ln((i * i) as f64 / (127.0f64 * 127.0f64))
                / f64::ln(10.0f64);
            init[i] = (1.0f64 - x) as f32;
        }
        init
    };
    static ref PAN_TAB: [f32; 1002] = {
        let mut init = [0f32; 1002];
        let x = 3.141592654f64 / 2.0f64 / (1002f64 - 1.0f64);
        for i in 0..1002 {
            init[i] = f64::sin(i as f64 * x) as f32;
        }
        init
    };
}

pub fn fluid_ct2hz_real(cents: f32) -> f32 {
    return {
        if cents < 0f32 {
            1.0f32
        } else if cents < 900f32 {
            6.875f32 * CT2HZ_TAB[cents as usize + 300]
        } else if cents < 2100f32 {
            13.75f32 * CT2HZ_TAB[cents as usize - 900]
        } else if cents < 3300f32 {
            27.5f32 * CT2HZ_TAB[cents as usize - 2100]
        } else if cents < 4500f32 {
            55.0f32 * CT2HZ_TAB[cents as usize - 3300]
        } else if cents < 5700f32 {
            110.0f32 * CT2HZ_TAB[cents as usize - 4500]
        } else if cents < 6900f32 {
            220.0f32 * CT2HZ_TAB[cents as usize - 5700]
        } else if cents < 8100f32 {
            440.0f32 * CT2HZ_TAB[cents as usize - 6900]
        } else if cents < 9300f32 {
            880.0f32 * CT2HZ_TAB[cents as usize - 8100]
        } else if cents < 10500f32 {
            1760.0f32 * CT2HZ_TAB[cents as usize - 9300]
        } else if cents < 11700f32 {
            3520.0f32 * CT2HZ_TAB[cents as usize - 10500]
        } else if cents < 12900f32 {
            7040.0f32 * CT2HZ_TAB[cents as usize - 11700]
        } else if cents < 14100f32 {
            14080.0f32 * CT2HZ_TAB[cents as usize - 12900]
        } else {
            1.0f32
        }
    };
}

pub fn fluid_ct2hz(mut cents: f32) -> f32 {
    if cents >= 13500f32 {
        cents = 13500f32
    } else if cents < 1500f32 {
        cents = 1500f32
    }
    return fluid_ct2hz_real(cents);
}

pub fn fluid_cb2amp(cb: f32) -> f32 {
    return if cb < 0f32 {
        1.0f32
    } else if cb >= 961f32 {
        0.0f32
    } else {
        CB2AMP_TAB[cb as i32 as usize]
    };
}

pub fn fluid_atten2amp(atten: f32) -> f32 {
    return if atten < 0f32 {
        1.0f32
    } else if atten >= 1441f32 {
        0.0f32
    } else {
        ATTEN2AMP_TAB[atten as i32 as usize]
    };
}

pub fn fluid_tc2sec_delay(mut tc: f32) -> f32 {
    if tc <= -32768.0f32 {
        return 0.0f32;
    }
    if tc < -12000.0f32 {
        tc = -12000.0f32
    }
    if tc > 5000.0f32 {
        tc = 5000.0f32
    }
    return f64::powf(2.0f64, tc as f64 / 1200.0f64) as f32;
}

pub fn fluid_tc2sec_attack(mut tc: f32) -> f32 {
    if tc <= -32768.0f32 {
        return 0.0f32;
    }
    if tc < -12000.0f32 {
        tc = -12000.0f32
    }
    if tc > 8000.0f32 {
        tc = 8000.0f32
    }
    return f64::powf(2.0f64, tc as f64 / 1200.0f64) as f32;
}

pub fn fluid_tc2sec(tc: f32) -> f32 {
    return f64::powf(2.0f64, tc as f64 / 1200.0f64) as f32;
}

pub fn fluid_tc2sec_release(mut tc: f32) -> f32 {
    if tc <= -32768.0f32 {
        return 0.0f32;
    }
    if tc < -12000.0f32 {
        tc = -12000.0f32
    }
    if tc > 8000.0f32 {
        tc = 8000.0f32
    }
    return f64::powf(2.0f64, tc as f64 / 1200.0f64) as f32;
}

pub fn fluid_act2hz(c: f32) -> f32 {
    return (8.176f64 * f64::powf(2.0f64, c as f64 / 1200.0f64)) as f32;
}

pub fn fluid_pan(mut c: f32, left: i32) -> f32 {
    if left != 0 {
        c = -c
    }

    return if c < -500f32 {
        0f32
    } else if c > 500f32 {
        1f32
    } else {
        PAN_TAB[(c + 500f32) as usize]
    };
}

pub fn fluid_concave(val: f32) -> f32 {
    return if val < 0f32 {
        0f32
    } else if val > 127f32 {
        1f32
    } else {
        CONCAVE_TAB[val as i32 as usize]
    };
}

pub fn fluid_convex(val: f32) -> f32 {
    return if val < 0f32 {
        0f32
    } else if val > 127f32 {
        1f32
    } else {
        CONVEX_TAB[val as i32 as usize]
    };
}
