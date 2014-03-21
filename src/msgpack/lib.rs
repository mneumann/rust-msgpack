#[crate_id = "msgpack#0.1"];
#[comment = "msgpack.org implementation for Rust"];
#[license = "MIT/ASL2"];
#[crate_type = "lib"];
#[feature(struct_variant)];
#[allow(unused_must_use, dead_code)];

extern crate serialize = "serialize#0.10-pre";

use std::{io, str, slice, cast};
use std::str::from_utf8;
use std::io::{MemReader,MemWriter};

use serialize::{Encodable,Decodable};

mod rpc;

pub enum Value {
  Nil,
  Boolean(bool),
  Integer(i64),
  Unsigned(u64),
  Float(f32),
  Double(f64),
  Array(~[Value]),
  Map(~[(Value, Value)]),
  String(~[u8]),
  Binary(~[u8]),
  Extended(i8, ~[u8])
}

#[inline(always)]
fn read_float(rd: &mut io::Reader) -> f32 {
  let v = rd.read_be_u32().unwrap();
  unsafe { cast::transmute(v) }
}

#[inline(always)]
fn read_double(rd: &mut io::Reader) -> f64 {
  let v = rd.read_be_u64().unwrap();
  unsafe { cast::transmute(v) }
}

/// A structure to decode Msgpack from a reader.
pub struct Decoder<'a> {
  priv rd: &'a mut io::Reader,
  priv next_byte: Option<u8>
}

impl<'a> Decoder<'a> {
  /// Creates a new Msgpack decoder for decoding from the
  /// specified reader.
  pub fn new(rd: &'a mut io::Reader) -> Decoder<'a> {
    Decoder {
      rd: rd,
      next_byte: None
    }
  }
}

impl<'a> Decoder<'a> {
  fn _peek_byte(&mut self) -> u8 {
    match self.next_byte {
      Some(byte) => byte,
      None => {
        self.next_byte = self.rd.read_byte().ok();
        match self.next_byte {
          Some(byte) => byte,
          None => fail!("Unexpected EOF")
        }
      }
    }
  }

  fn _read_byte(&mut self) -> u8 {
    match self.next_byte {
      Some(byte) => {
        self.next_byte = None;
        byte
      }
      None => {
        match self.rd.read_byte().ok() {
          Some(byte) => byte,
          None => fail!("Unexpected EOF")
        }
      }
    }
  }

  fn _read_unsigned(&mut self) -> u64 {
    let c = self._read_byte();
    match c {
      0x00 .. 0x7f => c as u64,
      0xcc         => self.rd.read_u8().unwrap() as u64,
      0xcd         => self.rd.read_be_u16().unwrap() as u64,
      0xce         => self.rd.read_be_u32().unwrap() as u64,
      0xcf         => self.rd.read_be_u64().unwrap(),
      _            => fail!("No unsigned integer")
    }
  }

  fn _read_signed(&mut self) -> i64 {
    let c = self._read_byte();
    match c {
      0xd0         => self.rd.read_i8().unwrap() as i64,
      0xd1         => self.rd.read_be_i16().unwrap() as i64,
      0xd2         => self.rd.read_be_i32().unwrap() as i64,
      0xd3         => self.rd.read_be_i64().unwrap(),
      0xe0 .. 0xff => (c as i8) as i64,
      _            => fail!("No signed integer")
    }
  }

  fn _read_raw(&mut self, len: uint) -> ~[u8] {
    self.rd.read_bytes(len).unwrap()
  }

  fn _read_str(&mut self, len: uint) -> ~str {
    str::from_utf8_owned(self.rd.read_bytes(len).unwrap()).unwrap()
  }

  fn _read_vec_len(&mut self) -> uint {
    let c = self._read_byte();

    match c {
      0x90 .. 0x9f => (c as uint) & 0x0F,
      0xdc         => self.rd.read_be_u16().unwrap() as uint,
      0xdd         => self.rd.read_be_u32().unwrap() as uint,
      _            => fail!("Invalid byte code in _read_vec_len")
    }
  }

