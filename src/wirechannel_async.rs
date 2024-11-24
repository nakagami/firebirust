// MIT License
//
// Copyright (c) 2021-2024 Hajime Nakagami<nakagami@gmail.com>
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

use super::crypt_translater::{Arc4, ChaCha, CryptTranslator};
use super::error::Error;
use async_std::io::prelude::*;
use async_std::net::TcpStream;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use hex;

pub struct WireChannelAsync {
    stream: TcpStream,
    read_buf: Vec<u8>,
    read_trans: Option<Box<dyn CryptTranslator>>,
    write_trans: Option<Box<dyn CryptTranslator>>,
}

impl WireChannelAsync {
    pub async fn new(host: &str, port: u16) -> Result<WireChannelAsync, Error> {
        let stream = TcpStream::connect(format!("{}:{}", host, port)).await?;
        Ok(WireChannelAsync {
            stream,
            read_buf: Vec::new(),
            read_trans: None,
            write_trans: None,
        })
    }

    pub fn set_crypt_key(&mut self, plugin: &[u8], key: &[u8], nonce: &[u8]) {
        if plugin == b"ChaCha" {
            let mut hasher = Sha256::new();
            hasher.input(&key);
            let key = &hex::decode(hasher.result_str()).unwrap();
            self.read_trans = Some(Box::new(ChaCha::new(key, nonce)));
            self.write_trans = Some(Box::new(ChaCha::new(key, nonce)));
        } else if plugin == b"Arc4" {
            self.read_trans = Some(Box::new(Arc4::new(key)));
            self.write_trans = Some(Box::new(Arc4::new(key)));
        }
    }

    pub async fn read(&mut self, n: usize) -> Result<Vec<u8>, Error> {
        let mut input_buf = [0u8; 4096];

        while self.read_buf.len() < n {
            let ln = self.stream.read(&mut input_buf).await?;
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

    pub async fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        if let Some(ref mut trans) = self.write_trans {
            self.stream.write(&*trans.translate(buf)).await?;
        } else {
            self.stream.write(buf).await?;
        }
        Ok(())
    }
}
