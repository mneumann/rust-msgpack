extern mod msgpack = "msgpack#0.1";

fn main() {
  let a = msgpack::to_msgpack(&~"");
  println!("{:?}", a);
}
