#[crate_id = "msgpack#0.1"];
#[comment = "msgpack.org implementation for Rust"];
#[license = "MIT/ASL2"];
#[crate_type = "lib"];

extern mod extra;

pub use msgpack::{encoder,decoder,value,to_msgpack,from_msgpack};

pub mod msgpack;
