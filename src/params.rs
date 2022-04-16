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

#[derive(PartialEq, Debug, Clone)]
pub enum Param {
    Null,
    Text(String),
    Short(i16),
    Long(i32),
    Float(f32),
    //  Time(time),
    //  Date(date),
    Double(f64),
    //    TimeStamp(??),
    Blob(Vec<u8>),
    Int64(i64),
    Int128(i128),
    //    TimeStampTZ(??),
    //    TimeTz(??),
    //  Decimal(??),
    Boolean(bool),
}

impl From<&str> for Param {
    fn from(v: &str) -> Param {
        Param::Text(v.to_string())
    }
}

impl From<i32> for Param {
    fn from(v: i32) -> Param {
        Param::Long(v)
    }
}

#[test]
fn test_params() {
    use super::params;
    let params = vec![Param::from(1i32), Param::from("foo"), Param::Null];
    assert_eq!(params, params![1i32, "foo", Param::Null]);
    assert_eq!(params[0], Param::Long(1));
    assert_eq!(params[1], Param::Text("foo".to_string()));
    assert_eq!(params[2], Param::Null);
}
