extern mod std;
extern mod extra;

use std::path::Path;
use std::hashmap;
use std::hashmap::HashMap;

use extra::time;
use extra::serialize::{Decoder, Decodable, Encoder, Encodable};

mod msgpack;


#[deriving(Encodable,Decodable)]

struct Blah {
  f: u8,
  g: u16,
  i: ~str,
  a: ~[u32],
  c: HashMap<u32, u32>
}

fn decod(bytes: &[u8]) {
  let a: ~[~Blah] = do std::io::with_bytes_reader(bytes) |rd| {
    let parser = msgpack::Decoder::new(rd);
    Decodable::decode(&parser)
  };
  printfln!("%?", a);
}

fn main() {
  {
    let res = std::io::buffered_file_writer(&Path("test.msgpack"));
    if res.is_ok() {
      let mut c = HashMap::new();
      c.insert(1_u32, 2_u32);

      let enc = msgpack::Encoder { wr: res.unwrap() };
      let blah = Blah { f: 1, g: 2, i: ~"hallo", a: ~[], c: c };
      let blub = Blah { f: 5, g: 1, i: ~"nochwas", a: ~[1,2,3], c:  c };
      let b = ~[blah, blub];
      b.encode(&enc);
      //5.encode(&enc);
      println!("OK");
    }
  }

  let bytes: ~[u8] = std::io::read_whole_file(&Path("test.msgpack")).unwrap();
  let b = time::precise_time_ns();
  decod(bytes);
  let total = time::precise_time_ns() - b;
  printfln!("%?", total / 1000000);
}
