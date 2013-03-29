extern mod std;
extern mod msgpack;

use core::path::Path;

use std::*;
use std::serialize::*;

struct MapItem<K, V> {
  key: K,
  val: V
}

impl<K,V> MapItem<K,V> {
  fn mk(k: K, v: V) -> MapItem<K,V> {
    MapItem {key: k, val: v}
  }
}

impl<D: serialize::Decoder,
     K: serialize::Decodable<D>,
     V: serialize::Decodable<D>> serialize::Decodable<D> for MapItem<K,V> {
  #[inline(always)]
  fn decode(d: &D) -> MapItem<K,V> {
    MapItem {key: Decodable::decode(d), val: Decodable::decode(d)}
  }
}

impl<S: serialize::Encoder,
     K: serialize::Encodable<S>,
     V: serialize::Encodable<S>> serialize::Encodable<S> for MapItem<K,V> {
  fn encode(&self, s: &S) {
    self.key.encode(s);
    self.val.encode(s)
  }
}

#[auto_encode]
#[auto_decode]
struct Blah {
  f: u8,
  g: u16,
  i: ~str,
  a: ~[u32],
  c: ~[MapItem<u32, u32>]
}

fn decod(bytes: &[u8]) {
  let a: ~[~Blah] = do io::with_bytes_reader(bytes) |rd| {
    let parser = msgpack::Decoder::new(rd);
    serialize::Decodable::decode(&parser)
  };
  io::println(fmt!("%?", a));
}

fn main() {
  {
    let res = io::buffered_file_writer(&Path("test.msgpack"));
    if res.is_ok() {
      let enc = msgpack::Encoder { wr: res.unwrap() };
      let blah = Blah { f: 1, g: 2, i: ~"hallo", a: ~[], c: ~[MapItem::mk(1,2)] }; 
      let blub = Blah { f: 5, g: 1, i: ~"nochwas", a: ~[1,2,3], c: ~[MapItem::mk(1,2)] }; 
      let b = ~[blah, blub];
      b.encode(&enc);
      //5.encode(&enc);
      io::println("OK");
    }
  }

  let bytes: ~[u8] = result::unwrap(io::read_whole_file(&Path("test.msgpack")));
  let b = time::precise_time_ns();
  decod(bytes);
  let total = time::precise_time_ns() - b;
  io::println(fmt!("%?", total / 1000000));
}
