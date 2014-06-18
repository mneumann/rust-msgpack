#![feature(phase)]

#[phase(syntax, link)] extern crate log;
extern crate msgpack = "msgpack#0.1";

use std::io::{File};
use std::os::args;

fn main() {
  let contents = File::open(&Path::new(args().get(1).clone())).read_to_end().ok().unwrap();
  debug!("{}", contents);

  let a: msgpack::Value = msgpack::from_msgpack(contents).ok().unwrap();
  debug!("{}", a);
}
