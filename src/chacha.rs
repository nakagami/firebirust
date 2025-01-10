// MIT License
//
// Copyright (c) 2024 watson
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

type Block = [u32; 16];
#[derive(Clone)]
pub struct ChaCha {
  state: Block,
}

impl ChaCha {
  fn new() -> Self {
    Self { state: [0; 16] }
  }
  fn quarter_round(state: &mut [u32], i: usize, j: usize, k: usize, l: usize) {
    /*
    1.  a += b; d ^= a; d <<<= 16;
    2.  c += d; b ^= c; b <<<= 12;
    3.  a += b; d ^= a; d <<<= 8;
    4.  c += d; b ^= c; b <<<= 7;
    */
    //let rot32 = |x: u32, n: u32| x << n | x >> (32 - n);
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

  fn init(key: &[u32], counter: u32, nonce: &[u32]) -> Block {
    /*
    The ChaCha20 state is initialized as follows:

    o  The first four words (0-3) are constants: 0x61707865, 0x3320646e,
       0x79622d32, 0x6b206574.

    o  The next eight words (4-11) are taken from the 256-bit key by
       reading the bytes in little-endian order, in 4-byte chunks.

    o  Word 12 is a block counter.  Since each block is 64-byte, a 32-bit
       word is enough for 256 gigabytes of data.

    o  Words 13-15 are a nonce, which should not be repeated for the same
       key.  The 13th word is the first 32 bits of the input nonce taken
       as a little-endian integer, while the 15th word is the last 32
       bits.

        cccccccc  cccccccc  cccccccc  cccccccc
        kkkkkkkk  kkkkkkkk  kkkkkkkk  kkkkkkkk
        kkkkkkkk  kkkkkkkk  kkkkkkkk  kkkkkkkk
        bbbbbbbb  nnnnnnnn  nnnnnnnn  nnnnnnnn

        c=constant k=key b=blockcount n=nonce */

    /*const 0x61707865, 0x3320646e,
      0x79622d32, 0x6b206574.
    */
    if !(key.len() == 8 && nonce.len() == 3) {
      panic!("block key size or nonce is not varid");
    }
    let mut block: Block = [0; 16];
    block[0] = 0x61707865;
    block[1] = 0x3320646e;
    block[2] = 0x79622d32;
    block[3] = 0x6b206574;

    (0..key.len()).into_iter().for_each(|i| {
      block[4 + i] = key[i];
    });
    block[12] = counter;

    (0..nonce.len()).into_iter().for_each(|i| {
      block[13 + i] = nonce[i];
    });

    println!("block init is {:x?}", block);
    block
  }

  fn block(key: &[u32], counter: u32, nonce: &[u32]) -> Block {
    let init = ChaCha::init(key, counter, nonce);
    let mut state = init;
    let inner_block = |state: &mut Block| {
      /*
       inner_block (state):
        Qround(state, 0, 4, 8,12)
        Qround(state, 1, 5, 9,13)
        Qround(state, 2, 6,10,14)
        Qround(state, 3, 7,11,15)
        Qround(state, 0, 5,10,15)
        Qround(state, 1, 6,11,12)
        Qround(state, 2, 7, 8,13)
        Qround(state, 3, 4, 9,14)
        end
      */
      //
      ChaCha::quarter_round(state, 0, 4, 8, 12);
      ChaCha::quarter_round(state, 1, 5, 9, 13);
      ChaCha::quarter_round(state, 2, 6, 10, 14);
      ChaCha::quarter_round(state, 3, 7, 11, 15);

      ChaCha::quarter_round(state, 0, 5, 10, 15);
      ChaCha::quarter_round(state, 1, 6, 11, 12);
      ChaCha::quarter_round(state, 2, 7, 8, 13);
      ChaCha::quarter_round(state, 3, 4, 9, 14);
    };

    for i in 0..10 {
      inner_block(&mut state);
    }
    state
      .iter_mut()
      .enumerate()
      .for_each(|(i, block)| *block = (*block).wrapping_add(init[i]));
    state
  }

  pub fn serialize(state: &[u32]) -> Vec<u8> {
    state
      .iter()
      .flat_map(|x| x.to_le_bytes())
      .collect::<Vec<u8>>()
  }
  fn encode(key: &[u32], counter: u32, nonce: &[u32], plain_text: &[u8]) -> Vec<u8> {
    let be_to_u32 = |x: &[u8]| {
      let mut ret = 0u32;
      for x in x.iter().enumerate() {
        ret ^= (*x.1 as u32) << (8u32 * (3 - x.0 as u32));
      }
      println!("be text is {:x?}", ret);

      ret
    };
    let le_to_u32 = |x: &[u8]| {
      let mut ret = 0u32;
      //println!("le is {:x?} ", x);
      for x in x.iter().enumerate() {
        ret ^= (*x.1 as u32) << (8u32 * x.0 as u32);
      }
      ret
    };
    let mut encrypt_message = plain_text
      .chunks(64)
      .enumerate()
      .flat_map(|(i, chunk)| {
        println!("encrypt chunk size is {}", chunk.len());
        let key_stream = ChaCha::block(key, counter + i as u32, nonce)
          .iter()
          .flat_map(|x| x.to_le_bytes())
          .collect::<Vec<u8>>();
        let mut chunk = chunk.clone().to_vec();
        println!("key stream is {:x?}", key_stream);
        println!("chunk {} is {:x?}", i, chunk);
        for (a, b) in chunk.iter_mut().zip(key_stream.iter()) {
          println!("key to le is {:x?}", b);
          *a ^= b;
        }
        println!("{} block result is {:x?}", i, chunk);
        chunk
      })
      .collect::<Vec<u8>>();
    encrypt_message
  }
}
#[cfg(test)]
mod test {
  use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng, ChaChaRng};

