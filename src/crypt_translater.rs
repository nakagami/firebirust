// MIT License
//
// Copyright (c) 2021-2025 Hajime Nakagami<nakagami@gmail.com>
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

fn quaterround_u32(state: &mut [u32; 16], i: usize, j: usize, k: usize, l: usize) {
    let mut a = state[i];
    let mut b = state[j];
    let mut c = state[k];
    let mut d = state[l];
    a = a.wrapping_add(b);
    d ^= a;
    d = d.rotate_left(16);
    c = c.wrapping_add(d);
    b ^= c;
    b = b.rotate_left(12);
    a = a.wrapping_add(b);
    d ^= a;
    d = d.rotate_left(8);
    c = c.wrapping_add(d);
    b ^= c;
    b = b.rotate_left(7);
    state[i] = a;
    state[j] = b;
    state[k] = c;
    state[l] = d;
}

pub(crate) trait CryptTranslator {
    fn translate(&mut self, plain: &[u8]) -> Vec<u8>;
}

#[derive(Debug)]
pub(crate) struct ChaCha {
    key: Vec<u32>,
    nonce: Vec<u32>,
    counter: u64,
    block: [u8; 64],
    block_pos: usize,
}

impl ChaCha {
    pub fn new(key: &[u8], nonce: &[u8]) -> ChaCha {
        if key.len() != 32 {
            panic!("ChaCha key is 32 bytes length");
        }
        if nonce.len() != 8 && nonce.len() != 12 {
            panic!("ChaCha nonce is 8 bytes or 12 bytes length");
        }

        let key = key
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect::<Vec<u32>>();

        let nonce = nonce
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect::<Vec<u32>>();

        if key.len() != 8 {
            panic!("Invalid key length.");
        }
        if nonce.len() != 2 && nonce.len() != 3 {
            panic!("Invalid nonce length.");
        }

        let counter = 0u64;

        let mut chacha = ChaCha {
            key,
            nonce,
            counter,
            block: [0; 64],
            block_pos: 0,
        };
        chacha.set_chacha20_round_block();
        chacha
    }

    fn to_state(&self) -> [u32; 16] {
        let mut state = [0u32; 16];
        state[0] = 0x61707865;
        state[1] = 0x3320646e;
        state[2] = 0x79622d32;
        state[3] = 0x6b206574;
        (0..self.key.len()).into_iter().for_each(|i| {
            state[4 + i] = self.key[i];
        });

        if self.nonce.len() == 3 {
            state[12] = self.counter as u32;
            state[13] = self.nonce[0];
            state[14] = self.nonce[1];
            state[15] = self.nonce[2];
        } else {
            state[12] = self.counter as u32;
            state[13] = (self.counter >> 32) as u32;
            state[14] = self.nonce[0];
            state[15] = self.nonce[1];
        }
        state
    }

    fn set_chacha20_round_block(&mut self) {
        let state = self.to_state();

        let mut x = [0u32; 16];
        x.copy_from_slice(&state);
        for _ in 0..10 {
            quaterround_u32(&mut x, 0, 4, 8, 12);
            quaterround_u32(&mut x, 1, 5, 9, 13);
            quaterround_u32(&mut x, 2, 6, 10, 14);
            quaterround_u32(&mut x, 3, 7, 11, 15);

            quaterround_u32(&mut x, 0, 5, 10, 15);
            quaterround_u32(&mut x, 1, 6, 11, 12);
            quaterround_u32(&mut x, 2, 7, 8, 13);
            quaterround_u32(&mut x, 3, 4, 9, 14);
        }
        for i in 0..16 {
            x[i] = x[i].wrapping_add(state[i]);
        }
        let key_stream = x
            .iter()
            .flat_map(|state| state.to_le_bytes())
            .collect::<Vec<u8>>();

        self.block.copy_from_slice(&key_stream);
        self.block_pos = 0;
    }
}

impl CryptTranslator for ChaCha {
    fn translate(&mut self, plain: &[u8]) -> Vec<u8> {
        let mut enc = vec![0u8; plain.len()];

        for i in 0..plain.len() {
            enc[i] = plain[i] ^ self.block[self.block_pos];
            self.block_pos += 1;
            if self.block.len() == self.block_pos {
                self.counter += 1;
                self.set_chacha20_round_block()
            }
        }
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
