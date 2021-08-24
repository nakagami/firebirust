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

use std::collections::HashMap;

use super::error::UrlError;
use percent_encoding::percent_decode;
use url::Url;

#[derive(Debug)]
pub struct ConnParams {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub db_name: String,
}

impl ConnParams {
    pub fn from_url(s: &str) -> Result<(ConnParams, HashMap<String, String>), UrlError> {
        let url = Url::parse(s)?;
        if url.scheme() != "firebird" {
            return Err(UrlError::UnsupportedScheme(url.scheme().to_string()));
        }
        if url.cannot_be_a_base() {
            return Err(UrlError::BadUrl);
        }

        let host = url.host().unwrap().to_string();
        let port = url.port().unwrap_or(3050);
        let username = percent_decode(url.username().as_ref())
            .decode_utf8_lossy()
            .into_owned();
        let password = percent_decode(url.password().unwrap().as_ref())
            .decode_utf8_lossy()
            .into_owned();
        let mut db_name = percent_decode(url.path().as_ref())
            .decode_utf8_lossy()
            .into_owned();
        let mut slash_count = 0;
        for b in db_name.as_bytes() {
            if b == &b'/' {
                slash_count += 1;
            }
        }
        if slash_count == 1 {
            // "/foo.fdb" -> "foo.fdb"
            db_name = db_name[1..].to_string();
        }

        let mut options: HashMap<String, String> = url.query_pairs().into_owned().collect();
        options
            .entry(String::from("role"))
            .or_insert("".to_string());
        options
            .entry(String::from("timezone"))
            .or_insert("".to_string());
        options
            .entry(String::from("wire_crypt"))
            .or_insert("true".to_string());
        options
            .entry(String::from("auth_plugin_name"))
            .or_insert("Srp256".to_string());
        options
            .entry(String::from("page_size"))
            .or_insert("4096".to_string());
        Ok((
            ConnParams {
                host,
                port,
                username,
                password,
                db_name,
            },
            options,
        ))
    }
}

#[test]
fn test_conn_params() {
    let (params, _options) =
        ConnParams::from_url("firebird://user:pass%20word@localhost:3051/foo/bar.fdb").unwrap();
    assert_eq!(&params.host, "localhost");
    assert_eq!(params.port, 3051u16);
    assert_eq!(&params.username, "user");
    assert_eq!(&params.password, "pass word");
    assert_eq!(&params.db_name, "/foo/bar.fdb");

    let (params, _options) = ConnParams::from_url("firebird://user:pass@foo/bar.fdb").unwrap();

    assert_eq!(&params.host, "foo");
    assert_eq!(params.port, 3050u16);
    assert_eq!(&params.username, "user");
    assert_eq!(&params.password, "pass");
    assert_eq!(&params.db_name, "bar.fdb"); // leading slash omited

    let (_params, options) =
        ConnParams::from_url("firebird://user:pass@localhost/foo/bar.fdb?page_size=1234").unwrap();
    assert_eq!(&options["page_size"], "1234");
}
