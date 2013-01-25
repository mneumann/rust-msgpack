extern mod std;

use core::io::{WriterUtil,ReaderUtil};
use core::path::Path;
use core::cast::reinterpret_cast;

use std::*;

use std::serialize::*;

pub struct Encoder {
  priv wr: io::Writer
}

#[inline(always)]
priv fn can_cast_i16_to_i8(v : i16) -> bool {
  const I: u16 = 0xFF80;
  ((v as u16) & I == 0) || ((v as u16) & I == I)
}

#[inline(always)]
priv fn can_cast_i32_to_i16(v : i32) -> bool {
  const I: u32 = 0xFFFF8000;
  ((v as u32) & I == 0) || ((v as u32) & I == I)
}

#[inline(always)]
priv fn can_cast_i64_to_i32(v : i64) -> bool {
  const I: u64 = 0xFFFFFFFF80000000;
  ((v as u64) & I == 0) || ((v as u64) & I == I)
}

impl Encoder {

  #[inline(always)]
  fn _emit_u8(&self, v: u8) {
    if v & 0x80 != 0 {
      self.wr.write_u8(0xcc);
    }
    self.wr.write_u8(v);
  }
 
  #[inline(always)]
  fn _emit_u16(&self, v: u16) {
    if v & 0xFF00 != 0 {
      self.wr.write_u8(0xcd);
      self.wr.write_be_u16(v);
    }
    else {
      self._emit_u8(v as u8);
    }
  }

  #[inline(always)]
  fn _emit_u32(&self, v: u32) {
    if v & 0xFFFF0000 != 0 {
      self.wr.write_u8(0xce);
      self.wr.write_be_u32(v);
    }
    else {
      self._emit_u16(v as u16);
    }
  }

