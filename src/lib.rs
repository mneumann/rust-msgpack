#[desc = "msgpack.org implementation for Rust"]
#[license = "MIT"]
#[crate_id = "github.com/mneumann/rust-msgpack#0.1"]
#[crate_type = "lib"];

extern mod extra;

use std::io::mem::{MemReader,MemWriter,with_mem_writer};
use encoder::Encoder;
use decoder::Decoder;
use extra::serialize::{Encodable,Decodable};

mod utils;
mod encoder;
mod decoder;
mod parser;
#[cfg(test)] mod tests;

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
