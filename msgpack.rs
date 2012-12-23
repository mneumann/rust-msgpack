use io::{WriterUtil,ReaderUtil};
use path::Path;
use cast::reinterpret_cast;

struct Serializer {
  priv wr: io::Writer
}

impl Serializer {
  fn emit_u8(v: u8) {
    if (v & 128) != 0 {
      self.wr.write_u8(0xcc);
    }
    self.wr.write_u8(v);
  }

  fn emit_u16(v: u16) {
    self.wr.write_u8(0xcd);
    self.wr.write_be_u16(v);
  }

  fn emit_u32(v: u32) {
    self.wr.write_u8(0xce);
    self.wr.write_be_u32(v);
  }

  fn emit_u64(v: u64) {
    self.wr.write_u8(0xcf);
    self.wr.write_be_u64(v);
  }

  fn emit_uint(v: u64) {
    if      (v & 0xFFFFFFFFFFFFFF00) == 0 { self.emit_u8(v as u8);  }
    else if (v & 0xFFFFFFFFFFFF0000) == 0 { self.emit_u16(v as u16); }
    else if (v & 0xFFFFFFFF00000000) == 0 { self.emit_u32(v as u32); }
    else                                  { self.emit_u64(v); }
  }

  fn emit_i8(v: i8) {
    let v: u8 = v as u8;
    if (v & 0xe0) != 0xe0 {
      self.wr.write_u8(0xd0);
    }
    self.wr.write_u8(v);
  }

  fn emit_i16(v: i16) {
    self.wr.write_u8(0xd1);
    self.wr.write_be_i16(v);
  }

  fn emit_i32(v: i32) {
    self.wr.write_u8(0xd2);
    self.wr.write_be_i32(v);
  }

  fn emit_i64(v: i64) {
    self.wr.write_u8(0xd3);
    self.wr.write_be_i64(v);
  }

  fn emit_int(v: i64) {
    if      v >= -(1i64<<7)  && v <= (1i64<<7)-1    { self.emit_i8(v as i8); }
    else if v >= -(1i64<<15) && v <= (1i64<<15)-1   { self.emit_i16(v as i16); }
    else if v >= -(1i64<<31) && v <= (1i64<<31)-1   { self.emit_i32(v as i32); }
    else /* v >= -(1i64<<63) && v <= (1i64<<63)-1) */ { self.emit_i64(v); }
  }

  fn emit_nil() {
    self.wr.write_u8(0xc0);
  }

  fn emit_bool(v: bool) {
    if v {
      self.wr.write_u8(0xc3);
    } else {
      self.wr.write_u8(0xc2);
    }
  }

  fn emit_f32(v: f32) {
    self.wr.write_u8(0xca);
    unsafe { self.wr.write_be_u32(reinterpret_cast(&v)); }
  }

  fn emit_f64(v: f64) {
    self.wr.write_u8(0xcb);
    unsafe { self.wr.write_be_u64(reinterpret_cast(&v)); }
  }

  priv fn emit_raw_len(len: uint) {
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

  fn emit_raw(v: &[const u8]) {
    self.emit_raw_len(vec::len(v));
    self.wr.write(v);    
  }

  fn emit_str(v: &str) {
    self.emit_raw_len(str::len(v));
    self.wr.write_str(v);    
  }

  fn emit_array_len(len: uint) {
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
    let mut values = vec::with_capacity(len);
    for len.times {
      values.push(self.parse_value());
    }
    Array(values)
  }

  priv fn parse_map(len: uint) -> Value {
    let mut values = vec::with_capacity(len);
    for len.times {
      let k = self.parse_value();
      let v = self.parse_value();
      values.push((k, v));
    }
    Map(values)
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
    let c = c as u8;

    if c <= 0x7f {
      Uint(c as u64)
    } else if c <= 0x8f {
      self.parse_map(c as uint & 0x0F)
    } else if c <= 0x9f {
      self.parse_array(c as uint & 0x0F)
    } else if c <= 0xbf {
      self.parse_raw(c as uint & 0x1F)
    } else if c >= 0xe0 {
      Int((c as i8) as i64)
    } else {
      match c {
        0xc0 => Nil,
        0xc1 => fail ~"Reserved",
        0xc2 => Bool(false),
        0xc3 => Bool(true),
        0xc4 .. 0xc9 => fail ~"Reserved",
        0xd4 .. 0xd9 => fail ~"Reserved",
        0xca => Float(self.conv_float(self.rd.read_be_u32())),
        0xcb => Double(self.conv_double(self.rd.read_be_u64())),
        0xcc => Uint(self.rd.read_u8() as u64),
        0xcd => Uint(self.rd.read_be_u16() as u64),
        0xce => Uint(self.rd.read_be_u32() as u64),
        0xcf => Uint(self.rd.read_be_u64()),
        0xd0 => Int(self.rd.read_i8() as i64),
        0xd1 => Int(self.rd.read_be_i16() as i64),
        0xd2 => Int(self.rd.read_be_i32() as i64),
        0xd3 => Int(self.rd.read_be_i64()),
        0xda => self.parse_raw(self.rd.read_be_u16() as uint),
        0xdb => self.parse_raw(self.rd.read_be_u32() as uint),
        0xdc => self.parse_array(self.rd.read_be_u16() as uint),
        0xdd => self.parse_array(self.rd.read_be_u32() as uint),
        0xde => self.parse_map(self.rd.read_be_u16() as uint),
        0xdf => self.parse_map(self.rd.read_be_u32() as uint),
        _ => fail ~"Invalid"
      }
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

fn main() {
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

  //result::iter(&io::read_whole_file(&Path("/tmp/matching.msgpack")), |buf| {doit(*buf);});
  let bytes = move io::read_whole_file(&Path("/tmp/matching.msgpack")).get();
  doit(bytes);

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