  use super::*;

  #[test]
  fn quarter_test() {
    /*
    o  a = 0x11111111
    o  b = 0x01020304
    o  c = 0x9b8d6f43
    o  d = 0x01234567
    o  c = c + d = 0x77777777 + 0x01234567 = 0x789abcde
    o  b = b ^ c = 0x01020304 ^ 0x789abcde = 0x7998bfda
    o  b = b <<< 7 = 0x7998bfda <<< 7 = 0xcc5fed3c




    o  a = 0xea2a92f4
    o  b = 0xcb1cf8ce
    o  c = 0x4581472e
    o  d = 0x5881c4bb
    */

    let mut state: [u32; 4] = [0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567];
    let ans: [u32; 4] = [0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb];
    ChaCha::quarter_round(&mut state, 0, 1, 2, 3);
    assert_eq!(state, ans);
  }

  #[test]
  fn block_test() {
    /*

    For a test vector, we will use the following inputs to the ChaCha20
    block function:

    o  Key = 00:01:02:03:04:05:06:07:08:09:0a:0b:0c:0d:0e:0f:10:11:12:13:
       14:15:16:17:18:19:1a:1b:1c:1d:1e:1f.  The key is a sequence of
       octets with no particular structure before we copy it into the
       ChaCha state.

    o  Nonce = (00:00:00:09:00:00:00:4a:00:00:00:00)

    o  Block Count = 1.

    After setting up the ChaCha state, it looks like this:

    ChaCha state with the key setup.

        61707865  3320646e  79622d32  6b206574
        03020100  07060504  0b0a0908  0f0e0d0c
        13121110  17161514  1b1a1918  1f1e1d1c
        00000001  09000000  4a000000  00000000

    After running 20 rounds (10 column rounds interleaved with 10
    "diagonal rounds"), the ChaCha state looks like this:

    ChaCha state after 20 rounds

        837778ab  e238d763  a67ae21e  5950bb2f
        c4f2d0c7  fc62bb2f  8fa018fc  3f5ec7b7
        335271c2  f29489f3  eabda8fc  82e46ebd
        d19c12b4  b04e16de  9e83d0cb  4e3c50a2

    Finally, we add the original state to the result (simple vector or
    matrix addition), giving this:

    ChaCha state at the end of the ChaCha20 operation

        e4e7f110  15593bd1  1fdd0f50  c47120a3
        c7f4d1c7  0368c033  9aaa2204  4e6cd4c3
        466482d2  09aa9f07  05d7c214  a2028bd9
        d19c12b5  b94e16de  e883d0cb  4e3c50a2
     */
    let le_to_u32 = |x: &[u8]| {
      let mut ret = 0u32;
      //println!("le is {:x?} ", x);
      for x in x.iter().enumerate() {
        ret ^= (*x.1 as u32) << (8u32 * x.0 as u32);
      }
      ret
    };
    let be_to_u32 = |x: &[u8]| {
      let mut ret = 0u32;
      for x in x.iter().enumerate() {
        ret += (*x.1 as u32) << (8u32 * (3 - x.0 as u32));
      }
      ret
    };

    let key = hex::decode("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f")
      .expect("key is invalid")
      .chunks(4)
      .map(le_to_u32)
      .collect::<Vec<u32>>();
    let nonce = hex::decode(b"000000090000004a00000000")
      .expect("nonce is invalid")
      .chunks(4)
      .map(le_to_u32)
      .collect::<Vec<u32>>();
    let counter = 1u32;
    let ans: Vec<u32> = "e4e7f110  15593bd1  1fdd0f50  c47120a3
            c7f4d1c7  0368c033  9aaa2204  4e6cd4c3
            466482d2  09aa9f07  05d7c214  a2028bd9
            d19c12b5  b94e16de  e883d0cb  4e3c50a2"
      .split_whitespace()
      .map(|x| {
        let x = hex::decode(x).expect("ans decode fail");
        let ret = u32::from_be_bytes(x.try_into().unwrap());
        ret
      })
      .collect();
    //println!("ans is {:x?}", ans);
    println!(
      "key is {:x?} , nonce is {:x?} , counter is {}",
      key, nonce, counter
    );
    let chacha = ChaCha::block(&key, counter, &nonce);
    println!("block vec is {:x?}", chacha);
    assert_eq!(ans, chacha);
    println!("serialize block is {:x?}", ChaCha::serialize(&chacha));
  }

