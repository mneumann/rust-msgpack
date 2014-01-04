use std::cast;
use std::io::Reader;

#[inline(always)]
pub fn read_float(rd: &mut Reader) -> f32 {
  let v = rd.read_be_u32();
  unsafe { cast::transmute(v) }
}

#[inline(always)]
pub fn read_double(rd: &mut Reader) -> f64 {
  let v = rd.read_be_u64();
  unsafe { cast::transmute(v) }
}
