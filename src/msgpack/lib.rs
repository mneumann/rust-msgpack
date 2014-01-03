#[crate_id = "msgpack#0.1"];
#[comment = "msgpack.org implementation for Rust"];
#[license = "MIT/ASL2"];
#[crate_type = "lib"];
#[feature(globs)];

extern mod extra;

pub use msgpack::*;

pub mod msgpack;