  #[test]
  fn encode_test() {
    /*
         For a test vector, we will use the following inputs to the ChaCha20
       block function:

       o  Key = 00:01:02:03:04:05:06:07:08:09:0a:0b:0c:0d:0e:0f:10:11:12:13:
          14:15:16:17:18:19:1a:1b:1c:1d:1e:1f.

       o  Nonce = (00:00:00:00:00:00:00:4a:00:00:00:00).

       o  Initial Counter = 1.

       We use the following for the plaintext.  It was chosen to be long
       enough to require more than one block, but not so long that it would
       make this example cumbersome (so, less than 3 blocks):

      Plaintext Sunscreen:
      000  4c 61 64 69 65 73 20 61 6e 64 20 47 65 6e 74 6c  Ladies and Gentl
      016  65 6d 65 6e 20 6f 66 20 74 68 65 20 63 6c 61 73  emen of the clas
      032  73 20 6f 66 20 27 39 39 3a 20 49 66 20 49 20 63  s of '99: If I c
      048  6f 75 6c 64 20 6f 66 66 65 72 20 79 6f 75 20 6f  ould offer you o
      064  6e 6c 79 20 6f 6e 65 20 74 69 70 20 66 6f 72 20  nly one tip for
      080  74 68 65 20 66 75 74 75 72 65 2c 20 73 75 6e 73  the future, suns
      096  63 72 65 65 6e 20 77 6f 75 6c 64 20 62 65 20 69  creen would be i
      112  74 2e                                            t.





    Nir & Langley                 Informational                    [Page 11]

    RFC 7539                   ChaCha20 & Poly1305                  May 2015


       The following figure shows four ChaCha state matrices:

       1.  First block as it is set up.

       2.  Second block as it is set up.  Note that these blocks are only
           two bits apart -- only the counter in position 12 is different.

       3.  Third block is the first block after the ChaCha20 block
           operation.

       4.  Final block is the second block after the ChaCha20 block
           operation was applied.

       After that, we show the keystream.

       First block setup:
           61707865  3320646e  79622d32  6b206574
           03020100  07060504  0b0a0908  0f0e0d0c
           13121110  17161514  1b1a1918  1f1e1d1c
           00000001  00000000  4a000000  00000000

       Second block setup:
           61707865  3320646e  79622d32  6b206574
           03020100  07060504  0b0a0908  0f0e0d0c
           13121110  17161514  1b1a1918  1f1e1d1c
           00000002  00000000  4a000000  00000000

       First block after block operation:
           f3514f22  e1d91b40  6f27de2f  ed1d63b8
           821f138c  e2062c3d  ecca4f7e  78cff39e
           a30a3b8a  920a6072  cd7479b5  34932bed
           40ba4c79  cd343ec6  4c2c21ea  b7417df0

       Second block after block operation:
           9f74a669  410f633f  28feca22  7ec44dec
           6d34d426  738cb970  3ac5e9f3  45590cc4
           da6e8b39  892c831a  cdea67c1  2b7e1d90
           037463f3  a11a2073  e8bcfb88  edc49139

       Keystream:
       22:4f:51:f3:40:1b:d9:e1:2f:de:27:6f:b8:63:1d:ed:8c:13:1f:82:3d:2c:06
       e2:7e:4f:ca:ec:9e:f3:cf:78:8a:3b:0a:a3:72:60:0a:92:b5:79:74:cd:ed:2b
       93:34:79:4c:ba:40:c6:3e:34:cd:ea:21:2c:4c:f0:7d:41:b7:69:a6:74:9f:3f
       63:0f:41:22:ca:fe:28:ec:4d:c4:7e:26:d4:34:6d:70:b9:8c:73:f3:e9:c5:3a
       c4:0c:59:45:39:8b:6e:da:1a:83:2c:89:c1:67:ea:cd:90:1d:7e:2b:f3:63






    Nir & Langley                 Informational                    [Page 12]

    RFC 7539                   ChaCha20 & Poly1305                  May 2015


       Finally, we XOR the keystream with the plaintext, yielding the
       ciphertext:

      Ciphertext Sunscreen:
      000  6e 2e 35 9a 25 68 f9 80 41 ba 07 28 dd 0d 69 81  n.5.%h..A..(..i.
      016  e9 7e 7a ec 1d 43 60 c2 0a 27 af cc fd 9f ae 0b  .~z..C`..'......
      032  f9 1b 65 c5 52 47 33 ab 8f 59 3d ab cd 62 b3 57  ..e.RG3..Y=..b.W
      048  16 39 d6 24 e6 51 52 ab 8f 53 0c 35 9f 08 61 d8  .9.$.QR..S.5..a.
      064  07 ca 0d bf 50 0d 6a 61 56 a3 8e 08 8a 22 b6 5e  ....P.jaV....".^
      080  52 bc 51 4d 16 cc f8 06 81 8c e9 1a b7 79 37 36  R.QM.........y76
      096  5a f9 0b bf 74 a3 5b e6 b4 0b 8e ed f2 78 5e 42  Z...t.[......x^B
      112  87 4d                                            .M

         */
    let le_to_u32 = |x: &[u8]| {
      let mut ret = 0u32;
      //println!("le is {:x?} ", x);
      for x in x.iter().enumerate() {
        ret ^= (*x.1 as u32) << (8u32 * x.0 as u32);
      }
      ret
    };
    let be_to_u32 = |x: &[u8]| {
      let mut ret = 0u32;
      for x in x.iter().enumerate() {
        ret += (*x.1 as u32) << (8u32 * (4 - x.0 as u32));
      }
      ret
    };

    let key = hex::decode("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f")
      .expect("key is invalid")
      .chunks(4)
      .map(le_to_u32)
      .collect::<Vec<u32>>();
    let nonce = hex::decode(b"000000000000004a00000000")
      .expect("nonce is invalid")
      .chunks(4)
      .map(le_to_u32)
      .collect::<Vec<u32>>();
    let counter = 1u32;
    let plain_text = b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it.";
    let ans = hex::decode("6e2e359a2568f98041ba0728dd0d6981e97e7aec1d4360c20a27afccfd9fae0bf91b65c5524733ab8f593dabcd62b3571639d624e65152ab8f530c359f0861d807ca0dbf500d6a6156a38e088a22b65e52bc514d16ccf806818ce91ab77937365af90bbf74a35be6b40b8eedf2785e42874d").expect("stream ans is error");
    //let ans = ans.chunks(4).map(le_to_u32).collect::<Vec<u32>>();
    println!("plain text is {:x?} ", plain_text);
    let encode_text = ChaCha::encode(&key, counter, &nonce, plain_text);
    println!("encode text is {:x?}", encode_text);
    assert_eq!(ans, encode_text);
  }

