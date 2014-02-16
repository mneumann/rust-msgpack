extern crate msgpack = "msgpack#0.1";

fn main() {
  let arr = ~[~"str1", ~"str2"];
  let str = msgpack::to_msgpack(&arr);
  println!("Encoded: {:?}", str);

  let dec: ~[~str] = msgpack::from_msgpack(str);
  println!("Decoded: {:?}", dec);
}
