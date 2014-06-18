extern crate msgpack = "msgpack#0.1";

fn main() {
  let arr = vec!["str1".to_str(), "str2".to_str()];
  let str = msgpack::to_msgpack(&arr).ok().unwrap();
  println!("Encoded: {}", str);

  let dec: Vec<String> = msgpack::from_msgpack(str).ok().unwrap();
  println!("Decoded: {}", dec);
}
