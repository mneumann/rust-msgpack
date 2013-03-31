#[link(name = "msgpack",
       vers = "0.1",
       uuid = "812ef35d-c3f2-4d94-9199-a0746bfa346e")];
#[crate_type = "lib"];

extern mod std;

use core::io::{WriterUtil,ReaderUtil};
use std::serialize;

pub struct Encoder {
  wr: @io::Writer
}

pub struct Decoder {
  rd: @io::Reader
}

#[inline(always)]
priv fn can_cast_i16_to_i8(v : i16) -> bool {
  let I: u16 = 0xFF80;
  ((v as u16) & I == 0) || ((v as u16) & I == I)
}

#[inline(always)]
priv fn can_cast_i32_to_i16(v : i32) -> bool {
  let I: u32 = 0xFFFF8000;
  ((v as u32) & I == 0) || ((v as u32) & I == I)
}

#[inline(always)]
priv fn can_cast_i64_to_i32(v : i64) -> bool {
  let I: u64 = 0xFFFFFFFF80000000;
  ((v as u64) & I == 0) || ((v as u64) & I == I)
}

#[inline(always)]
priv fn can_cast_u64_to_u8(v : u64) -> bool {
  (v & 0xFFFFFFFFFFFFFF00 == 0)
}

#[inline(always)]
priv fn can_cast_u64_to_u16(v : u64) -> bool {
  (v & 0xFFFFFFFFFFFF0000 == 0)
}

#[inline(always)]
priv fn can_cast_u64_to_u32(v : u64) -> bool {
  (v & 0xFFFFFFFF00000000 == 0)
}

#[inline(always)]
priv fn can_cast_u32_to_u8(v : u32) -> bool {
  (v & 0xFFFFFF00 == 0)
}

#[inline(always)]
priv fn can_cast_u32_to_u16(v : u32) -> bool {
  (v & 0xFFFF0000 == 0)
}

#[inline(always)]
priv fn can_cast_u16_to_u8(v : u16) -> bool {
  (v & 0xFF00 == 0)
}

#[inline(always)]
priv fn conv_float(v: u32) -> f32 { unsafe { cast::transmute(v) } }

#[inline(always)]
priv fn conv_double(v: u64) -> f64 { unsafe { cast::transmute(v) } }

pub impl Encoder {
  fn new(wr: @io::Writer) -> Encoder { Encoder { wr: wr } }

  #[inline(always)]
  fn _emit_u8(&self, v: u8) {
    if v & 0x80 != 0 {
      self.wr.write_u8(0xcc);
    }
    self.wr.write_u8(v);
  }
 
  #[inline(always)]
  fn _emit_u16(&self, v: u16) {
    if !can_cast_u16_to_u8(v) {
      self.wr.write_u8(0xcd);
      self.wr.write_be_u16(v);
    }
    else {
      self._emit_u8(v as u8);
    }
  }

  #[inline(always)]
  fn _emit_u32(&self, v: u32) {
    if !can_cast_u32_to_u16(v) {
      self.wr.write_u8(0xce);
      self.wr.write_be_u32(v);
    }
    else {
      self._emit_u16(v as u16);
    }
  }

  #[inline(always)]
  fn _emit_u64(&self, v: u64) {
    if !can_cast_u64_to_u8(v) {
      self.wr.write_u8(0xcf);
      self.wr.write_be_u64(v);
    }
    else {
      self._emit_u32(v as u32);
    }
  }

  #[inline(always)]
  fn _emit_i8(&self, v: i8) {
    if (v as u8) & 0xe0 != 0xe0 {
      self.wr.write_u8(0xd0);
    }
    self.wr.write_u8(v as u8);
  }

  #[inline(always)]
  fn _emit_i16(&self, v: i16) {
    if !can_cast_i16_to_i8(v) {
      self.wr.write_u8(0xd1);
      self.wr.write_be_i16(v);
    }
    else {
      self._emit_i8(v as i8);
    }
  }

