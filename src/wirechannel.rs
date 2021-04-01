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

use super::error::Error;
use std::io::prelude::*;
use std::net::TcpStream;

trait CryptTranslator {
    fn translate(&mut self, plain: &[u8]) -> Vec<u8>;
}

#[derive(Debug)]
struct Arc4 {
    state: Vec<u8>,
    x: usize,
    y: usize,
}

impl Arc4 {
    pub fn new(key: &[u8]) -> Arc4 {
        let mut state: Vec<u8> = Vec::new();
        for i in 0..256 {
            state.push(i as u8);
        }
        assert_eq!(state.len(), 256);

        let mut index1: usize = 0;
        let mut index2: usize = 0;

        for i in 0..256 {
            index2 = (key[index1] as usize + state[i] as usize + index2) % 256;
            let tmp: u8 = state[i];
            state[i] = state[index2];
            state[index2] = tmp;
            index1 = (index1 + 1) % key.len();
        }

        Arc4 { state, x: 0, y: 0 }
    }
}

impl CryptTranslator for Arc4 {
    fn translate(&mut self, plain: &[u8]) -> Vec<u8> {
        let mut enc: Vec<u8> = Vec::new();
        for i in 0..plain.len() {
            self.x = (self.x + 1) % 256;
            self.y = (self.y + self.state[self.x] as usize) % 256;

            let tmp = self.state[self.x];
            self.state[self.x] = self.state[self.y];
            self.state[self.y] = tmp;

            let xor_index: usize =
                (self.state[self.x] as usize + self.state[self.y] as usize) % 256;
            enc.push(plain[i] ^ self.state[xor_index]);
        }

        enc
    }
}

#[derive(Debug)]
pub struct WireChannel {
    stream: TcpStream,
    read_buf: Vec<u8>,
    read_trans: Option<Arc4>,
    write_trans: Option<Arc4>,
}

impl WireChannel {
    pub fn new(host: &str, port: u16) -> Result<WireChannel, Error> {
        let stream = TcpStream::connect(format!("{}:{}", host, port))?;
        Ok(WireChannel {
            stream,
            read_buf: Vec::new(),
            read_trans: None,
            write_trans: None,
        })
    }

    pub fn set_arc4_key(&mut self, key: &[u8]) {
        self.read_trans = Some(Arc4::new(key));
        self.write_trans = Some(Arc4::new(key));
    }

    pub fn read(&mut self, n: usize) -> Result<Vec<u8>, Error> {
        let mut input_buf = [0u8; 4096];

        while self.read_buf.len() < n {
            let ln = self.stream.read(&mut input_buf)?;
            self.read_buf.extend(input_buf[..ln].iter().copied());
        }

        let mut v: Vec<u8> = Vec::new();
        for _ in 0..n {
            v.push(self.read_buf.remove(0));
        }

        if let Some(ref mut trans) = self.read_trans {
            let translated: Vec<u8> = (*trans.translate(&v)).to_vec();
            Ok(translated)
        } else {
            Ok(v)
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        if let Some(ref mut trans) = self.write_trans {
            self.stream.write(&*trans.translate(buf))?;
        } else {
            self.stream.write(buf)?;
        }
        Ok(())
    }
}

#[test]
fn test_arc4() {
    let mut a1 = Arc4::new(b"a key");
    let enc = a1.translate(b"plain text");
    let correct: Vec<u8> = vec![0x4b, 0x4b, 0xdc, 0x65, 0x02, 0xb3, 0x08, 0x17, 0x48, 0x82];
    assert_eq!(&enc, &correct);

    let mut a2 = Arc4::new(b"a key");
    let plain = a2.translate(&enc);
    assert_eq!(&plain, b"plain text");
}
