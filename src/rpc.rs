use super::{Value, _invalid_input};
use serialize::{Encodable, Decodable, Encoder, Decoder};
use std::old_io::{IoError, IoResult};

pub enum RpcMessage {
  RpcRequest      {msgid: u32, method: String, params: Vec<Value>}, // 0
  RpcResponse     {msgid: u32, error: Value, result: Value}, // 1
  RpcNotification {method: String, params: Vec<Value>} // 2
}

impl Encodable for RpcMessage {
  fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
    match *self {
      RpcMessage::RpcRequest {msgid, ref method, ref params} => {
        (0usize, msgid, method, params).encode(s)
      }
      RpcMessage::RpcResponse {msgid, ref error, ref result} => {
        (1usize, msgid, error, result).encode(s)
      }
      RpcMessage::RpcNotification {ref method, ref params} => {
        (2usize, method, params).encode(s)
      }
    }
  }
}

impl<R: Reader> Decodable<Decoder<R>, IoError> for RpcMessage {
  fn decode(s: &mut Decoder<R>) -> IoResult<RpcMessage> {
    let len = try!(s._read_vec_len());
    let ty: usize = try!(Decodable::decode(s));

    match ty {
      0 => {
        if len != 4 { return Err(_invalid_input("Invalid msgpack-rpc message array length")) }
        let msgid = try!(Decodable::decode(s));
        let method = try!(Decodable::decode(s));
        let params = try!(Decodable::decode(s));
        Ok(RpcMessage::RpcRequest {msgid: msgid, method: method, params: params})
      }
      1 => {
        if len != 4 { return Err(_invalid_input("Invalid msgpack-rpc message array length")) }
        let msgid = try!(Decodable::decode(s));
        let error = try!(Decodable::decode(s));
        let result = try!(Decodable::decode(s));
        Ok(RpcMessage::RpcResponse {msgid: msgid, error: error, result: result})
      }
      2 => {
        if len != 3 { return Err(_invalid_input("Invalid msgpack-rpc message array length")) }
        let method = try!(Decodable::decode(s));
        let params = try!(Decodable::decode(s));
        Ok(RpcMessage::RpcNotification {method: method, params: params})
      }
      _ => {
        Err(_invalid_input("Invalid msgpack-rpc message type"))
      }
    }

  }
}
