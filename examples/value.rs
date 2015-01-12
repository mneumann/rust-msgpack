#![allow(unstable)]

#[macro_use] extern crate log;
extern crate msgpack;

use std::io::File;
use std::os::args;

fn main() {
  let contents = File::open(&Path::new(args()[1].clone())).read_to_end().ok().unwrap();
  debug!("{:?}", contents);

  let a: msgpack::Value = msgpack::from_msgpack(contents.as_slice()).ok().unwrap();
  debug!("{:?}", a);
}
