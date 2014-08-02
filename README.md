# rust-msgpack [![Build Status][travis-image]][travis-link]

[travis-image]: https://travis-ci.org/mneumann/rust-msgpack.png?branch=master
[travis-link]: https://travis-ci.org/mneumann/rust-msgpack

[Msgpack][msgpack-home] implementation for [Rust][rust-home] language.

[msgpack-home]: http://www.msgpack.org
[rust-home]: http://www.rust-lang.org

## Installation

Simply include the rust-msgpack in your Cargo dependencies.

```toml
[dependencies.msgpack]

git = "git@github.com:mneumann/rust-msgpack.git"
```

## Quickstart

```rust
extern crate msgpack;

fn main() {
  let arr = vec!["str1".to_string(), "str2".to_string()];
  let str = msgpack::Encoder::to_msgpack(&arr).ok().unwrap();
  println!("Encoded: {}", str);

  let dec: Vec<String> = msgpack::from_msgpack(str).ok().unwrap();
  println!("Decoded: {}", dec);
}
```

To enable your own data structures to be automatically serialized from and to
msgpack, derive from <code>Encodable</code> and <code>Decodable</code> as shown
in the following example:

```rust
extern crate serialize;

#[deriving(Encodable,Decodable)]
struct MyStruct {
  a: Vec<u32>,
  s: String
}
```

## Testing

```
cargo test
```

## License

This code licensed under the same terms as Rust itself: dual MIT/Apache2 license options.
