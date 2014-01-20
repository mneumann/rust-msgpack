use std::io;
use std::vec;
use super::utils;

pub enum Value {
  Nil,
  Boolean(bool),
  Array(~[Value]),
  Map(~[(Value, Value)]),
  Integer(i64),
  Unsigned(u64),
  Float(f32),
  Double(f64),
  String(~[u8]),
  Binary(~[u8]),
  Extended(i8, ~[u8])
}

/// A structure to decode Msgpack from a reader into a Value.
pub struct Decoder<'a> {
  priv rd: &'a mut io::Reader,
}

impl<'a> Decoder<'a> {

  /// Creates a new Msgpack decoder from the specified reader.
  pub fn new(rd: &'a mut io::Reader) -> Decoder<'a> {
    Decoder { rd: rd }
  }

  fn decode_array(&mut self, len: uint) -> Value {
    Array(vec::from_fn(len, |_| { self.decode() }))
  }

  fn decode_map(&mut self, len: uint) -> Value {
    Map(vec::from_fn(len, |_| { (self.decode(), self.decode()) }))
  }

  fn decode_ext(&mut self, len: uint) -> Value {
    let typ = self.rd.read_i8();
    if typ < 0 { fail!("Reserved type") }
    let data = self.rd.read_bytes(len);
    Extended(typ, data)
  }

  fn _read_raw(&mut self, len: uint) -> ~[u8] {
    self.rd.read_bytes(len)
  }

  pub fn decode(&mut self) -> Value {
    let c: u8 = self.rd.read_byte().unwrap();
    match c {
      0xc0         => Nil,

      0xc1         => fail!("Reserved"),

      0xc2         => Boolean(false),
      0xc3         => Boolean(true),

      0x00 .. 0x7f => Unsigned(c as u64),
      0xcc         => Unsigned(self.rd.read_u8() as u64),
      0xcd         => Unsigned(self.rd.read_be_u16() as u64),
      0xce         => Unsigned(self.rd.read_be_u32() as u64),
      0xcf         => Unsigned(self.rd.read_be_u64()),

      0xd0         => Integer(self.rd.read_i8() as i64),
      0xd1         => Integer(self.rd.read_be_i16() as i64),
      0xd2         => Integer(self.rd.read_be_i32() as i64),
      0xd3         => Integer(self.rd.read_be_i64()),
      0xe0 .. 0xff => Integer((c as i8) as i64),

      0xca         => Float(utils::read_float(self.rd)),
      0xcb         => Double(utils::read_double(self.rd)),

      0xa0 .. 0xbf => String(self._read_raw((c as uint) & 0x1F)),
      0xd9         => { let b = self.rd.read_u8() as uint; String(self._read_raw(b)) },
      0xda         => { let b = self.rd.read_be_u16() as uint; String(self._read_raw(b)) },
      0xdb         => { let b = self.rd.read_be_u32() as uint; String(self._read_raw(b)) },

      0xc4         => { let b = self.rd.read_u8() as uint; Binary(self._read_raw(b)) },
      0xc5         => { let b = self.rd.read_be_u16() as uint; Binary(self._read_raw(b)) },
      0xc6         => { let b = self.rd.read_be_u32() as uint; Binary(self._read_raw(b)) },

      0x90 .. 0x9f => self.decode_array((c as uint) & 0x0F),
      0xdc         => { let b = self.rd.read_be_u16() as uint; self.decode_array(b) },
      0xdd         => { let b = self.rd.read_be_u32() as uint; self.decode_array(b) },
     
      0x80 .. 0x8f => self.decode_map((c as uint) & 0x0F),
      0xde         => { let b = self.rd.read_be_u16() as uint; self.decode_map(b) },
      0xdf         => { let b = self.rd.read_be_u32() as uint; self.decode_map(b) },

      0xd4         => self.decode_ext(1),
      0xd5         => self.decode_ext(2),
      0xd6         => self.decode_ext(4),
      0xd7         => self.decode_ext(8),
      0xd8         => self.decode_ext(16),
      0xc7         => { let b = self.rd.read_u8() as uint; self.decode_ext(b) },
      0xc8         => { let b = self.rd.read_be_u16() as uint; self.decode_ext(b) },
      0xc9         => { let b = self.rd.read_be_u32() as uint; self.decode_ext(b) },

      // XXX: This is only here to satify Rust's pattern checker.
      _            => fail!("Fatal")
    }
  }
}

pub fn from_msgpack(bytes: ~[u8]) -> Value {
  use std::io::MemReader;
  let mut rd = MemReader::new(bytes);
  let mut decoder = Decoder::new(&mut rd);
  decoder.decode()
}
