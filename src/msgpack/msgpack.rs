use std::io::mem::{MemReader,MemWriter,with_mem_writer};
use msgpack::encoder::Encoder;
use msgpack::decoder::Decoder;
use msgpack::parser::{Value,Parser};
use extra::serialize::{Encodable,Decodable};

mod utils;
mod encoder;
mod decoder;
mod parser;

pub fn to_msgpack<'a, T: Encodable<Encoder<'a>>>(t: &T) -> ~[u8] {
  with_mem_writer(|wr: &mut MemWriter| {
    let mut encoder = Encoder::new(wr);
    t.encode(&mut encoder);
  })
}

pub fn from_msgpack<'a, T: Decodable<Decoder<'a>>>(bytes: ~[u8]) -> T {
  let mut rd = MemReader::new(bytes);
  let mut decoder = Decoder::new(&mut rd);
  Decodable::decode(&mut decoder)
}

pub fn parse_msgpack(bytes: ~[u8]) -> parser::Value {
  let mut rd = MemReader::new(bytes);
  let mut parser = Parser::new(&mut rd);
  parser.parse()
}
