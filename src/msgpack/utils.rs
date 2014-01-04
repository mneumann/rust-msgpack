use std::cast;

#[inline(always)]
pub fn conv_float(v: u32) -> f32 { unsafe { cast::transmute(v) } }

#[inline(always)]
pub fn conv_double(v: u64) -> f64 { unsafe { cast::transmute(v) } }
