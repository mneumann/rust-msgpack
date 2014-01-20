pub enum Value {
  Nil,
  Boolean(bool),
  Integer(i64),
  Unsigned(u64),
  Float(f32),
  Double(f64),
  Array(~[Value]),
  Map(~[(Value, Value)]),
  String(~[u8]),
  Binary(~[u8]),
  Extended(i8, ~[u8])
}