  fn _read_map_len(&mut self) -> uint {
    let c = self._read_byte();
    match c {
      0x80 .. 0x8f => (c as uint) & 0x0F,
      0xde         => self.rd.read_be_u16().unwrap() as uint,
      0xdf         => self.rd.read_be_u32().unwrap() as uint,
      _            => fail!("Invalid byte code in _read_map_len")
    }
  }

  fn decode_array(&mut self, len: uint) -> Value {
    Array(slice::from_fn(len, |_| { self.decode_value() }))
  }

  fn decode_map(&mut self, len: uint) -> Value {
    Map(slice::from_fn(len, |_| { (self.decode_value(), self.decode_value()) }))
  }

  fn decode_ext(&mut self, len: uint) -> Value {
    let typ = self.rd.read_i8().unwrap();
    if typ < 0 { fail!("Reserved type") }
    let data = self.rd.read_bytes(len).unwrap();
    Extended(typ, data)
  }

  fn decode_value(&mut self) -> Value {
    let c: u8 = self._read_byte();
    match c {
      0xc0         => Nil,

      0xc1         => fail!("Reserved"),

      0xc2         => Boolean(false),
      0xc3         => Boolean(true),

      0x00 .. 0x7f => Unsigned(c as u64),
      0xcc         => Unsigned(self.rd.read_u8().unwrap() as u64),
      0xcd         => Unsigned(self.rd.read_be_u16().unwrap() as u64),
      0xce         => Unsigned(self.rd.read_be_u32().unwrap() as u64),
      0xcf         => Unsigned(self.rd.read_be_u64().unwrap()),

      0xd0         => Integer(self.rd.read_i8().unwrap() as i64),
      0xd1         => Integer(self.rd.read_be_i16().unwrap() as i64),
      0xd2         => Integer(self.rd.read_be_i32().unwrap() as i64),
      0xd3         => Integer(self.rd.read_be_i64().unwrap()),
      0xe0 .. 0xff => Integer((c as i8) as i64),

      0xca         => Float(read_float(self.rd)),
      0xcb         => Double(read_double(self.rd)),

      0xa0 .. 0xbf => String(self._read_raw((c as uint) & 0x1F)),
      0xd9         => { let b = self.rd.read_u8().unwrap() as uint; String(self._read_raw(b)) },
      0xda         => { let b = self.rd.read_be_u16().unwrap() as uint; String(self._read_raw(b)) },
      0xdb         => { let b = self.rd.read_be_u32().unwrap() as uint; String(self._read_raw(b)) },

      0xc4         => { let b = self.rd.read_u8().unwrap() as uint; Binary(self._read_raw(b)) },
      0xc5         => { let b = self.rd.read_be_u16().unwrap() as uint; Binary(self._read_raw(b)) },
      0xc6         => { let b = self.rd.read_be_u32().unwrap() as uint; Binary(self._read_raw(b)) },

      0x90 .. 0x9f => self.decode_array((c as uint) & 0x0F),
      0xdc         => { let b = self.rd.read_be_u16().unwrap() as uint; self.decode_array(b) },
      0xdd         => { let b = self.rd.read_be_u32().unwrap() as uint; self.decode_array(b) },

      0x80 .. 0x8f => self.decode_map((c as uint) & 0x0F),
      0xde         => { let b = self.rd.read_be_u16().unwrap() as uint; self.decode_map(b) },
      0xdf         => { let b = self.rd.read_be_u32().unwrap() as uint; self.decode_map(b) },

      0xd4         => self.decode_ext(1),
      0xd5         => self.decode_ext(2),
      0xd6         => self.decode_ext(4),
      0xd7         => self.decode_ext(8),
      0xd8         => self.decode_ext(16),
      0xc7         => { let b = self.rd.read_u8().unwrap() as uint; self.decode_ext(b) },
      0xc8         => { let b = self.rd.read_be_u16().unwrap() as uint; self.decode_ext(b) },
      0xc9         => { let b = self.rd.read_be_u32().unwrap() as uint; self.decode_ext(b) },

      // XXX: This is only here to satify Rust's pattern checker.
      _            => unreachable!()
    }
  }


}

impl<'a> serialize::Decoder for Decoder<'a> {
    #[inline(always)]
    fn read_nil(&mut self) -> () {
      if self._read_byte() != 0xc0 { fail!("Invalid nil opcode") }
    }

