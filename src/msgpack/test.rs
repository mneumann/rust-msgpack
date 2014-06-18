#![feature(macro_rules)]

extern crate msgpack = "msgpack#0.1";
extern crate serialize;

use std::collections::hashmap::HashMap;
use msgpack::{from_msgpack,to_msgpack};
use serialize::Encodable;

macro_rules! assert_msgpack_circular(
    ($inp:expr) => (
        assert_eq!($inp, from_msgpack(to_msgpack(&$inp).ok().unwrap()).ok().unwrap())
    );
)


#[test]
fn test_circular_str() {
  assert_msgpack_circular!("".to_str());
  assert_msgpack_circular!("a".to_str());
  assert_msgpack_circular!("abcdef".to_str());
}

#[test]
fn test_circular_int() {
  assert_msgpack_circular!(123);
  assert_msgpack_circular!(-123);
}

#[test]
fn test_circular_float() {
  assert_msgpack_circular!(-1243.111);
}

#[test]
fn test_circular_bool() {
  assert_msgpack_circular!(true);
  assert_msgpack_circular!(false);
}

#[test]
fn test_circular_list() {
  assert_msgpack_circular!(vec![1,2,3]);
}

#[test]
fn test_circular_map() {
  let mut v = HashMap::new();
  v.insert(1, 2);
  v.insert(3, 4);
  assert_msgpack_circular!(v);
}

#[test]
fn test_circular_option() {
  let v: Option<int> = Some(1);
  assert_msgpack_circular!(v);

  let v: Option<int> = None;
  assert_msgpack_circular!(v);
}

#[test]
fn test_circular_char() {
  assert_msgpack_circular!('a');
}

#[deriving(Encodable,Decodable,PartialEq,Show)]
struct S {
  f: u8,
  g: u16,
  i: String,
  a: Vec<u32>,
  c: HashMap<u32, u32>
}

#[test]
fn test_circular_struct() {
  let mut c = HashMap::new();
  c.insert(1_u32, 2_u32);
  c.insert(2_u32, 3_u32);

  let s1 = S { f: 1, g: 2, i: "foo".to_str(), a: vec![], c: c.clone() };
  let s2 = S { f: 5, g: 1, i: "bar".to_str(), a: vec![1,2,3], c: c.clone() };
  let s = vec![s1, s2];

  assert_msgpack_circular!(s);
}