  #[inline(always)]
  fn _emit_i32(&self, v: i32) {
    if !can_cast_i32_to_i16(v) {
      self.wr.write_u8(0xd2);
      self.wr.write_be_i32(v);
    }
    else {
      self._emit_i16(v as i16);
    }
  }

  #[inline(always)]
  fn _emit_i64(&self, v: i64) {
    if !can_cast_i64_to_i32(v) {
      self.wr.write_u8(0xd3);
      self.wr.write_be_i64(v);
    }
    else {
      self._emit_i32(v as i32);
    }
  }

  priv fn _emit_raw_len(&self, len: uint) {
    if len <= 31 {
      self.wr.write_u8(0xa0 | (len as u8));
    } else if len <= 0xFFFF {
      self.wr.write_u8(0xda);
      self.wr.write_be_u16(len as u16);
    } else {
      self.wr.write_u8(0xdb);
      self.wr.write_be_u32(len as u32);
    }
  }

  priv fn _emit_raw(&self, v: &[const u8]) {
    self._emit_raw_len(vec::len(v));
    self.wr.write(v);
  }

  priv fn _emit_vec_len(&self, len: uint) {
    if len <= 15 {
      self.wr.write_u8(0x90 | (len as u8));
    } else if len <= 0xFFFF {
      self.wr.write_u8(0xdc);
      self.wr.write_be_u16(len as u16);
    } else {
      self.wr.write_u8(0xdd);
      self.wr.write_be_u32(len as u32);
    }
  }

  fn _emit_map_len(&self, len: uint) {
    if len <= 15 {
      self.wr.write_u8(0x80 | (len as u8));
    } else if len <= 0xFFFF {
      self.wr.write_u8(0xde);
      self.wr.write_be_u16(len as u16);
    } else {
      self.wr.write_u8(0xdf);
      self.wr.write_be_u32(len as u32);
    }
  }

}

impl serialize::Encoder for Encoder {
  //
  // Unsiged integers
  //

  fn emit_u8(&self, v: u8) {
    self._emit_u8(v)
  }
 
  fn emit_u16(&self, v: u16) {
    self._emit_u16(v)
  }

  fn emit_u32(&self, v: u32) {
    self._emit_u32(v)
  }

  fn emit_u64(&self, v: u64) {
    self._emit_u64(v)
  }

  fn emit_uint(&self, v: uint) {
    self._emit_u64(v as u64)
  }

  //
  // Signed integers
  //

  fn emit_i8(&self, v: i8) {
    self._emit_i8(v)
  }

  fn emit_i16(&self, v: i16) {
    self._emit_i16(v)
  }

  fn emit_i32(&self, v: i32) {
    self._emit_i32(v)
  }

  fn emit_i64(&self, v: i64) {
    self._emit_i64(v)
  }

  fn emit_int(&self, v: int) {
    self._emit_i64(v as i64);
  }

  //
  // Floating point
  //

  fn emit_f32(&self, v: f32) {
    self.wr.write_u8(0xca);
    unsafe { self.wr.write_be_u32(cast::transmute(v)); }
  }

  fn emit_f64(&self, v: f64) {
    self.wr.write_u8(0xcb);
    unsafe { self.wr.write_be_u64(cast::transmute(v)); }
  }

  fn emit_float(&self, v: float) {
    self.emit_f64(v as f64); // XXX
  }

  //
  // Strings
  //

  fn emit_str(&self, v: &str) {
    self._emit_raw_len(str::len(v));
    self.wr.write_str(v);   
  }

  fn emit_char(&self, v: char) {
    self.emit_str(str::from_char(v));
  }

  //
  // Vectors
  //

  fn emit_seq(&self, len: uint, f: &fn()) {
    self._emit_vec_len(len);
    f();
  }

  fn emit_seq_elt(&self, _idx: uint, f: &fn()) {
    f();
  }

  //
  // Other
  //

  fn emit_struct(&self, _name: &str, len: uint, f: &fn()) {
    self._emit_map_len(len);
    f();
  }

  fn emit_field(&self, _name: &str, _idx: uint, f: &fn()) {
    f();
  }