    #[inline(always)]
    fn read_u64(&mut self) -> u64 { self._read_unsigned() }

    #[inline(always)]
    fn read_uint(&mut self) -> uint {
      self._read_unsigned().to_uint().unwrap()
    }

    #[inline(always)]
    fn read_u32(&mut self) -> u32 {
      self._read_unsigned().to_u32().unwrap()
    }

    #[inline(always)]
    fn read_u16(&mut self) -> u16 {
      self._read_unsigned().to_u16().unwrap()
    }

    #[inline(always)]
    fn read_u8(&mut self) -> u8 {
      self._read_unsigned().to_u8().unwrap()
    }

    #[inline(always)]
    fn read_i64(&mut self) -> i64 {
      self._read_signed()
    }

    #[inline(always)]
    fn read_int(&mut self) -> int {
      self._read_signed().to_int().unwrap()
    }

    #[inline(always)]
    fn read_i32(&mut self) -> i32 {
      self._read_signed().to_i32().unwrap()
    }

    #[inline(always)]
    fn read_i16(&mut self) -> i16 {
      self._read_signed().to_i16().unwrap()
    }

    #[inline(always)]
    fn read_i8(&mut self) -> i8 {
      self._read_signed().to_i8().unwrap()
    }

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
        0xcb => read_double(self.rd),
        _    => fail!()
      }
    }

    #[inline(always)]
    fn read_f32(&mut self) -> f32 {
      match self._read_byte() {
        0xca => read_float(self.rd),
        _    => fail!()
      }
    }

    #[inline(always)]
    fn read_char(&mut self) -> char {
      let s = self.read_str();
      if s.len() != 1 { fail!("no character") }
      s[0] as char
    }

    #[inline(always)]
    fn read_str(&mut self) -> ~str {
      let c = self._read_byte();
      match c {
        0xa0 .. 0xbf => self._read_str((c as uint) & 0x1F),
        0xda         => {
	  let b : uint = self.rd.read_be_u16().unwrap() as uint;
	  self._read_str(b)
	},
	0xdb         => {
	  let b : uint = self.rd.read_be_u32().unwrap() as uint;
	  self._read_str(b)
	},
        _            => fail!()
      }
    }

    fn read_enum<T>(&mut self, _name: &str, _f: |&mut Decoder<'a>| -> T) -> T { fail!() }
    fn read_enum_variant<T>(&mut self, _names: &[&str], _f: |&mut Decoder<'a>, uint| -> T) -> T { fail!() }
    fn read_enum_variant_arg<T>(&mut self, _idx: uint, _f: |&mut Decoder<'a>| -> T) -> T { fail!() }

    #[inline(always)]
    fn read_seq<T>(&mut self, f: |&mut Decoder<'a>, uint| -> T) -> T {
      let len = self._read_vec_len();
      f(self, len)
    }

    #[inline(always)]
    fn read_seq_elt<T>(&mut self, _idx: uint, f: |&mut Decoder<'a>| -> T) -> T {
      f(self)
    }

    #[inline(always)]
    fn read_struct<T>(&mut self, _name: &str, len: uint, f: |&mut Decoder<'a>| -> T) -> T {
      // XXX: Why are we using a map length here?
      if len != self._read_map_len() { fail!() }
      f(self)
    }

    #[inline(always)]
    fn read_struct_field<T>(&mut self, _name: &str, _idx: uint, f: |&mut Decoder<'a>| -> T) -> T {
      f(self)
    }

    fn read_option<T>(&mut self, f: |&mut Decoder<'a>, bool| -> T) -> T {
      match self._peek_byte() {
        0xc0 => f(self, false),
        _    => f(self, true)
      }
    }

    fn read_map<T>(&mut self, f: |&mut Decoder<'a>, uint| -> T) -> T {
      let len = self._read_map_len();
      f(self, len)
    }
    fn read_map_elt_key<T>(&mut self, _idx: uint, f: |&mut Decoder<'a>| -> T) -> T { f(self) }
    fn read_map_elt_val<T>(&mut self, _idx: uint, f: |&mut Decoder<'a>| -> T) -> T { f(self) }


    fn read_enum_struct_variant<T>(&mut self,
                                   names: &[&str],
                                   f: |&mut Decoder<'a>, uint| -> T)
                                   -> T {
      self.read_enum_variant(names, f)
    }


    fn read_enum_struct_variant_field<T>(&mut self,
                                         _name: &str,
                                         idx: uint,
                                         f: |&mut Decoder<'a>| -> T)
                                         -> T {
      self.read_enum_variant_arg(idx, f)
    }

    fn read_tuple<T>(&mut self, f: |&mut Decoder<'a>, uint| -> T) -> T {
      self.read_seq(f)
    }

    fn read_tuple_arg<T>(&mut self, idx: uint, f: |&mut Decoder<'a>| -> T) -> T {
      self.read_seq_elt(idx, f)
    }

    fn read_tuple_struct<T>(&mut self,
                            _name: &str,
                            f: |&mut Decoder<'a>, uint| -> T)
                            -> T {
      self.read_tuple(f)
    }

    fn read_tuple_struct_arg<T>(&mut self,
                                idx: uint,
                                f: |&mut Decoder<'a>| -> T)
                                -> T {
      self.read_tuple_arg(idx, f)
    }
}

impl<'a> serialize::Decodable<Decoder<'a>> for Value {
  fn decode(s: &mut Decoder<'a>) -> Value {
    s.decode_value()
  }
}


/// A structure for implementing serialization to Msgpack.
pub struct Encoder<'a> {
  priv wr: &'a mut io::Writer
}

impl<'a> Encoder<'a> {
  /// Creates a new Msgpack encoder whose output will be written to the writer
  /// specified.
  pub fn new(wr: &'a mut io::Writer) -> Encoder<'a> {
    Encoder { wr: wr }
  }

  /// Emits the most efficient representation of the given unsigned integer
  fn _emit_unsigned(&mut self, v: u64) {
    if v <= 127 {
      self.wr.write_u8(v as u8);
    }
    else if v <= 0xFF {
      self.wr.write_u8(0xcc);
      self.wr.write_u8(v as u8);
    }
    else if v <= 0xFFFF {
      self.wr.write_u8(0xcd);
      self.wr.write_be_u16(v as u16);
    }
    else if v <= 0xFFFF_FFFF {
      self.wr.write_u8(0xce);
      self.wr.write_be_u32(v as u32);
    }
    else {
      self.wr.write_u8(0xcf);
      self.wr.write_be_u64(v);
    }
  }

  /// Emits the most efficient representation of the given signed integer
  fn _emit_signed(&mut self, v: i64) {
    if v >= -(1i64<<7) && v < (1i64<<7) {
      let v = v as i8;
      if (v as u8) & 0xe0 != 0xe0 {
        self.wr.write_u8(0xd0);
      }
      self.wr.write_u8(v as u8);
    }
    else if v >= -(1i64<<15) && v < (1i64<<15) {
      let v = v as i16;
      self.wr.write_u8(0xd1);
      self.wr.write_be_i16(v);
    }
    else if v >= -(1i64<<31) && v < (1i64<<31) {
      let v = v as i32;
      self.wr.write_u8(0xd2);
      self.wr.write_be_i32(v);
    }
    else {
      self.wr.write_u8(0xd3);
      self.wr.write_be_i64(v);
    }
  }

  #[inline(always)]
  fn _emit_len(&mut self, len: uint, (op1, sz1): (u8, uint), (op2, sz2): (u8, uint), op3: u8, op4: u8) {
    if len < sz1 {
      self.wr.write_u8(op1);
    } else if len < sz2 {
      self.wr.write_u8(op2);
      self.wr.write_u8(len as u8);
    } else if len <= 0xFFFF {
      self.wr.write_u8(op3);
      self.wr.write_be_u16(len as u16);
    } else {
      assert!(len <= 0xFFFF_FFFF);
      self.wr.write_u8(op4);
      self.wr.write_be_u32(len as u32);
    }
  }

  fn _emit_str_len(&mut self, len: uint) {
    self._emit_len(len, (0xa0_u8 | (len & 31) as u8, 32),
                        (0xd9, 256),
                         0xda,
                         0xdb)
  }

  fn _emit_bin_len(&mut self, len: uint) {
    self._emit_len(len, (0x00, 0),
                        (0xc4, 256),
                         0xc5,
                         0xc6)
  }


  fn _emit_array_len(&mut self, len: uint) {
    self._emit_len(len, (0x90_u8 | (len & 15) as u8, 16),
                        (0x00, 0),
                         0xdc,
                         0xdd)
  }

  fn _emit_map_len(&mut self, len: uint) {
    self._emit_len(len, (0x80_u8 | (len & 15) as u8, 16),
                        (0x00, 0),
                         0xde,
                         0xdf)
  }
}

impl<'a> serialize::Encoder for Encoder<'a> {
  fn emit_nil(&mut self) { self.wr.write_u8(0xc0); }

  #[inline(always)]
  fn emit_uint(&mut self, v: uint) { self._emit_unsigned(v as u64); }
  #[inline(always)]
  fn emit_u64(&mut self, v: u64)   { self._emit_unsigned(v as u64); }
  #[inline(always)]
  fn emit_u32(&mut self, v: u32)   { self._emit_unsigned(v as u64); }
  #[inline(always)]
  fn emit_u16(&mut self, v: u16)   { self._emit_unsigned(v as u64); }
  #[inline(always)]
  fn emit_u8(&mut self, v: u8)     { self._emit_unsigned(v as u64); }

  #[inline(always)]
  fn emit_int(&mut self, v: int) { self._emit_signed(v as i64); }
  #[inline(always)]
  fn emit_i64(&mut self, v: i64) { self._emit_signed(v as i64); }
  #[inline(always)]
  fn emit_i32(&mut self, v: i32) { self._emit_signed(v as i64); }
  #[inline(always)]
  fn emit_i16(&mut self, v: i16) { self._emit_signed(v as i64); }
  #[inline(always)]
  fn emit_i8(&mut self,  v: i8)  { self._emit_signed(v as i64); }

  fn emit_f64(&mut self, v: f64) {
    self.wr.write_u8(0xcb);
    unsafe { self.wr.write_be_u64(cast::transmute(v)); }
  }

  fn emit_f32(&mut self, v: f32) {
    self.wr.write_u8(0xca);
    unsafe { self.wr.write_be_u32(cast::transmute(v)); }
  }

  fn emit_bool(&mut self, v: bool) {
    if v {
      self.wr.write_u8(0xc3);
    } else {
      self.wr.write_u8(0xc2);
    }
  }

  fn emit_char(&mut self, v: char) {
    self.emit_str(str::from_char(v));
  }

  fn emit_str(&mut self, v: &str) {
    self._emit_str_len(v.len());
    self.wr.write(v.as_bytes());
  }

  fn emit_enum(&mut self, _name: &str, _f: |&mut Encoder<'a>|) {
    fail!("Enum not supported");
  }

  fn emit_enum_variant(&mut self, _name: &str, _id: uint, _cnt: uint, _f: |&mut Encoder<'a>|) {
    fail!("Enum not supported");
  }

  fn emit_enum_variant_arg(&mut self, _idx: uint, _f: |&mut Encoder<'a>|) {
    fail!("Enum not supported");
  }

  fn emit_enum_struct_variant(&mut self, name: &str, id: uint, cnt: uint, f: |&mut Encoder<'a>|) {
    self.emit_enum_variant(name, id, cnt, f);
  }

  fn emit_enum_struct_variant_field(&mut self, _name: &str, idx: uint, f: |&mut Encoder<'a>|) {
    self.emit_enum_variant_arg(idx, f);
  }

  // TODO: Option, to enable different ways to write out structs
  //       For example, to emit structs as maps/vectors.
  // XXX: Correct to use _emit_map_len here?
  fn emit_struct(&mut self, _name: &str, len: uint, f: |&mut Encoder<'a>|) {
    self._emit_map_len(len);
    f(self);
  }

  fn emit_struct_field(&mut self, _name: &str, _idx: uint, f: |&mut Encoder<'a>|) {
    f(self);
  }

  fn emit_tuple(&mut self, len: uint, f: |&mut Encoder<'a>|) {
    self.emit_seq(len, f);
  }

  fn emit_tuple_arg(&mut self, idx: uint, f: |&mut Encoder<'a>|) {
    self.emit_seq_elt(idx, f);
  }

  fn emit_tuple_struct(&mut self,
                       _name: &str,
                       len: uint,
                       f: |&mut Encoder<'a>|) {
    self.emit_seq(len, f);
  }

  fn emit_tuple_struct_arg(&mut self, idx: uint, f: |&mut Encoder<'a>|) {
    self.emit_seq_elt(idx, f);
  }

  fn emit_option(&mut self, f: |&mut Encoder<'a>|) { f(self); }
  fn emit_option_none(&mut self) { self.emit_nil(); }
  fn emit_option_some(&mut self, f: |&mut Encoder<'a>|) { f(self); }

  fn emit_seq(&mut self, len: uint, f: |&mut Encoder<'a>|) {
    self._emit_array_len(len);
    f(self);
  }

  fn emit_seq_elt(&mut self, _idx: uint, f: |&mut Encoder<'a>|) {
    f(self);
  }

  fn emit_map(&mut self, len: uint, f: |&mut Encoder<'a>|) {
    self._emit_map_len(len);
    f(self);
  }

  fn emit_map_elt_key(&mut self, _idx: uint, f: |&mut Encoder<'a>|) {
    f(self);
  }

  fn emit_map_elt_val(&mut self, _idx: uint, f: |&mut Encoder<'a>|) {
    f(self);
  }
}

impl<'a> serialize::Encodable<Encoder<'a>> for Value {
  fn encode(&self, s: &mut Encoder<'a>) {
    match *self {
      Nil => (s as &mut serialize::Encoder).emit_nil(),
      Boolean(b) => (s as &mut serialize::Encoder).emit_bool(b),
      Integer(i) => (s as &mut serialize::Encoder).emit_i64(i),
      Unsigned(u) => (s as &mut serialize::Encoder).emit_u64(u),
      Float(f) => (s as &mut serialize::Encoder).emit_f32(f),
      Double(d) => (s as &mut serialize::Encoder).emit_f64(d),
      Array(ref ary) => {
        s._emit_array_len(ary.len());
        for elt in ary.iter() { elt.encode(s); }
      }
      Map(ref map) => {
        s._emit_map_len(map.len());
        for &(ref key, ref val) in map.iter() {
          key.encode(s);
          val.encode(s);
        }
      }
      String(ref str) => (s as &mut serialize::Encoder).emit_str(from_utf8(str.as_slice()).unwrap()),
      Binary(_) => fail!(),
      Extended(_, _) => fail!()
    }
  }
}


pub fn to_msgpack<'a, T: Encodable<Encoder<'a>>>(t: &T) -> ~[u8] {
  let mut wr = MemWriter::new();
  let mut encoder = Encoder::new(&mut wr);
  t.encode(&mut encoder);
  wr.unwrap()
}

pub fn from_msgpack<'a, T: Decodable<Decoder<'a>>>(bytes: ~[u8]) -> T {
  let mut rd = MemReader::new(bytes);
  let mut decoder = Decoder::new(&mut rd);
  Decodable::decode(&mut decoder)
}
