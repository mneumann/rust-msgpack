extern crate msgpack;

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let mut contents = Vec::new();
    File::open(&Path::new(filename)).unwrap().read_to_end(&mut contents).unwrap();
    println!("{:?}", contents);

    /* todo
   let a: msgpack::Value = msgpack::from_msgpack(contents.as_slice()).ok().unwrap();
   println!("{:?}", a);
   */
}