  #[inline(always)]
  fn _emit_u64(&self, v: u64) {
    if v & 0xFFFFFFFF00000000 != 0 {
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

  fn emit_map_len(len: uint) {
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

pub impl Encoder: serialize::Encoder {
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
    unsafe { self.wr.write_be_u32(reinterpret_cast(&v)); }
  }

  fn emit_f64(&self, v: f64) {
    self.wr.write_u8(0xcb);
    unsafe { self.wr.write_be_u64(reinterpret_cast(&v)); }
  }

  fn emit_float(&self, v: float) {
    self.emit_f64(v as f64); // XXX
  }

  //
  // Strings
  //

  fn emit_borrowed_str(&self, v: &str) {
    self._emit_raw_len(str::len(v));
    self.wr.write_str(v);   
  }

  fn emit_owned_str(&self, v: &str) {
    self.emit_borrowed_str(v);
  }

  fn emit_managed_str(&self, v: &str) {
    self.emit_borrowed_str(v);
  }

  fn emit_char(&self, v: char) {
    self.emit_borrowed_str(str::from_char(v));
  }

  //
  // Vectors
  //

  fn emit_borrowed_vec(&self, len: uint, f: fn()) {
    self._emit_vec_len(len);
    f();
  }

  fn emit_owned_vec(&self, len: uint, f: fn()) {
    self.emit_borrowed_vec(len, f);
  }

  fn emit_managed_vec(&self, len: uint, f: fn()) {
    self.emit_borrowed_vec(len, f);
  }

  fn emit_vec_elt(&self, idx: uint, f: fn()) {
    f();
  }

  //
  // Other
  //

  fn emit_rec(&self, f: fn()) {
    fail ~"Records not supported";
  }

  fn emit_struct(&self, _name: &str, len: uint, f: fn()) {
    self._emit_vec_len(len);
    f();
  }

  fn emit_field(&self, name: &str, idx: uint, f: fn()) {
    f();
  }

  fn emit_tup(&self, len: uint, f: fn()) {
    self._emit_vec_len(len);
    f();
  }

  fn emit_tup_elt(&self, idx: uint, f: fn()) {
    f();
  }

  // XXX
  fn emit_borrowed(&self, f: fn()) { f(); }
  fn emit_owned(&self, f: fn()) { f(); }
  fn emit_managed(&self, f: fn()) { f(); }

  fn emit_enum(&self, _name: &str, f: fn()) {
    fail ~"enum not supported";
  }

  fn emit_enum_variant(&self, _name: &str, id: uint, _cnt: uint, f: fn()) {
    fail ~"enum not supported";
  }

  fn emit_enum_variant_arg(&self, _idx: uint, f: fn()) {
    fail ~"enum not supported";
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
}

struct Parser {
  priv rd: io::Reader
}

enum Error {
  Eof,
  Reserved,
  Invalid,
  Fatal
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

impl Parser {

  priv fn parse_array(len: uint) -> Value {
    Array(vec::from_fn(len, |_| { self.parse_value() }))
  }

  priv fn parse_map(len: uint) -> Value {
    Map(vec::from_fn(len, |_| { (self.parse_value(), self.parse_value()) }))
  }

  priv fn parse_raw(len: uint) -> Value {
    Raw(self.rd.read_bytes(len))
  }

  priv fn conv_float(v: u32) -> f32 { unsafe { reinterpret_cast(&v) } }
  priv fn conv_double(v: u64) -> f64 { unsafe { reinterpret_cast(&v) } }

  fn parse() -> Value {
    self.parse_value()
  }

  priv fn parse_value() -> Value {
    let c = self.rd.read_byte();
    if (c < 0) {
      fail;
    }
    match (c as u8) {
      0x00 .. 0x7f => Uint(c as u64),
      0x80 .. 0x8f => self.parse_map(c as uint & 0x0F),
      0x90 .. 0x9f => self.parse_array(c as uint & 0x0F),
      0xa0 .. 0xbf => self.parse_raw(c as uint & 0x1F),
      0xc0         => Nil,
      0xc1         => fail ~"Reserved",
      0xc2         => Bool(false),
      0xc3         => Bool(true),
      0xc4 .. 0xc9 => fail ~"Reserved",
      0xca         => Float(self.conv_float(self.rd.read_be_u32())),
      0xcb         => Double(self.conv_double(self.rd.read_be_u64())),
      0xcc         => Uint(self.rd.read_u8() as u64),
      0xcd         => Uint(self.rd.read_be_u16() as u64),
      0xce         => Uint(self.rd.read_be_u32() as u64),
      0xcf         => Uint(self.rd.read_be_u64()),
      0xd0         => Int(self.rd.read_i8() as i64),
      0xd1         => Int(self.rd.read_be_i16() as i64),
      0xd2         => Int(self.rd.read_be_i32() as i64),
      0xd3         => Int(self.rd.read_be_i64()),
      0xd4 .. 0xd9 => fail ~"Reserved",
      0xda         => self.parse_raw(self.rd.read_be_u16() as uint),
      0xdb         => self.parse_raw(self.rd.read_be_u32() as uint),
      0xdc         => self.parse_array(self.rd.read_be_u16() as uint),
      0xdd         => self.parse_array(self.rd.read_be_u32() as uint),
      0xde         => self.parse_map(self.rd.read_be_u16() as uint),
      0xdf         => self.parse_map(self.rd.read_be_u32() as uint),
      0xe0 .. 0xff => Int((c as i8) as i64),
      _            => fail ~"Invalid"
    }
  }

  fn parse_all() -> ~[Value] {
    let mut arr = ~[];
    while !self.rd.eof() {
      arr.push(self.parse_value())
    }
    return arr;
  }

}

fn doit(bytes: &[u8]) -> Value {
  let br = io::BytesReader { bytes: bytes, pos: 0 };
  let parser = Parser { rd: br as io::Reader };
  parser.parse()
}

#[auto_encode]
struct Blah {
  f: uint,
  g: uint,
  i: ~str,
  a: ~[uint]
}

/*
pub impl<S: serialize::Encoder> Blah: serialize::Encodable<S> {
  fn encode(&self, s: &S) {
    do s.emit_borrowed_vec(2) {
      s.emit_uint(self.f);
      s.emit_uint(self.g);
    }
  }
}
*/

fn main() {
  let res = io::buffered_file_writer(&Path("test.msgpack"));
  if res.is_ok() {
    let enc = Encoder { wr: res.get() };
    let blah = Blah { f: 1, g: 2, i: ~"hallo", a: ~[] }; 
    let blub = Blah { f: 5, g: 1, i: ~"nochwas", a: ~[1,2,3] }; 
    let b = ~[blah, blub];
    b.encode(&enc);
    5.encode(&enc);
  }


/*
  let res = io::buffered_file_writer(&Path("test.msgpack"));
  if res.is_ok() {
    l]et ser = Serializer { wr: res.get() };

    ser.emit_array_len(10);
    let mut i = 0;
    while i < 10 {
      i += 1;
      ser.emit_int(i);
    }

    ser.emit_u8(123);

    ser.emit_u8(6666);

    ser.emit_i8(-1);
    ser.emit_i16(-12343);

    ser.emit_u16(12334);
    ser.emit_i32(-4444444999);

    ser.emit_uint(12323232);

    ser.emit_int(-100000);

    ser.emit_bool(true);
    ser.emit_bool(false);
    ser.emit_nil();

    ser.emit_f32(12333.5);
    ser.emit_f64(12333.1239999);

    ser.emit_str("test");

    let r: &[const u8] = [1, 2, 3, 4];
    ser.emit_raw(r);

    ser.emit_map_len(2);
    ser.emit_str("hallooooo");
    ser.emit_uint(123);

    ser.emit_str("test");
    ser.emit_int(-123);
  }
*/

  //let res = io::file_reader(&Path("/tmp/matching.msgpack"));
  //doit(res.get());

  //let bytes: ~[u8] = result::unwrap(io::read_whole_file(&Path("Matching.msgpack")));
  //doit(bytes);

  /*let bytes: &[u8] = io::read_whole_file(&Path("/tmp/matching.msgpack")).get();
  let br = io::BytesReader { bytes: bytes, pos: 0 };
  let parser = Parser { rd: br as io::Reader };
  let bidding = parser.parse();
*/
  //let bytes = io::read_whole_file(&Path("/tmp/matching.msgpack")).get();
  //doit(bytes);

  //if res.is_ok() {
    //let parser = Parser { rd: res.get() };
    //io::println(fmt!("%?", bidding));
  //}
 
}
