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

// See http://srp.stanford.edu/design.html

#![allow(dead_code)]

use crypto::digest::Digest;
use crypto::sha1::Sha1;
use crypto::sha2::Sha256;
use num_bigint::BigInt;
use rand::prelude::*;

use super::*;

const SRP_KEY_SIZE: usize = 128;
const SRP_SALT_SIZE: usize = 32;

const SRP_DEBUG: bool = true;
const DEBUG_PRIVATE_KEY: &'static [u8; 64] =
    b"60975527035CF2AD1989806F0407210BC81EDC04E2762A56AFD529DDDA2D4393";
const DEBUG_SALT: &str = "02E268803000000079A478A700000002D1A6979000000026E1601C000000054F";

fn pad(v: &BigInt) -> Vec<u8> {
    let mut buf: Vec<u8> = utils::big_int_to_bytes(v);
    while buf.len() > SRP_KEY_SIZE {
        buf.remove(0);
    }
    buf
}

fn get_prime() -> (BigInt, BigInt, BigInt) {
    let prime: BigInt = utils::big_int_from_hex_string(b"E67D2E994B2F900C3F41F08F5BB2627ED0D49EE1FE767A52EFCD565CD6E768812C3E1E9CE8F0A8BEA6CB13CD29DDEBF7A96D4A93B55D488DF099A15C89DCB0640738EB2CBDD9A8F7BAB561AB1B0DC1C6CDABF303264A08D1BCA932D1F1EE428B619D970F342ABA9A65793B8B2F041AE5364350C16F735F56ECBCA87BD57B29E7");
    let g: BigInt = utils::big_int_from_string(b"2");
    let k: BigInt =
        utils::big_int_from_string(b"1277432915985975349439481660349303019122249719989");
    (prime, g, k)
}

pub fn get_scramble(key_public_a: &BigInt, key_public_b: &BigInt) -> BigInt {
    // key_a:A client public ephemeral values
    // key_b:B server public ephemeral values
    let mut hasher = Sha1::new();
    hasher.input(&pad(key_public_a));
    hasher.input(&pad(key_public_b));
    utils::big_int_from_hex_string(&hasher.result_str().as_bytes())
}

pub fn get_string_hash(s: &str) -> BigInt {
    let mut hasher = Sha1::new();
    hasher.input(s.as_bytes());
    utils::big_int_from_hex_string(&hasher.result_str().as_bytes())
}

pub fn get_user_hash(salt: &[u8], user: &str, password: &str) -> BigInt {
    let mut hash1 = Sha1::new();
    hash1.input(user.as_bytes());
    hash1.input(b":");
    hash1.input(password.as_bytes());
    let mut hash2 = Sha1::new();
    hash2.input(salt);
    hash2.input(&hex::decode(hash1.result_str()).unwrap());
    utils::bytes_to_big_int(&hex::decode(hash2.result_str()).unwrap())
}

pub fn get_client_seed() -> (BigInt, BigInt) {
    let (prime, g, _) = get_prime();
    let mut key_private_a: BigInt;
    if SRP_DEBUG {
        key_private_a = utils::big_int_from_hex_string(DEBUG_PRIVATE_KEY);
    } else {
        let r: u128 = random();
        key_private_a = BigInt::from(r);
    }

    let key_public_a = g.modpow(&key_private_a, &prime);
    (key_public_a, key_private_a)
}

pub fn get_salt() -> Vec<u8> {
    if SRP_DEBUG {
        return hex::decode(DEBUG_SALT).unwrap();
    }
    let mut buf: Vec<u8> = Vec::new();
    for _ in 0..SRP_SALT_SIZE {
        buf.push(random());
    }
    assert!(buf.len() == SRP_SALT_SIZE);
    buf
}

pub fn get_verifier(user: &str, password: &str, salt: &Vec<u8>) -> BigInt {
    let (prime, g, _k) = get_prime();
    let x = get_user_hash(salt, user, password);
    g.modpow(&x, &prime)
}

pub fn get_server_seed(v: &BigInt) -> (BigInt, BigInt) {
    let (prime, g, k) = get_prime();
    let key_private_b: BigInt;

    if SRP_DEBUG {
        key_private_b = utils::big_int_from_hex_string(DEBUG_PRIVATE_KEY);
    } else {
        let r: u128 = random();
        key_private_b = BigInt::from(r);
    }

    let gb = g.modpow(&key_private_b, &prime); // gb = pow(g, b, N)
    let kv = (&k * v) % &prime; // kv = (k * v) % N
    let key_public_b = (&kv + &gb) % &prime; // B = (kv + gb) % N

    (key_public_b, key_private_b) // (B, b)
}

