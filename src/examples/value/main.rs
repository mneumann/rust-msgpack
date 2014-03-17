#[feature(phase)];

#[phase(syntax, link)] extern crate log;
extern crate msgpack = "msgpack#0.1";

use std::io::{File};
use std::os::args;

fn main() {
  let contents = File::open(&Path::new(args()[1])).read_to_end().unwrap();
  debug!("{:?}", contents);

  let a: msgpack::Value = msgpack::from_msgpack(contents);
  debug!("{:?}", a);
}
