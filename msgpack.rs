#[link(name = "msgpack",
       vers = "0.1",
       uuid = "812ef35d-c3f2-4d94-9199-a0746bfa346e")];
#[crate_type = "lib"];

extern mod std;
extern mod extra;

use std::io::{WriterUtil,ReaderUtil};
use extra::serialize;
use std::cast;
use std::vec;
use std::str;

pub struct Encoder {
  wr: @std::io::Writer
}

pub struct Decoder {
  rd: @std::io::Reader,
  next_byte: @mut Option<u8>,
}

#[inline(always)]
fn can_cast_i16_to_i8(v : i16) -> bool {
  let I: u16 = 0xFF80;
  ((v as u16) & I == 0) || ((v as u16) & I == I)
}

#[inline(always)]
fn can_cast_i32_to_i16(v : i32) -> bool {
  let I: u32 = 0xFFFF8000;
  ((v as u32) & I == 0) || ((v as u32) & I == I)
}

#[inline(always)]
fn can_cast_i64_to_i32(v : i64) -> bool {
  let I: u64 = 0xFFFFFFFF80000000;
  ((v as u64) & I == 0) || ((v as u64) & I == I)
}

#[inline(always)]
fn can_cast_u64_to_u8(v : u64) -> bool {
  (v & 0xFFFFFFFFFFFFFF00 == 0)
}

#[inline(always)]
fn can_cast_u64_to_u16(v : u64) -> bool {
  (v & 0xFFFFFFFFFFFF0000 == 0)
}

#[inline(always)]
fn can_cast_u64_to_u32(v : u64) -> bool {
  (v & 0xFFFFFFFF00000000 == 0)
}

#[inline(always)]
fn can_cast_u32_to_u8(v : u32) -> bool {
  (v & 0xFFFFFF00 == 0)
}

#[inline(always)]
fn can_cast_u32_to_u16(v : u32) -> bool {
  (v & 0xFFFF0000 == 0)
}

#[inline(always)]
fn can_cast_u16_to_u8(v : u16) -> bool {
  (v & 0xFF00 == 0)
}

#[inline(always)]
fn conv_float(v: u32) -> f32 { unsafe { cast::transmute(v) } }

#[inline(always)]
fn conv_double(v: u64) -> f64 { unsafe { cast::transmute(v) } }

impl Encoder {
  pub fn new(wr: @std::io::Writer) -> Encoder { 
    //println!("in encoder::new");
    Encoder { wr: wr } 
  }

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