  fn emit_enum(&self, _name: &str, _f: &fn()) {
    fail!(~"enum not supported");
  }

  fn emit_enum_variant(&self, _name: &str, _id: uint, _cnt: uint, _f: &fn()) {
    fail!(~"enum not supported");
  }

  fn emit_enum_variant_arg(&self, _idx: uint, _f: &fn()) {
    fail!(~"enum not supported");
  }

  fn emit_nil(&self) {
    self.wr.write_u8(0xc0);
  }

  fn emit_bool(&self, v: bool) {
    if v {
      self.wr.write_u8(0xc3);
    } else {
      self.wr.write_u8(0xc2);
    }
  }

  fn emit_option(&self, _f: &fn()) { fail!() }
  fn emit_option_none(&self) { fail!() }
  fn emit_option_some(&self, _f: &fn()) { fail!() }

  fn emit_map(&self, len: uint, f: &fn()) {
    self._emit_map_len(len);
    f()
  }
  fn emit_map_elt_key(&self, _idx: uint, f: &fn()) { f() }
  fn emit_map_elt_val(&self, _idx: uint, f: &fn()) { f() }
}

pub impl Decoder {
  fn new(rd: @io::Reader) -> Decoder { Decoder { rd: rd } }

  #[inline(always)]
  fn _read_byte(&self) -> u8 {
    let c = self.rd.read_byte();
    if (c < 0) { fail!() }
    c as u8
  }

  #[inline(always)]
  fn _read_unsigned(&self) -> u64 {
    let c = self._read_byte();
    match c {
      0x00 .. 0x7f => c as u64,
      0xcc         => self.rd.read_u8() as u64,
      0xcd         => self.rd.read_be_u16() as u64,
      0xce         => self.rd.read_be_u32() as u64,
      0xcf         => self.rd.read_be_u64(),
      _            => fail!(~"No unsigned integer")
    }
  }

  #[inline(always)]
  fn _read_signed(&self) -> i64 {
    let c = self._read_byte();
    match c {
      0xd0         => self.rd.read_i8() as i64,
      0xd1         => self.rd.read_be_i16() as i64,
      0xd2         => self.rd.read_be_i32() as i64,
      0xd3         => self.rd.read_be_i64(),
      0xe0 .. 0xff => (c as i8) as i64,
      _            => fail!(~"No signed integer")
    }
  }

  #[inline(always)]
  fn _read_raw(&self, len: uint) -> ~[u8] {
    self.rd.read_bytes(len)
  }

  #[inline(always)]
  fn _read_str(&self, len: uint) -> ~str {
    unsafe {
      // XXX: add NUL byte!
      cast::transmute(self.rd.read_bytes(len))
      //str::from_bytes(self.rd.read_bytes(len))
    }
  }

  #[inline(always)]
  fn _read_vec_len(&self) -> uint {
    let c = self._read_byte();
    match c {
      0x90 .. 0x9f => c as uint & 0x0F,
      0xdc         => self.rd.read_be_u16() as uint,
      0xdd         => self.rd.read_be_u32() as uint,
      _            => fail!()
    }
  }

  #[inline(always)]
  fn _read_map_len(&self) -> uint {
    let c = self._read_byte();
    match c {
      0x80 .. 0x8f => c as uint & 0x0F,
      0xde         => self.rd.read_be_u16() as uint,
      0xdf         => self.rd.read_be_u32() as uint,
      _            => fail!()
    }
  }

  #[inline(always)]
  fn _read_elt_len(&self) -> uint {
    let c = self._read_byte();
    match c {
      0x80 .. 0x9f => c as uint & 0x0F,
      0xdc | 0xde  => self.rd.read_be_u16() as uint,
      0xdd | 0xdf  => self.rd.read_be_u32() as uint,
      _            => fail!()
    }
  }

}

impl serialize::Decoder for Decoder {
    #[inline(always)]
    fn read_nil(&self) -> () {
      if self.rd.read_byte() != 0xc0 { fail!() }
    }

    #[inline(always)]
    fn read_uint(&self) -> uint {
      self._read_unsigned() as uint // XXX
    }

