use std::cast;

#[inline(always)]
pub fn can_cast_i16_to_i8(v : i16) -> bool {
  let I: u16 = 0xFF80;
  ((v as u16) & I == 0) || ((v as u16) & I == I)
}

#[inline(always)]
pub fn can_cast_i32_to_i16(v : i32) -> bool {
  let I: u32 = 0xFFFF8000;
  ((v as u32) & I == 0) || ((v as u32) & I == I)
}

#[inline(always)]
pub fn can_cast_i64_to_i32(v : i64) -> bool {
  let I: u64 = 0xFFFFFFFF80000000;
  ((v as u64) & I == 0) || ((v as u64) & I == I)
}

// TODO: Improve
#[inline(always)]
pub fn can_cast_i64_to_i16(v : i64) -> bool {
  can_cast_i64_to_i32(v) &&
  can_cast_i32_to_i16(v as i32)
}

// TODO: Improve
#[inline(always)]
pub fn can_cast_i64_to_i8(v : i64) -> bool {
  can_cast_i64_to_i32(v) &&
  can_cast_i32_to_i16(v as i32) &&
  can_cast_i16_to_i8(v as i16)
}

#[inline(always)]
pub fn can_cast_u64_to_u8(v : u64) -> bool {
  (v & 0xFFFFFFFFFFFFFF00 == 0)
}

#[inline(always)]
pub fn can_cast_u64_to_u16(v : u64) -> bool {
  (v & 0xFFFFFFFFFFFF0000 == 0)
}

#[inline(always)]
pub fn can_cast_u64_to_u32(v : u64) -> bool {
  (v & 0xFFFFFFFF00000000 == 0)
}

// XXX
#[inline(always)]
pub fn can_cast_u64_to_uint(v : u64) -> bool {
  true
}

// XXX
#[inline(always)]
pub fn can_cast_i64_to_int(v : i64) -> bool {
  true
}

#[inline(always)]
pub fn conv_float(v: u32) -> f32 { unsafe { cast::transmute(v) } }

#[inline(always)]
pub fn conv_double(v: u64) -> f64 { unsafe { cast::transmute(v) } }