  fn _emit_raw_len(&self, len: uint) {
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

  fn _emit_raw(&self, v: &[u8]) {
    self._emit_raw_len(v.len());
    self.wr.write(v);
  }

  fn _emit_vec_len(&self, len: uint) {
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

impl extra::serialize::Encoder for Encoder {
  //
  // Unsiged integers
  //

  fn emit_u8(&mut self, v: u8) {
    self._emit_u8(v)
  }
 
  fn emit_u16(&mut self, v: u16) {
    self._emit_u16(v)
  }

  fn emit_u32(&mut self, v: u32) {
    self._emit_u32(v)
  }

  fn emit_u64(&mut self, v: u64) {
    self._emit_u64(v)
  }

  fn emit_uint(&mut self, v: uint) {
    self._emit_u64(v as u64)
  }

  //
  // Signed integers
  //

  fn emit_i8(&mut self, v: i8) {
    self._emit_i8(v)
  }

  fn emit_i16(&mut self, v: i16) {
    self._emit_i16(v)
  }

  fn emit_i32(&mut self, v: i32) {
    self._emit_i32(v)
  }

  fn emit_i64(&mut self, v: i64) {
    self._emit_i64(v)
  }

  fn emit_int(&mut self, v: int) {
    self._emit_i64(v as i64);
  }

  //
  // Floating point
  //

  fn emit_f32(&mut self, v: f32) {
    self.wr.write_u8(0xca);
    unsafe { self.wr.write_be_u32(cast::transmute(v)); }
  }

  fn emit_f64(&mut self, v: f64) {
    self.wr.write_u8(0xcb);
    unsafe { self.wr.write_be_u64(cast::transmute(v)); }
  }

  fn emit_float(&mut self, v: float) {
    self.emit_f64(v as f64); // XXX
  }

  //
  // Strings
  //

  fn emit_str(&mut self, v: &str) {
    self._emit_raw_len(v.len());
    self.wr.write_str(v);   
  }

  fn emit_char(&mut self, v: char) {
    self.emit_str(str::from_char(v));
  }

  //
  // Vectors
  //

  fn emit_seq(&mut self, len: uint, f: &fn(&mut Encoder)) {
    self._emit_vec_len(len);
    f(self);
  }

  fn emit_seq_elt(&mut self, _idx: uint, f: &fn(&mut Encoder)) {
    f(self);
  }

  //
  // Other
  //

  fn emit_struct(&mut self, _name: &str, len: uint, f: &fn(&mut Encoder)) {
    self._emit_map_len(len);
    f(self);
  }

  fn emit_struct_field(&mut self, _name: &str, _idx: uint, f: &fn(&mut Encoder)) {
    f(self);
  }


  fn emit_enum(&mut self, _name: &str, _f: &fn(&mut Encoder)) {
    fail!(~"enum not supported");
  }

  fn emit_enum_struct_variant(&mut self, name: &str, id: uint, cnt: uint, f: &fn(&mut Encoder)) {
    self.emit_enum_variant(name, id, cnt, f);
  }

  fn emit_enum_struct_variant_field(&mut self, _name: &str, idx: uint, f: &fn(&mut Encoder)) {
  self.emit_enum_variant_arg(idx, f);
  }


  fn emit_enum_variant(&mut self, _name: &str, _id: uint, _cnt: uint, _f: &fn(&mut Encoder)) {
    fail!(~"enum not supported");
  }

  fn emit_enum_variant_arg(&mut self, _idx: uint, _f: &fn(&mut Encoder)) {
    fail!(~"enum not supported");
  }

  fn emit_nil(&mut self) {
    self.wr.write_u8(0xc0);
  }

  fn emit_bool(&mut self, v: bool) {
    if v {
      self.wr.write_u8(0xc3);
    } else {
      self.wr.write_u8(0xc2);
    }
  }

    fn emit_tuple(&mut self, len: uint, f: &fn(&mut Encoder)) {
        self.emit_seq(len, f)
    }
    fn emit_tuple_arg(&mut self, idx: uint, f: &fn(&mut Encoder)) {
        self.emit_seq_elt(idx, f)
    }

    fn emit_tuple_struct(&mut self,
                         _name: &str,
                         len: uint,
                         f: &fn(&mut Encoder)) {
        self.emit_seq(len, f)
    }
    fn emit_tuple_struct_arg(&mut self, idx: uint, f: &fn(&mut Encoder)) {
        self.emit_seq_elt(idx, f)
    }


  fn emit_option(&mut self, f: &fn(&mut Encoder)) { f(self) }
  fn emit_option_none(&mut self) { self.emit_nil() }
  fn emit_option_some(&mut self, f: &fn(&mut Encoder)) { f(self) }

  fn emit_map(&mut self, len: uint, f: &fn(&mut Encoder)) {
    self._emit_map_len(len);
    f(self)
  }
  fn emit_map_elt_key(&mut self, _idx: uint, f: &fn(&mut Encoder)) { f(self) }
  fn emit_map_elt_val(&mut self, _idx: uint, f: &fn(&mut Encoder)) { f(self) }

}

impl Decoder {
  pub fn new(rd: @std::io::Reader) -> Decoder {
    Decoder {
      rd: rd,
      next_byte: @mut None,
    }
  }
}

impl Decoder {
  #[inline(always)]
  fn _peek_byte(&mut self) -> u8 {
    match *self.next_byte {
      Some(byte) => byte,
      None => {
        let byte = self.rd.read_byte();
        if (byte < 0) { fail!() }
        let byte = byte as u8;
        *self.next_byte = Some(byte);
        byte
      }
    }
  }

  #[inline(always)]
  fn _read_byte(&mut self) -> u8 {
    match *self.next_byte {
      Some(byte) => { *self.next_byte = None; byte },
      None => {
        let byte = self.rd.read_byte();
        if (byte < 0) { fail!() }
        byte as u8
      }
    }
  }

  #[inline(always)]
  fn _read_unsigned(&mut self) -> u64 {
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
  fn _read_signed(&mut self) -> i64 {
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
  fn _read_raw(&mut self, len: uint) -> ~[u8] {
    self.rd.read_bytes(len)
  }

  #[inline(always)]
  fn _read_str(&mut self, len: uint) -> ~str {
    str::from_utf8(self.rd.read_bytes(len))
  }

  //[inline(always)]
  fn _read_vec_len(&mut self) -> uint {
      //printfln!("in _read_vec_len on self: %?", self);
    let c = self._read_byte();
      //printfln!("in _read_vec_len on byte c= %?", c);

    match c {
      0x90 .. 0x9f => c as uint & 0x0F,
      0xdc         => self.rd.read_be_u16() as uint,
      0xdd         => self.rd.read_be_u32() as uint,
	_            => fail!("unimplmeneted _read_vec_len() byte code: %?", c)
    }
  }

  #[inline(always)]
  fn _read_map_len(&mut self) -> uint {
    let c = self._read_byte();
    match c {
      0x80 .. 0x8f => c as uint & 0x0F,
      0xde         => self.rd.read_be_u16() as uint,
      0xdf         => self.rd.read_be_u32() as uint,
      _            => fail!()
    }
  }
}

impl serialize::Decoder for Decoder {
    #[inline(always)]
    fn read_nil(&mut self) -> () {
      if self._read_byte() != 0xc0 { fail!() }
    }

    #[inline(always)]
    fn read_uint(&mut self) -> uint {
      self._read_unsigned() as uint // XXX
    }

    #[inline(always)]
    fn read_u64(&mut self) -> u64 { self._read_unsigned() }

    #[inline(always)]
    fn read_u32(&mut self) -> u32 {
      let v = self._read_unsigned();
      if !can_cast_u64_to_u32(v) { fail!() }
      v as u32
    }

    #[inline(always)]
    fn read_u16(&mut self) -> u16 {
      let v = self._read_unsigned();
      if !can_cast_u64_to_u16(v) { fail!() }
      v as u16
    }

    #[inline(always)]
    fn read_u8(&mut self) -> u8 {
      let v = self._read_unsigned();
      if !can_cast_u64_to_u8(v) { fail!() }
      v as u8
    }

    // XXX
    #[inline(always)]
    fn read_int(&mut self) -> int { self._read_signed() as int }
    #[inline(always)]
    fn read_i64(&mut self) -> i64 { self._read_signed() }
    #[inline(always)]
    fn read_i32(&mut self) -> i32 { self._read_signed() as i32 }
    #[inline(always)]
    fn read_i16(&mut self) -> i16 { self._read_signed() as i16 }
    #[inline(always)]
    fn read_i8(&mut self) -> i8 { self._read_signed() as i8 }

    #[inline(always)]
    fn read_bool(&mut self) -> bool {
      match self._read_byte() {
        0xc2 => false,
        0xc3 => true,
        _    => fail!()
      }
    }

    #[inline(always)]
    fn read_f64(&mut self) -> f64 {
      match self._read_byte() {
        0xcb => conv_double(self.rd.read_be_u64()),
        _ => fail!()
      }
    }

    #[inline(always)]
    fn read_f32(&mut self) -> f32 {
      match self._read_byte() {
        0xca => conv_float(self.rd.read_be_u32()),
        _ => fail!()
      }
    }

    #[inline(always)]
    fn read_float(&mut self) -> float {
      self.read_f64() as float // XXX
    }

    #[inline(always)]
    fn read_char(&mut self) -> char {
      let s = self.read_str();
      if s.len() == 0 { fail!(~"no character") }
      s[0] as char
    }

    #[inline(always)]
    fn read_str(&mut self) -> ~str {
      let c = self._read_byte();
      match c {
        0xa0 .. 0xbf => self._read_str(c as uint & 0x1F),
        0xda         => {
	  let b : uint = self.rd.read_be_u16() as uint;
	  self._read_str(b)
	},
	0xdb         => {
	  let b : uint = self.rd.read_be_u32() as uint;
	  self._read_str(b)
	},
        _            => fail!()
      }
    }

    fn read_enum<T>(&mut self, _name: &str, _f: &fn(&mut Decoder) -> T) -> T { fail!() }
    fn read_enum_variant<T>(&mut self, _names: &[&str], _f: &fn(&mut Decoder, uint) -> T) -> T { fail!() }
    fn read_enum_variant_arg<T>(&mut self, _idx: uint, _f: &fn(&mut Decoder) -> T) -> T { fail!() }

    #[inline(always)]
    fn read_seq<T>(&mut self, f: &fn(&mut Decoder,uint) -> T) -> T {
      //println!("in read_seq.");
      let len = self._read_vec_len();
      //printfln!("I see length of %?", len);
      f(self, len)
    }
    
    #[inline(always)]
    fn read_seq_elt<T>(&mut self, _idx: uint, f: &fn(&mut Decoder) -> T) -> T { f(self) }

    #[inline(always)]
    fn read_struct<T>(&mut self, _name: &str, len: uint, f: &fn(&mut Decoder) -> T) -> T {
        //printfln!("in read_struct");
      if len != self._read_map_len() { fail!() }
      f(self)
    }

    #[inline(always)]
    fn read_struct_field<T>(&mut self, _name: &str, _idx: uint, f: &fn(&mut Decoder) -> T) -> T {
      f(self)
    }

    fn read_option<T>(&mut self, f: &fn(&mut Decoder, bool) -> T) -> T {
      match self._peek_byte() {
        0xc0 => f(self, false),
        _ => f(self, true)
      }
    }

    fn read_map<T>(&mut self, f: &fn(&mut Decoder, uint) -> T) -> T {
      let len = self._read_map_len();
      f(self, len)
    }
    fn read_map_elt_key<T>(&mut self, _idx: uint, f: &fn(&mut Decoder) -> T) -> T { f(self) }
    fn read_map_elt_val<T>(&mut self, _idx: uint, f: &fn(&mut Decoder) -> T) -> T { f(self) }


    fn read_enum_struct_variant<T>(&mut self,
                                   names: &[&str],
                                   f: &fn(&mut Decoder, uint) -> T)
                                   -> T {
        debug!("read_enum_struct_variant(names=%?)", names);
        self.read_enum_variant(names, f)
    }


    fn read_enum_struct_variant_field<T>(&mut self,
                                         name: &str,
                                         idx: uint,
                                         f: &fn(&mut Decoder) -> T)
                                         -> T {
        debug!("read_enum_struct_variant_field(name=%?, idx=%u)", name, idx);
        self.read_enum_variant_arg(idx, f)
    }

    fn read_tuple<T>(&mut self, f: &fn(&mut Decoder, uint) -> T) -> T {
        debug!("read_tuple()");
        self.read_seq(f)
    }

    fn read_tuple_arg<T>(&mut self,
                         idx: uint,
                         f: &fn(&mut Decoder) -> T)
                         -> T {
        debug!("read_tuple_arg(idx=%u)", idx);
        self.read_seq_elt(idx, f)
    }

    fn read_tuple_struct<T>(&mut self,
                            name: &str,
                            f: &fn(&mut Decoder, uint) -> T)
                            -> T {
        debug!("read_tuple_struct(name=%?)", name);
        self.read_tuple(f)
    }

    fn read_tuple_struct_arg<T>(&mut self,
                                idx: uint,
                                f: &fn(&mut Decoder) -> T)
                                -> T {
        debug!("read_tuple_struct_arg(idx=%u)", idx);
        self.read_tuple_arg(idx, f)
    }


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
  fn parse_array(&mut self, len: uint) -> Value {
    Array(vec::from_fn(len, |_| { self.parse() }))
  }

  fn parse_map(&mut self, len: uint) -> Value {
    Map(vec::from_fn(len, |_| { (self.parse(), self.parse()) }))
  }

  fn parse(&mut self) -> Value {
    let c = self.rd.read_byte();
    //printfln!("in parse: read value %x",c as uint);
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
      0xda         => { let b = self.rd.read_be_u16() as uint; Raw(self._read_raw(b)) },
      0xdb         => { let b = self.rd.read_be_u32() as uint; Raw(self._read_raw(b)) },
      0xdc         => { let b = self.rd.read_be_u16() as uint; self.parse_array(b) },
      0xdd         => { let b = self.rd.read_be_u32() as uint; self.parse_array(b) },
      0xde         => { let b = self.rd.read_be_u16() as uint; self.parse_map(b) },
      0xdf         => { let b = self.rd.read_be_u32() as uint; self.parse_map(b) },
      0xe0 .. 0xff => Int((c as i8) as i64),
      _            => fail!(~"Invalid")
    }
  }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hashmap::HashMap;
    use extra::serialize::{Decodable, Encodable};

    fn to_msgpack<T: Encodable<Encoder>>(t: &T) -> ~[u8] {
        do std::io::with_bytes_writer |wr| {
            let mut encoder = Encoder::new(wr);
            t.encode(&mut encoder);
        }
    }

    fn from_msgpack<T: Decodable<Decoder>>(bytes: ~[u8]) -> T {
        do std::io::with_bytes_reader(bytes) |rd| {
            let mut decoder = Decoder::new(rd);
            Decodable::decode(&mut decoder)
        }
    }

    #[test]
    fn test_circular_str() {
        assert_eq!(~"", from_msgpack(to_msgpack(&~"")));
        assert_eq!(~"a", from_msgpack(to_msgpack(&~"a")));
        assert_eq!(~"abcdef", from_msgpack(to_msgpack(&~"abcdef")));
    }

    #[test]
    fn test_circular_int() {
        assert_eq!(123, from_msgpack(to_msgpack(&123)));
        assert_eq!(-123, from_msgpack(to_msgpack(&-123)));
    }

    #[test]
    fn test_circular_float() {
        let v = -1243.111;
        assert_eq!(v, from_msgpack(to_msgpack(&v)));
    }
    #[test]
    fn test_circular_bool() {
        assert_eq!(true, from_msgpack(to_msgpack(&true)));
        assert_eq!(false, from_msgpack(to_msgpack(&false)));
    }

    #[test]
    fn test_circular_list() {
        let v = ~[1, 2, 3];
        assert_eq!(v.clone(), from_msgpack(to_msgpack(&v)));
    }

    #[test]
    fn test_circular_map() {
        let mut v = HashMap::new();
        v.insert(1, 2);
        v.insert(3, 4);
        assert_eq!(v.clone(), from_msgpack(to_msgpack(&v)));
    }

    #[test]
    fn test_circular_option() {
        let mut v = Some(1);
        let w : Option<int> = from_msgpack(to_msgpack(&v));

        assert_eq!(w, v);

        v = None;
        assert_eq!(v.clone(), from_msgpack(to_msgpack(&v)));
    }

    /*
    #[test]
    fn test_circular_char() {
      let a: char = str::char_at("a", 0);
      assert_eq!(a, from_msgpack(to_msgpack(&a)))
    }
    */
}
