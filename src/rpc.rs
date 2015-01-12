use serialize;
use serialize::{Encodable,Decodable};

use super::{Value,Encoder,Decoder};

pub enum RpcMessage {
  RpcRequest      {msgid: u32, method: String, params: Vec<Value>}, // 0
  RpcResponse     {msgid: u32, error: Value, result: Value}, // 1
  RpcNotification {method: String, params: Vec<Value>} // 2
}

impl serialize::Encodable for RpcMessage {
  fn encode<E: serialize::Encoder>(&self, s: &mut E) -> Result<(), E::Error> {
    match *self {
      RpcMessage::RpcRequest {msgid, ref method, ref params} => {
        (0us, msgid, method, params).encode(s)
      }
      RpcMessage::RpcResponse {msgid, ref error, ref result} => {
        (1us, msgid, error, result).encode(s)
      }
      RpcMessage::RpcNotification {ref method, ref params} => {
        (2us, method, params).encode(s)
      }
    }
  }
}

impl serialize::Decodable for RpcMessage {
  fn decode<D: serialize::Decoder>(s: &mut D) -> Result<RpcMessage, D::Error> {
    let len: usize = try!(Decodable::decode(s));
    let ty: usize = try!(Decodable::decode(s));

    match ty {
      0 => {
        if len != 4 { return Err(s.error("Invalid msgpack-rpc message array length")) }
        let msgid = try!(Decodable::decode(s));
        let method = try!(Decodable::decode(s));
        let params = try!(Decodable::decode(s));
        Ok(RpcMessage::RpcRequest {msgid: msgid, method: method, params: params})
      }
      1 => {
        if len != 4 { return Err(s.error("Invalid msgpack-rpc message array length")) }
        let msgid = try!(Decodable::decode(s));
        let error = try!(Decodable::decode(s));
        let result = try!(Decodable::decode(s));
        Ok(RpcMessage::RpcResponse {msgid: msgid, error: error, result: result})
      }
      2 => {
        if len != 3 { return Err(s.error("Invalid msgpack-rpc message array length")) }
        let method = try!(Decodable::decode(s));
        let params = try!(Decodable::decode(s));
        Ok(RpcMessage::RpcNotification {method: method, params: params})
      }
      _ => {
        Err(s.error("Invalid msgpack-rpc message type"))
      }
    }

  }
}
