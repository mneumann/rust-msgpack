extern crate msgpack;

use std::old_io::File;
use std::os::args;

fn main() {
  let contents = File::open(&Path::new(args()[1].clone())).read_to_end().ok().unwrap();
  println!("{:?}", contents);

/* todo
  let a: msgpack::Value = msgpack::from_msgpack(contents.as_slice()).ok().unwrap();
  println!("{:?}", a);
*/
}