pub fn get_client_session(
    user: &str,
    password: &str,
    salt: &[u8],
    key_public_a: &BigInt,
    key_public_b: &BigInt,
    key_private_a: &BigInt,
) -> Vec<u8> {
    let (prime, g, k) = get_prime();
    let u = get_scramble(key_public_a, key_public_b);
    let x = get_user_hash(salt, user, password);
    let gx = g.modpow(&x, &prime);
    let kgx = (&k * &gx) % &prime;
    let diff = (key_public_b - kgx) % &prime;
    let ux = (u * x) % &prime;
    let aux = (key_private_a + ux) % &prime;
    let session_secret = diff.modpow(&aux, &prime);
    utils::big_int_to_sha1(&session_secret)
}

pub fn get_server_session(
    user: &str,
    password: &str,
    salt: &Vec<u8>,
    key_public_a: &BigInt,
    key_public_b: &BigInt,
    key_private_b: &BigInt,
) -> Vec<u8> {
    let (prime, _, _) = get_prime();
    let u = get_scramble(key_public_a, key_public_b);
    let v = get_verifier(user, password, salt);
    let mut vu = v.modpow(&u, &prime);
    let avu = (key_public_a * vu) % &prime;
    let session_secret = avu.modpow(&key_private_b, &prime);
    utils::big_int_to_sha1(&session_secret)
}

pub fn get_client_proof(
    user: &str,
    password: &str,
    salt: &[u8],
    key_public_a: &BigInt,
    key_public_b: &BigInt,
    key_private_a: &BigInt,
    plugin_name: &str,
) -> (Vec<u8>, Vec<u8>) {
    // M = H(H(N) xor H(g), H(I), s, A, B, K)

    let (prime, g, _) = get_prime();
    let key_k = get_client_session(
        user,
        password,
        salt,
        key_public_a,
        key_public_b,
        key_private_a,
    );
    let n1 = utils::bytes_to_big_int(&utils::big_int_to_sha1(&prime));
    let n2 = utils::bytes_to_big_int(&utils::big_int_to_sha1(&g));
    let n3 = n1.modpow(&n2, &prime);
    let n4 = get_string_hash(user);

    let mut key_m: Vec<u8>;
    // Srp
    if plugin_name == "Srp" {
        let mut hasher = Sha1::new();
        hasher.input(&utils::big_int_to_bytes(&n3));
        hasher.input(&utils::big_int_to_bytes(&n4));
        hasher.input(salt);
        hasher.input(&utils::big_int_to_bytes(&key_public_a));
        hasher.input(&utils::big_int_to_bytes(&key_public_b));
        hasher.input(&key_k);
        key_m = hex::decode(&hasher.result_str()).unwrap();
    } else if plugin_name == "Srp256" {
        let mut hasher = Sha256::new();
        hasher.input(&utils::big_int_to_bytes(&n3));
        hasher.input(&utils::big_int_to_bytes(&n4));
        hasher.input(salt);
        hasher.input(&utils::big_int_to_bytes(&key_public_a));
        hasher.input(&utils::big_int_to_bytes(&key_public_b));
        hasher.input(&key_k);
        key_m = hex::decode(&hasher.result_str()).unwrap();
    } else {
        panic!("srp protocol error");
    }

    (key_m, key_k) // M, K
}

#[test]
fn test_srp() {
    let user = "SYSDBA";
    let password = "masterkey";

    // Client send A to Server
    let (key_public_a, key_private_a) = get_client_seed();
    println!("a={}", &key_private_a);
    println!("A={}", &key_public_a);

    // Server send B, salt to Client
    let salt = get_salt();
    println!("salt={}", utils::bytes_to_big_int(&salt));
    let v = get_verifier(&user, &password, &salt);
    let (key_public_b, key_private_b) = get_server_seed(&v);
    println!("b={}", &key_private_b);
    println!("B={}", &key_public_b);

    let server_key = get_server_session(
        &user,
        &password,
        &salt,
        &key_public_a,
        &key_public_b,
        &key_private_b,
    );
    println!("server_key={}", utils::bytes_to_big_int(&server_key));

    let (_, client_key) = get_client_proof(
        &user,
        &password,
        &salt,
        &key_public_a,
        &key_public_b,
        &key_private_a,
        "Srp",
    );
    println!("Srp client_key={}", utils::bytes_to_big_int(&client_key));
    assert_eq!(&server_key, &client_key);

    let (_, client_key) = get_client_proof(
        &user,
        &password,
        &salt,
        &key_public_a,
        &key_public_b,
        &key_private_a,
        "Srp256",
    );
    println!("Srp256 client_key={}", utils::bytes_to_big_int(&client_key));
    assert_eq!(&server_key, &client_key);
}
