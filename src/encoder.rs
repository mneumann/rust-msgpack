use std::io::Write;
use byteorder::{BigEndian, WriteBytesExt};
use std::{self, mem};

#[inline]
fn encode_u7<W:Write>(wr: &mut W, val: u8) -> std::io::Result<()> {
    debug_assert!(val <= 127);
    wr.write_u8(val as u8)
}

#[inline]
fn encode_u8<W:Write>(wr: &mut W, val: u8) -> std::io::Result<()> {
    try!(wr.write_u8(0xcc));
    wr.write_u8(val)
}

#[inline]
fn encode_u16<W:Write>(wr: &mut W, val: u16) -> std::io::Result<()> {
    try!(wr.write_u8(0xcd));
    wr.write_u16::<BigEndian>(val)
}

#[inline]
fn encode_u32<W:Write>(wr: &mut W, val: u32) -> std::io::Result<()> {
    try!(wr.write_u8(0xce));
    wr.write_u32::<BigEndian>(val)
}

#[inline]
fn encode_u64<W:Write>(wr: &mut W, val: u64) -> std::io::Result<()> {
    try!(wr.write_u8(0xcf));
    wr.write_u64::<BigEndian>(val)
}

/// Encodes the most efficient representation of the given unsigned integer
pub fn encode_unsigned<W:Write>(wr: &mut W, val: u64) -> std::io::Result<()> {
    if val <= 127 {
        encode_u7(wr, val as u8)
    }
    else if val <= std::u8::MAX as u64 {
        encode_u8(wr, val as u8)
    }
    else if val <= std::u16::MAX as u64 {
        encode_u16(wr, val as u16)
    }
    else if val <= std::u32::MAX as u64 {
        encode_u32(wr, val as u32)
    }
    else {
        encode_u64(wr, val)
    }
}

#[inline]
fn encode_op_len<W:Write>(wr: &mut W, len: u32, op_sz1: Option<(u8, u32)>, op2_opt: Option<u8>, op16: u8, op32: u8) -> std::io::Result<()> {
    if let Some((op1, sz1)) = op_sz1 {
        if len <= sz1 {
            return wr.write_u8(op1 | ((len & sz1) as u8))
        }
    }
    if let Some(op2) = op2_opt {
        if len <= std::u8::MAX as u32 {
            try!(wr.write_u8(op2));
            return wr.write_u8(len as u8)
        }
    }
    if len <= std::u16::MAX as u32 {
        try!(wr.write_u8(op16));
        wr.write_u16::<BigEndian>(len as u16)
    } else {
        try!(wr.write_u8(op32));
        wr.write_u32::<BigEndian>(len)
    }
}

pub fn encode_str_len<W:Write>(wr: &mut W, len: u32) -> std::io::Result<()> {
    encode_op_len(wr, len, Some((0xa0, 31)), Some(0xd9), 0xda, 0xdb)
}

pub fn encode_bin_len<W:Write>(wr: &mut W, len: u32) -> std::io::Result<()> {
    encode_op_len(wr, len, None, Some(0xc4), 0xc5, 0xc6)
}

pub fn encode_vec_len<W:Write>(wr: &mut W, len: u32) -> std::io::Result<()> {
    encode_op_len(wr, len, Some((0x90, 15)), None, 0xdc, 0xdd)
}

pub fn encode_map_len<W:Write>(wr: &mut W, len: u32) -> std::io::Result<()> {
    encode_op_len(wr, len, Some((0x80, 15)), None, 0xde, 0xdf)
}

pub fn encode_nil<W:Write>(wr: &mut W) -> std::io::Result<()> {
    wr.write_u8(0xc0)
}

pub fn encode_bool<W:Write>(wr: &mut W, val: bool) -> std::io::Result<()> {
    if val {
        wr.write_u8(0xc3)
    } else {
        wr.write_u8(0xc2)
    }
}

pub fn encode_str<W:Write>(wr: &mut W, val: &str) -> std::io::Result<()> {
    let len = val.len();
    assert!(len <= std::u32::MAX as usize);
    try!(encode_str_len(wr, len as u32));
    wr.write_all(val.as_bytes())
}

pub fn encode_f32<W:Write>(wr: &mut W, val: f32) -> std::io::Result<()> {
    try!(wr.write_u8(0xca));
    unsafe { wr.write_u32::<BigEndian>(mem::transmute(val)) }
}

pub fn encode_f64<W:Write>(wr: &mut W, val: f64) -> std::io::Result<()> {
    try!(wr.write_u8(0xcb));
    unsafe { wr.write_u64::<BigEndian>(mem::transmute(val)) }
}
