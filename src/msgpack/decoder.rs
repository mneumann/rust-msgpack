use std::{io, str, vec, cast};
use extra::serialize;
use super::value::{Value,Nil,Boolean,Integer,Unsigned,Float,Double,Array,Map,String,Binary,Extended};

#[inline(always)]
fn read_float(rd: &mut io::Reader) -> f32 {
  let v = rd.read_be_u32();
  unsafe { cast::transmute(v) }
}

#[inline(always)]
fn read_double(rd: &mut io::Reader) -> f64 {
  let v = rd.read_be_u64();
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
        self.next_byte = self.rd.read_byte();
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
        match self.rd.read_byte() { 
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
      0xcc         => self.rd.read_u8() as u64,
      0xcd         => self.rd.read_be_u16() as u64,
      0xce         => self.rd.read_be_u32() as u64,
      0xcf         => self.rd.read_be_u64(),
      _            => fail!("No unsigned integer")
    }
  }

  fn _read_signed(&mut self) -> i64 {
    let c = self._read_byte();
    match c {
      0xd0         => self.rd.read_i8() as i64,
      0xd1         => self.rd.read_be_i16() as i64,
      0xd2         => self.rd.read_be_i32() as i64,
      0xd3         => self.rd.read_be_i64(),
      0xe0 .. 0xff => (c as i8) as i64,
      _            => fail!("No signed integer")
    }
  }

  fn _read_raw(&mut self, len: uint) -> ~[u8] {
    self.rd.read_bytes(len)
  }

  fn _read_str(&mut self, len: uint) -> ~str {
    str::from_utf8_owned(self.rd.read_bytes(len))
  }

  fn _read_vec_len(&mut self) -> uint {
    let c = self._read_byte();

    match c {
      0x90 .. 0x9f => (c as uint) & 0x0F,
      0xdc         => self.rd.read_be_u16() as uint,
      0xdd         => self.rd.read_be_u32() as uint,
      _            => fail!("Invalid byte code in _read_vec_len")
    }
  }

  fn _read_map_len(&mut self) -> uint {
    let c = self._read_byte();
    match c {
      0x80 .. 0x8f => (c as uint) & 0x0F,
      0xde         => self.rd.read_be_u16() as uint,
      0xdf         => self.rd.read_be_u32() as uint,
      _            => fail!("Invalid byte code in _read_map_len")
    }
  }

  fn decode_array(&mut self, len: uint) -> Value {
    Array(vec::from_fn(len, |_| { self.decode_value() }))
  }

  fn decode_map(&mut self, len: uint) -> Value {
    Map(vec::from_fn(len, |_| { (self.decode_value(), self.decode_value()) }))
  }

  fn decode_ext(&mut self, len: uint) -> Value {
    let typ = self.rd.read_i8();
    if typ < 0 { fail!("Reserved type") }
    let data = self.rd.read_bytes(len);
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
      0xcc         => Unsigned(self.rd.read_u8() as u64),
      0xcd         => Unsigned(self.rd.read_be_u16() as u64),
      0xce         => Unsigned(self.rd.read_be_u32() as u64),
      0xcf         => Unsigned(self.rd.read_be_u64()),

      0xd0         => Integer(self.rd.read_i8() as i64),
      0xd1         => Integer(self.rd.read_be_i16() as i64),
      0xd2         => Integer(self.rd.read_be_i32() as i64),
      0xd3         => Integer(self.rd.read_be_i64()),
      0xe0 .. 0xff => Integer((c as i8) as i64),

      0xca         => Float(read_float(self.rd)),
      0xcb         => Double(read_double(self.rd)),

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
