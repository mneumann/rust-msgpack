extern crate msgpack = "msgpack#0.1";
extern crate serialize= "serialize#0.10-pre";
extern crate collections;

use collections::hashmap::HashMap;
use msgpack::{from_msgpack,to_msgpack};
use serialize::Encodable;

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

#[deriving(Encodable,Decodable,Eq)]
struct S {
  f: u8,
  g: u16,
  i: ~str,
  a: ~[u32],
  c: HashMap<u32, u32>
}

#[test]
fn test_circular_struct() {
  let mut c = HashMap::new();
  c.insert(1_u32, 2_u32);
  c.insert(2_u32, 3_u32);

  let s1 = S { f: 1, g: 2, i: ~"foo", a: ~[], c: c.clone() };
  let s2 = S { f: 5, g: 1, i: ~"bar", a: ~[1,2,3], c: c.clone() };
  let s = ~[s1, s2];

  assert_eq!(s, from_msgpack(to_msgpack(&s)))
}
