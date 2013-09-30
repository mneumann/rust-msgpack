extern mod std;
extern mod extra;

use std::path::Path;
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

pub fn decod(bytes: &[u8]) {
  let a: ~[~Blah] = do std::io::with_bytes_reader(bytes) |rd| {
    let mut parser = msgpack::Decoder::new(rd);
    Decodable::decode(&mut parser)
  };
  printfln!("%?", a);
}

fn main() {
  //debug!("main started");
  {
    let res = std::io::buffered_file_writer(&Path("test.msgpack"));
    if res.is_ok() {
      let mut c = HashMap::new();
      c.insert(1_u32, 2_u32);

      let mut enc = msgpack::Encoder { wr: res.unwrap() };
      let blah = Blah { f: 1, g: 2, i: ~"hallo", a: ~[], c: c.clone() };
      let blub = Blah { f: 5, g: 1, i: ~"nochwas", a: ~[1,2,3], c:  c.clone() };
      let b = ~[blah, blub];
      b.encode(&mut enc);
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
