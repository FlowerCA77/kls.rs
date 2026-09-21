use std::any::Any;
use std::io;
use std::time::{self, UNIX_EPOCH};

#[unsafe(no_mangle)]
pub extern "C" fn kls_printd(x: f64) -> f64 {
    println!("{}", x);
    x
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_eprintd(x: f64) -> f64 {
    eprintln!("{}", x);
    x
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_putchard(x: f64) -> f64 {
    println!("{}", x as u8 as char);
    x
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_eputchard(x: f64) -> f64 {
    eprintln!("{}", x as u8 as char);
    x
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_readd() -> f64 {
    let mut buf = String::new();
    if let Ok(_) = io::stdin().read_line(&mut buf)
        && let Ok(x) = buf.trim().parse::<f64>()
    {
        x
    } else {
        eprintln!("cannot read input {}", buf);
        0.0
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_floor(x: f64) -> f64 {
    f64::floor(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_ceil(x: f64) -> f64 {
    f64::ceil(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_round(x: f64) -> f64 {
    f64::round(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_trunc(x: f64) -> f64 {
    f64::trunc(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_div_euclid(x: f64, y: f64) -> f64 {
    x.div_euclid(y)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_rem_euclid(x: f64, y: f64) -> f64 {
    x.rem_euclid(y)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_abs(x: f64) -> f64 {
    f64::abs(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_pow(x: f64, y: f64) -> f64 {
    x.powf(y)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_sin(x: f64) -> f64 {
    f64::sin(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_cos(x: f64) -> f64 {
    f64::cos(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_tan(x: f64) -> f64 {
    f64::tan(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_sinh(x: f64) -> f64 {
    f64::sinh(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_cosh(x: f64) -> f64 {
    f64::cosh(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_tanh(x: f64) -> f64 {
    f64::tanh(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_exp(x: f64) -> f64 {
    f64::exp(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_exp2(x: f64) -> f64 {
    f64::exp2(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_log(x: f64, y: f64) -> f64 {
    x.log(y)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_log2(x: f64) -> f64 {
    f64::log2(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_log10(x: f64) -> f64 {
    f64::log10(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_ln(x: f64) -> f64 {
    f64::ln(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_sqrt(x: f64) -> f64 {
    f64::sqrt(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_cbrt(x: f64) -> f64 {
    f64::cbrt(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_asin(x: f64) -> f64 {
    f64::asin(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_acos(x: f64) -> f64 {
    f64::acos(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_atan(x: f64) -> f64 {
    f64::atan(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_atan2(x: f64, y: f64) -> f64 {
    f64::atan2(x, y)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_hypot(x: f64, y: f64) -> f64 {
    f64::hypot(x, y)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_asinh(x: f64) -> f64 {
    f64::asinh(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_acosh(x: f64) -> f64 {
    f64::acosh(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_atanh(x: f64) -> f64 {
    f64::atanh(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_recip(x: f64) -> f64 {
    f64::recip(x)
}

/// Kadlec, Jan (2010). "Řrřlog::Improving the fast inverse square root" (personal blog). Archived from the original on 2018-07-09. Retrieved 2020-12-14.
#[unsafe(no_mangle)]
pub extern "C" fn kls_qrsqrt(x: f64) -> f64 {
    let mut u = (x as f32).to_bits() as i32;
    u = 0x5F1FFFF9 - (u >> 1);

    let mut f = f32::from_bits(u as u32);
    f *= 0.703952253 * (2.38924456 - x as f32 * f * f);

    f as f64
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_clock_s() -> f64 {
    time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_clock_ms() -> f64 {
    time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as f64
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_clock_us() -> f64 {
    time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as f64
}

#[unsafe(no_mangle)]
pub extern "C" fn kls_clock_ns() -> f64 {
    time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as f64
}

#[used]
static KEEP: &[&(dyn Any + Send + Sync)] = &[
    &(kls_clock_ms as extern "C" fn() -> f64),
    &(kls_clock_ns as extern "C" fn() -> f64),
    &(kls_clock_s as extern "C" fn() -> f64),
    &(kls_clock_us as extern "C" fn() -> f64),
    &(kls_readd as extern "C" fn() -> f64),
    &(kls_abs as extern "C" fn(f64) -> f64),
    &(kls_acos as extern "C" fn(f64) -> f64),
    &(kls_acosh as extern "C" fn(f64) -> f64),
    &(kls_asin as extern "C" fn(f64) -> f64),
    &(kls_asinh as extern "C" fn(f64) -> f64),
    &(kls_atan as extern "C" fn(f64) -> f64),
    &(kls_atanh as extern "C" fn(f64) -> f64),
    &(kls_cbrt as extern "C" fn(f64) -> f64),
    &(kls_ceil as extern "C" fn(f64) -> f64),
    &(kls_cos as extern "C" fn(f64) -> f64),
    &(kls_cosh as extern "C" fn(f64) -> f64),
    &(kls_eprintd as extern "C" fn(f64) -> f64),
    &(kls_eputchard as extern "C" fn(f64) -> f64),
    &(kls_exp as extern "C" fn(f64) -> f64),
    &(kls_exp2 as extern "C" fn(f64) -> f64),
    &(kls_floor as extern "C" fn(f64) -> f64),
    &(kls_ln as extern "C" fn(f64) -> f64),
    &(kls_log10 as extern "C" fn(f64) -> f64),
    &(kls_log2 as extern "C" fn(f64) -> f64),
    &(kls_printd as extern "C" fn(f64) -> f64),
    &(kls_putchard as extern "C" fn(f64) -> f64),
    &(kls_qrsqrt as extern "C" fn(f64) -> f64),
    &(kls_recip as extern "C" fn(f64) -> f64),
    &(kls_round as extern "C" fn(f64) -> f64),
    &(kls_sin as extern "C" fn(f64) -> f64),
    &(kls_sinh as extern "C" fn(f64) -> f64),
    &(kls_sqrt as extern "C" fn(f64) -> f64),
    &(kls_tan as extern "C" fn(f64) -> f64),
    &(kls_tanh as extern "C" fn(f64) -> f64),
    &(kls_trunc as extern "C" fn(f64) -> f64),
    &(kls_atan2 as extern "C" fn(f64, f64) -> f64),
    &(kls_div_euclid as extern "C" fn(f64, f64) -> f64),
    &(kls_hypot as extern "C" fn(f64, f64) -> f64),
    &(kls_log as extern "C" fn(f64, f64) -> f64),
    &(kls_pow as extern "C" fn(f64, f64) -> f64),
    &(kls_rem_euclid as extern "C" fn(f64, f64) -> f64),
];
