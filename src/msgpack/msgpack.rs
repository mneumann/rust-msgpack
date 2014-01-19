use std::io::{MemReader,MemWriter};
use msgpack::encoder::Encoder;
use msgpack::decoder::Decoder;
use extra::serialize::{Encodable,Decodable};

mod utils;
pub mod encoder;
pub mod decoder;
pub mod value;

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
