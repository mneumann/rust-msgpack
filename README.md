# rust-msgpack [![Build Status][travis-image]][travis-link]

[travis-image]: https://travis-ci.org/mneumann/rust-msgpack.png?branch=master
[travis-link]: https://travis-ci.org/mneumann/rust-msgpack

[Msgpack][msgpack-home] implementation for [Rust][rust-home] language.

[msgpack-home]: http://www.msgpack.org
[rust-home]: http://www.rust-lang.org

## Quickstart

```rust
extern mod msgpack = "msgpack#0.1";

fn main() {
  let arr = ~[~"str1", ~"str2"];
  let str = msgpack::to_msgpack(&arr);
  println!("Encoded: {:?}", str);

  let dec: ~[~str] = msgpack::from_msgpack(str);
  println!("Decoded: {:?}", dec);
}
```

To enable your own data structures to be serialized from and to msgpack, simply
use a <code>#[deriving(Encodable,Decodable)]</code> annotation as shown in the
following example:

```rust
#[deriving(Encodable,Decodable)]
struct MyStruct {
  a: ~[u32],
  s: ~str
}
```

## License

This code licensed under the same terms as Rust itself: dual MIT/Apache2 license options.