    #[inline(always)]
    fn read_u64(&self) -> u64 { self._read_unsigned() }

    #[inline(always)]
    fn read_u32(&self) -> u32 {
      let v = self._read_unsigned();
      if !can_cast_u64_to_u32(v) { fail!() }
      v as u32
    }

    #[inline(always)]
    fn read_u16(&self) -> u16 {
      let v = self._read_unsigned();
      if !can_cast_u64_to_u16(v) { fail!() }
      v as u16
    }

    #[inline(always)]
    fn read_u8(&self) -> u8 {
      let v = self._read_unsigned();
      if !can_cast_u64_to_u8(v) { fail!() }
      v as u8
    }

    // XXX
    #[inline(always)]
    fn read_int(&self) -> int { self._read_signed() as int }
    #[inline(always)]
    fn read_i64(&self) -> i64 { self._read_signed() }
    #[inline(always)]
    fn read_i32(&self) -> i32 { self._read_signed() as i32 }
    #[inline(always)]
    fn read_i16(&self) -> i16 { self._read_signed() as i16 }
    #[inline(always)]
    fn read_i8(&self) -> i8 { self._read_signed() as i8 }

    #[inline(always)]
    fn read_bool(&self) -> bool {
      match self.rd.read_byte() {
        0xc2 => false,
        0xc3 => true,
        _    => fail!()
      }
    }

    #[inline(always)]
    fn read_f64(&self) -> f64 {
      match self.rd.read_byte() {
        0xcb => conv_double(self.rd.read_be_u64()),
        _ => fail!()
      }
    }

    #[inline(always)]
    fn read_f32(&self) -> f32 {
      match self.rd.read_byte() {
        0xca => conv_float(self.rd.read_be_u32()),
        _ => fail!()
      }
    }

    #[inline(always)]
    fn read_float(&self) -> float {
      self.read_f64() as float // XXX
    }

    #[inline(always)]
    fn read_char(&self) -> char {
      fail!() // XXX
    }

    #[inline(always)]
    fn read_str(&self) -> ~str {
      let c = self._read_byte();
      match c {
        0xa0 .. 0xbf => self._read_str(c as uint & 0x1F),
        0xda         => self._read_str(self.rd.read_be_u16() as uint),
        0xdb         => self._read_str(self.rd.read_be_u32() as uint),
        _            => fail!()
      }
    }

    fn read_enum<T>(&self, _name: &str, _f: &fn() -> T) -> T { fail!() }
    fn read_enum_variant<T>(&self, _names: &[&str], _f: &fn(uint) -> T) -> T { fail!() }
    fn read_enum_variant_arg<T>(&self, _idx: uint, _f: &fn() -> T) -> T { fail!() }

    // XXX: In case of a map, the number of elements will be /2.
    #[inline(always)]
    fn read_seq<T>(&self, f: &fn(uint) -> T) -> T {
      f(self._read_elt_len())
    }
    
    #[inline(always)]
    fn read_seq_elt<T>(&self, _idx: uint, f: &fn() -> T) -> T { f() }

    #[inline(always)]
    fn read_struct<T>(&self, _name: &str, len: uint, f: &fn() -> T) -> T {
      if len != self._read_map_len() { fail!() }
      f()
    }

    #[inline(always)]
    fn read_field<T>(&self, _name: &str, _idx: uint, f: &fn() -> T) -> T {
      f()
    }

    fn read_option<T>(&self, _f: &fn(bool) -> T) -> T { fail!() }

    fn read_map<T>(&self, f: &fn(uint) -> T) -> T {
        f(self._read_map_len())
    }
    fn read_map_elt_key<T>(&self, _idx: uint, f: &fn() -> T) -> T { f() }
    fn read_map_elt_val<T>(&self, _idx: uint, f: &fn() -> T) -> T { f() }

}

enum Value {
  Nil,
  Bool(bool),
  Array(~[Value]),
  Map(~[(Value, Value)]),
  Int(i64),
  Uint(u64),
  Float(f32),
  Double(f64),
  Raw(~[u8])
}

