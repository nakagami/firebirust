// MIT License
//
// Copyright (c) 2021 Hajime Nakagami<nakagami@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
use std::io::prelude::*;
use std::mem::transmute;
use std::str;

use crypto::digest::Digest;
use crypto::sha1::Sha1;
use hex;
use num_bigint::{BigInt, BigUint, Sign};

pub fn int32_to_bytes(i: u32) -> [u8; 4] {
    // little endian u32 to Vec<u8>
    unsafe { transmute(i.to_le()) }
}

pub fn bint32_to_bytes(i: u32) -> [u8; 4] {
    // big endian u32 to Vec<u8>
    unsafe { transmute(i.to_be()) }
}

pub fn int16_to_bytes(i: u16) -> [u8; 2] {
    // little endian u16 to Vec<u8>
    unsafe { transmute(i.to_le()) }
}

pub fn f32_to_bytes(f: f32) -> [u8; 4] {
    f.to_le_bytes()
}

pub fn f64_to_bytes(f: f64) -> [u8; 8] {
    f.to_le_bytes()
}

pub fn bytes_to_str(b: &[u8]) -> String {
    str::from_utf8(b).unwrap().to_string()
}

pub fn bytes_to_int32(b: &[u8]) -> i32 {
    let mut tmp: [u8; 4] = [b[0], b[1], b[2], b[3]];
    let v: i32 = unsafe { transmute::<[u8; 4], i32>(tmp) };
    v
}

pub fn bytes_to_uint32(b: &[u8]) -> u32 {
    // little endian u32
    ((b[0] as u32) << 0) + ((b[1] as u32) << 8) + ((b[2] as u32) << 16) + ((b[3] as u32) << 24)
}

pub fn bytes_to_buint32(b: &[u8]) -> u32 {
    // big endian u32
    ((b[0] as u32) << 24) + ((b[1] as u32) << 16) + ((b[2] as u32) << 8) + ((b[3] as u32) << 0)
}

pub fn bytes_to_int16(b: &[u8]) -> i16 {
    let mut tmp: [u8; 2] = [b[0], b[1]];
    let v: i16 = unsafe { transmute::<[u8; 2], i16>(tmp) };
    v
}

pub fn bytes_to_uint16(b: &[u8]) -> u16 {
    // little endian u16
    ((b[0] as u16) << 0) + ((b[1] as u16) << 8)
}

pub fn bytes_to_buint16(b: &[u8]) -> u16 {
    // big endian u16
    ((b[2] as u16) << 8) + ((b[3] as u16) << 0)
}

pub fn bytes_to_uint64(b: &[u8]) -> u64 {
    // little endian u64
    ((b[0] as u64) << 0)
        + ((b[1] as u64) << 8)
        + ((b[2] as u64) << 16)
        + ((b[3] as u64) << 24)
        + ((b[4] as u64) << 32)
        + ((b[5] as u64) << 40)
        + ((b[6] as u64) << 48)
        + ((b[7] as u64) << 56)
}

pub fn bytes_to_buint64(b: &[u8]) -> u64 {
    // big endian u64
    ((b[0] as u64) << 56)
        + ((b[1] as u64) << 48)
        + ((b[2] as u64) << 40)
        + ((b[3] as u64) << 32)
        + ((b[4] as u64) << 24)
        + ((b[5] as u64) << 16)
        + ((b[6] as u64) << 8)
        + ((b[7] as u64) << 0)
}

pub fn bytes_to_f32(b: &[u8]) -> f32 {
    let mut tmp: [u8; 4] = [b[0], b[1], b[2], b[3]];
    let v: f32 = unsafe { transmute::<[u8; 4], f32>(tmp) };
    v
}

pub fn bytes_to_f64(b: &[u8]) -> f64 {
    let mut tmp: [u8; 8] = [b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]];
    let v: f64 = unsafe { transmute::<[u8; 8], f64>(tmp) };
    v
}

pub fn big_int_from_hex_string(s: &[u8]) -> BigInt {
    BigInt::parse_bytes(s, 16).unwrap()
}

pub fn big_int_from_string(s: &[u8]) -> BigInt {
    BigInt::parse_bytes(s, 10).unwrap()
}

pub fn big_uint_from_string(s: &[u8]) -> BigUint {
    BigUint::parse_bytes(s, 10).unwrap()
}

pub fn big_int_to_bytes(i: &BigInt) -> Vec<u8> {
    assert_eq!(i.sign(), Sign::Plus);
    let (_, v) = i.to_bytes_be();
    v
}

pub fn bytes_to_big_int(b: &[u8]) -> BigInt {
    BigInt::from_bytes_be(Sign::Plus, b)
}

pub fn big_int_to_sha1_hex(i: &BigInt) -> String {
    let mut hasher = Sha1::new();
    hasher.input(&big_int_to_bytes(i));
    hasher.result_str()
}

pub fn big_int_to_sha1(i: &BigInt) -> Vec<u8> {
    hex::decode(big_int_to_sha1_hex(i)).unwrap()
}

pub fn xdr_bytes(b: &[u8]) -> Vec<u8> {
    let mut write_buf: Vec<u8> = Vec::new();
    let n: usize = b.len();
    write_buf.write(&(n as u32).to_be_bytes()).unwrap();
    write_buf.write(b).unwrap();
    let mut padding: usize = 0;
    if n % 4 != 0 {
        padding = 4 - n % 4;
    }
    for _ in 0..padding {
        write_buf.push(0);
    }
    write_buf
}

pub fn bytes_to_blr(b: &[u8]) -> (Vec<u8>, Vec<u8>) {
    // TODO:
    let mut blr: Vec<u8> = Vec::new();
    let mut v: Vec<u8> = Vec::new();
    (blr, v)
}
