use std::sync::LazyLock;

static CT2HZ_TAB: LazyLock<[f32; 1200]> = LazyLock::new(|| {
    std::array::from_fn(|i| {
        let i = i as f64;

        f64::powf(2.0, i / 1200.0) as f32
    })
});

static CB2AMP_TAB: LazyLock<[f32; 961]> = LazyLock::new(|| {
    std::array::from_fn(|i| {
        let i = i as f64;

        f64::powf(10.0, i / -200.0) as f32
    })
});

static ATTEN2AMP_TAB: LazyLock<[f32; 1441]> = LazyLock::new(|| {
    std::array::from_fn(|i| {
        let i = i as f64;

        f64::powf(10.0, i / -200.0) as f32
    })
});

static CONCAVE_TAB: LazyLock<[f32; 128]> = LazyLock::new(|| {
    let mut init = [0.0; 128];
    init[0] = 0.0;
    init[127] = 1.0;
    let mut x: f64;
    for i in 1..127 {
        x = -20.0f64 / 96.0 * f64::ln((i * i) as f64 / (127.0 * 127.0)) / f64::ln(10.0);
        init[127 - i] = x as f32;
    }
    init
});

static CONVEX_TAB: LazyLock<[f32; 128]> = LazyLock::new(|| {
    let mut init = [0.0; 128];
    init[0] = 0.0;
    init[127] = 1.0;

    #[allow(clippy::needless_range_loop)]
    for i in 1..127 {
        let x = -20.0 / 96.0 * f64::ln((i * i) as f64 / (127.0 * 127.0)) / f64::ln(10.0);
        init[i] = (1.0 - x) as f32;
    }

    init
});

static PAN_TAB: LazyLock<[f32; 1002]> = LazyLock::new(|| {
    const X: f64 = std::f64::consts::PI / 2.0 / (1002.0 - 1.0);

    std::array::from_fn(|i| {
        let i = i as f64;

        f64::sin(i * X) as f32
    })
});

pub fn ct2hz_real(cents: f32) -> f32 {
    if cents < 0.0 {
        1.0
    } else if cents < 900.0 {
        6.875 * CT2HZ_TAB[cents as usize + 300]
    } else if cents < 2100.0 {
        13.75 * CT2HZ_TAB[cents as usize - 900]
    } else if cents < 3300.0 {
        27.5 * CT2HZ_TAB[cents as usize - 2100]
    } else if cents < 4500.0 {
        55.0 * CT2HZ_TAB[cents as usize - 3300]
    } else if cents < 5700.0 {
        110.0 * CT2HZ_TAB[cents as usize - 4500]
    } else if cents < 6900.0 {
        220.0 * CT2HZ_TAB[cents as usize - 5700]
    } else if cents < 8100.0 {
        440.0 * CT2HZ_TAB[cents as usize - 6900]
    } else if cents < 9300.0 {
        880.0 * CT2HZ_TAB[cents as usize - 8100]
    } else if cents < 10500.0 {
        1760.0 * CT2HZ_TAB[cents as usize - 9300]
    } else if cents < 11700.0 {
        3520.0 * CT2HZ_TAB[cents as usize - 10500]
    } else if cents < 12900.0 {
        7040.0 * CT2HZ_TAB[cents as usize - 11700]
    } else if cents < 14100.0 {
        14080.0 * CT2HZ_TAB[cents as usize - 12900]
    } else {
        1.0
    }
}

pub fn ct2hz(cents: f32) -> f32 {
    ct2hz_real(cents.clamp(1500.0, 13500.0))
}

pub fn cb2amp(cb: f32) -> f32 {
    if cb < 0.0 {
        1.0
    } else if cb >= 961.0 {
        0.0
    } else {
        CB2AMP_TAB[cb as i32 as usize]
    }
}

pub fn atten2amp(atten: f32) -> f32 {
    if atten < 0.0 {
        1.0
    } else if atten >= 1441.0 {
        0.0
    } else {
        ATTEN2AMP_TAB[atten as i32 as usize]
    }
}

pub fn tc2sec_delay(tc: f32) -> f32 {
    if tc <= -32768.0 {
        0.0
    } else {
        let tc = tc.clamp(-12000.0, 5000.0) as f64;
        f64::powf(2.0, tc / 1200.0) as f32
    }
}

pub fn tc2sec_attack(tc: f32) -> f32 {
    if tc <= -32768.0 {
        0.0
    } else {
        let tc = tc.clamp(-12000.0, 8000.0) as f64;
        f64::powf(2.0, tc / 1200.0) as f32
    }
}

pub fn tc2sec(tc: f64) -> f64 {
    f64::powf(2.0, tc / 1200.0)
}

pub fn tc2sec_release(tc: f32) -> f32 {
    if tc <= -32768.0 {
        0.0
    } else {
        let tc = tc.clamp(-12000.0, 8000.0) as f64;
        f64::powf(2.0, tc / 1200.0f64) as f32
    }
}

pub fn act2hz(c: f32) -> f32 {
    (8.176 * f64::powf(2.0, c as f64 / 1200.0f64)) as f32
}

pub fn pan(mut c: f32, left: i32) -> f32 {
    if left != 0 {
        c = -c
    }

    if c < -500.0 {
        0.0
    } else if c > 500.0 {
        1.0
    } else {
        PAN_TAB[(c + 500.0) as usize]
    }
}

pub fn concave(val: f32) -> f32 {
    if val < 0.0 {
        0.0
    } else if val > 127.0 {
        1.0
    } else {
        CONCAVE_TAB[val as usize]
    }
}

pub fn convex(val: f32) -> f32 {
    if val < 0.0 {
        0.0
    } else if val > 127.0 {
        1.0
    } else {
        CONVEX_TAB[val as usize]
    }
}
