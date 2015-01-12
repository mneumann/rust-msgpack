#![allow(unstable)]

extern crate msgpack;

fn main() {
  let arr = vec!["str1".to_string(), "str2".to_string()];
  let str = msgpack::Encoder::to_msgpack(&arr).unwrap();
  println!("Encoded: {:?}", str);

  let dec: Vec<String> = msgpack::from_msgpack(str.as_slice()).unwrap();
  println!("Decoded: {:?}", dec);
}