  #[test]
  fn pseudorandom_test() {
    use rand::{Rng, SeedableRng};
    let seed: [u8; 32] = [
      0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
      0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
      0x1e, 0x1f,
    ];
    let mut ans: ChaCha20Rng = SeedableRng::from_seed(seed);
    let ans = (0..16)
      .into_iter()
      .map(|x| ans.gen::<u32>())
      .collect::<Vec<u32>>();
    println!("pseudo random answer is {:x?} ", ans);
    let le_to_u32 = |x: &[u8]| {
      let mut ret = 0u32;
      //println!("le is {:x?} ", x);
      for x in x.iter().enumerate() {
        ret ^= (*x.1 as u32) << (8u32 * x.0 as u32);
      }
      ret
    };
    let key = hex::decode("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f")
      .expect("key is invalid")
      .chunks(4)
      .map(le_to_u32)
      .collect::<Vec<u32>>();
    let nonce = hex::decode(b"000000000000000000000000")
      .expect("nonce is invalid")
      .chunks(4)
      .map(le_to_u32)
      .collect::<Vec<u32>>();
    let counter = 0u32;

    let pseudo = ChaCha::block(&key, counter, &nonce);
    //let pseudo = ChaCha::serialize(&pseudo);
    println!("pseudo random my chacha is {:x?} ", pseudo);
    assert_eq!(ans, pseudo);
  }
}
