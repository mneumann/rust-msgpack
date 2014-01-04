use std::{io, cast, str};
use extra::serialize;
use super::utils;

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
  fn _emit_len(&mut self, len: uint, sz1: uint, op1: u8, op2: u8, op3: u8) {
    if len <= sz1 {
      self.wr.write_u8(op1 | (len as u8));
    } else if len <= 0xFFFF {
      self.wr.write_u8(op2);
      self.wr.write_be_u16(len as u16);
    } else {
      assert!(len <= 0xFFFF_FFFF);
      self.wr.write_u8(op3);
      self.wr.write_be_u32(len as u32);
    }
  }

  fn _emit_raw_len(&mut self, len: uint) {
    self._emit_len(len, 31, 0xa0, 0xda, 0xdb)
  }

  fn _emit_vec_len(&mut self, len: uint) {
    self._emit_len(len, 15, 0x90, 0xdc, 0xdd)
  }

  fn _emit_map_len(&mut self, len: uint) {
    self._emit_len(len, 15, 0x80, 0xde, 0xdf)
  }

  #[inline(always)]
  fn _emit_raw(&mut self, v: &[u8]) {
    self._emit_raw_len(v.len());
    self.wr.write(v);
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
    self._emit_raw(v.as_bytes());
  }

  fn emit_enum(&mut self, _name: &str, _f: |&mut Encoder<'a>|) {
    fail!(~"Enum not supported");
  }

  fn emit_enum_variant(&mut self, _name: &str, _id: uint, _cnt: uint, _f: |&mut Encoder<'a>|) {
    fail!(~"Enum not supported");
  }

  fn emit_enum_variant_arg(&mut self, _idx: uint, _f: |&mut Encoder<'a>|) {
    fail!(~"Enum not supported");
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
    self._emit_vec_len(len);
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