impl Decoder {
  priv fn parse_array(&self, len: uint) -> Value {
    Array(vec::from_fn(len, |_| { self.parse() }))
  }

  priv fn parse_map(&self, len: uint) -> Value {
    Map(vec::from_fn(len, |_| { (self.parse(), self.parse()) }))
  }

  fn parse(&self) -> Value {
    let c = self.rd.read_byte();
    if (c < 0) {
      fail!()
    }
    match (c as u8) {
      0x00 .. 0x7f => Uint(c as u64),
      0x80 .. 0x8f => self.parse_map(c as uint & 0x0F),
      0x90 .. 0x9f => self.parse_array(c as uint & 0x0F),
      0xa0 .. 0xbf => Raw(self._read_raw(c as uint & 0x1F)),
      0xc0         => Nil,
      0xc1         => fail!(~"Reserved"),
      0xc2         => Bool(false),
      0xc3         => Bool(true),
      0xc4 .. 0xc9 => fail!(~"Reserved"),
      0xca         => Float(conv_float(self.rd.read_be_u32())),
      0xcb         => Double(conv_double(self.rd.read_be_u64())),
      0xcc         => Uint(self.rd.read_u8() as u64),
      0xcd         => Uint(self.rd.read_be_u16() as u64),
      0xce         => Uint(self.rd.read_be_u32() as u64),
      0xcf         => Uint(self.rd.read_be_u64()),
      0xd0         => Int(self.rd.read_i8() as i64),
      0xd1         => Int(self.rd.read_be_i16() as i64),
      0xd2         => Int(self.rd.read_be_i32() as i64),
      0xd3         => Int(self.rd.read_be_i64()),
      0xd4 .. 0xd9 => fail!(~"Reserved"),
      0xda         => Raw(self._read_raw(self.rd.read_be_u16() as uint)),
      0xdb         => Raw(self._read_raw(self.rd.read_be_u32() as uint)),
      0xdc         => self.parse_array(self.rd.read_be_u16() as uint),
      0xdd         => self.parse_array(self.rd.read_be_u32() as uint),
      0xde         => self.parse_map(self.rd.read_be_u16() as uint),
      0xdf         => self.parse_map(self.rd.read_be_u32() as uint),
      0xe0 .. 0xff => Int((c as i8) as i64),
      _            => fail!(~"Invalid")
    }
  }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::hashmap::linear::LinearMap;
    use std::serialize::{Decodable, Encodable};

    fn to_msgpack<T: Encodable<Encoder>>(t: &T) -> ~[u8] {
        do io::with_bytes_writer |wr| {
            let encoder = Encoder::new(wr);
            t.encode(&encoder);
        }
    }

    fn from_msgpack<T: Decodable<Decoder>>(bytes: ~[u8]) -> T {
        do io::with_bytes_reader(bytes) |rd| {
            let decoder = Decoder::new(rd);
            Decodable::decode(&decoder)
        }
    }

    #[test]
    fn test_circular_str() {
        let v = ~"abcdef";
        assert_eq!(copy v, from_msgpack(to_msgpack(&v)));
    }

    #[test]
    fn test_circular_int() {
        assert_eq!(123, from_msgpack(to_msgpack(&123)));
        assert_eq!(-123, from_msgpack(to_msgpack(&-123)));
    }

    #[test]
    fn test_circular_float() {
        let v = -1243.111;
        assert_eq!(copy v, from_msgpack(to_msgpack(&v)));
    }

    #[test]
    fn test_circular_bool() {
        assert_eq!(true, from_msgpack(to_msgpack(&true)));
        assert_eq!(false, from_msgpack(to_msgpack(&false)));
    }

    #[test]
    fn test_circular_list() {
        let v = ~[1, 2, 3];
        assert_eq!(copy v, from_msgpack(to_msgpack(&v)));
    }

    #[test]
    fn test_circular_map() {
        let mut v = LinearMap::new();
        v.insert(1, 2);
        v.insert(3, 4);
        assert_eq!(copy v, from_msgpack(to_msgpack(&v)));
    }
}
