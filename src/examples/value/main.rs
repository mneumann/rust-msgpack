extern mod msgpack = "msgpack#0.1";
use std::io::File;
use std::os::args;

fn main() {
  let contents = File::open(&Path::new(args()[1])).read_to_end();
  println!("{:?}", contents);
  let a: msgpack::value::Value = msgpack::from_msgpack(contents);
  println!("{:?}", a);
}
