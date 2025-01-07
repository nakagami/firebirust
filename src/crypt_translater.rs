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

use chacha20::cipher::{NewCipher, StreamCipher};
use chacha20::{ChaCha20, Key, Nonce};

pub(crate) trait CryptTranslator {
    fn translate(&mut self, plain: &[u8]) -> Vec<u8>;
}

#[derive(Debug)]
pub(crate) struct ChaCha {
    cipher: ChaCha20,
}

impl ChaCha {
    pub fn new(key: &[u8], nonce: &[u8]) -> ChaCha {
        let key = Key::from_slice(key);
        let nonce = Nonce::from_slice(&nonce);
        let cipher = ChaCha20::new(&key, &nonce);

        ChaCha { cipher }
    }
}

impl CryptTranslator for ChaCha {
    fn translate(&mut self, plain: &[u8]) -> Vec<u8> {
        let mut enc: Vec<u8> = Vec::new();
        enc.extend_from_slice(&plain);
        self.cipher.apply_keystream(&mut enc);
        enc
    }
}

#[derive(Debug)]
pub(crate) struct Arc4 {
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

#[test]
fn test_chacha() {
    use hex;
    let key =
        hex::decode("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f").unwrap();
    let nonce = hex::decode("000000000000000000000000").unwrap();
    let mut a1 = ChaCha::new(&key, &nonce);
    let enc = a1.translate(b"plain text");

    let mut a2 = ChaCha::new(&key, &nonce);
    let plain = a2.translate(&enc);
    assert_eq!(&plain, b"plain text");
}
