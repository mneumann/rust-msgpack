extern mod extra;

use msgpack::{from_msgpack, to_msgpack};
use std::hashmap::HashMap;

mod msgpack;

#[test]
fn test_circular_str() {
  assert_eq!(~"", from_msgpack(to_msgpack(&~"")));
  assert_eq!(~"a", from_msgpack(to_msgpack(&~"a")));
  assert_eq!(~"abcdef", from_msgpack(to_msgpack(&~"abcdef")));
}

#[test]
fn test_circular_int() {
  assert_eq!(123, from_msgpack(to_msgpack(&123)));
  assert_eq!(-123, from_msgpack(to_msgpack(&-123)));
}

#[test]
fn test_circular_float() {
  let v = -1243.111;
  assert_eq!(v, from_msgpack(to_msgpack(&v)));
}

#[test]
fn test_circular_bool() {
  assert_eq!(true, from_msgpack(to_msgpack(&true)));
  assert_eq!(false, from_msgpack(to_msgpack(&false)));
}

#[test]
fn test_circular_list() {
  let v = ~[1, 2, 3];
  assert_eq!(v.clone(), from_msgpack(to_msgpack(&v)));
}

#[test]
fn test_circular_map() {
  let mut v = HashMap::new();
  v.insert(1, 2);
  v.insert(3, 4);
  assert_eq!(v.clone(), from_msgpack(to_msgpack(&v)));
}

#[test]
fn test_circular_option() {
  let mut v = Some(1);
  let w : Option<int> = from_msgpack(to_msgpack(&v));

  assert_eq!(w, v);

  v = None;
  assert_eq!(v.clone(), from_msgpack(to_msgpack(&v)));
}

#[test]
fn test_circular_char() {
  let a: char = 'a';
  assert_eq!(a, from_msgpack(to_msgpack(&a)))
}
