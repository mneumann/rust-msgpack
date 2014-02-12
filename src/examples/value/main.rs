extern mod msgpack = "msgpack#0.1";
use std::io::{File};
use std::io::{IoResult, IoError};
use std::os::args;

fn main() {
  let contents = File::open(&Path::new(args()[1])).read_to_end();
  println!("{:?}", contents);

  // let a = match(contents) {
    // _ 
  //   IoResult(r)    => msgpack::from_msgpack(contents),
  //   IoError(msg) => fail!(msg)
  // };
  // // let a: msgpack::Value = msgpack::from_msgpack(contents).unwrap();
  // println!("{:?}", a);
}
