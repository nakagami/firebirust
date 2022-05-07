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

use chrono;
use chrono_tz;
use rust_decimal::Decimal;

#[derive(PartialEq, Debug, Clone)]
pub enum Param {
    Null,
    Text(String),
    Short(i16),
    Long(i32),
    Int64(i64),
    Int128(i128),
    Time(chrono::NaiveTime),
    Date(chrono::NaiveDate),
    TimeStamp(chrono::NaiveDateTime),
    Float(f32),
    Double(f64),
    Blob(Vec<u8>),
    TimeStampTZ(chrono::DateTime<chrono_tz::Tz>),
    // TimeTz(??),
    Decimal(Decimal),
    Boolean(bool),
}

impl From<&str> for Param {
    fn from(v: &str) -> Param {
        Param::Text(v.to_string())
    }
}

impl From<i16> for Param {
    fn from(v: i16) -> Param {
        Param::Short(v)
    }
}

impl From<i32> for Param {
    fn from(v: i32) -> Param {
        Param::Long(v)
    }
}

impl From<i64> for Param {
    fn from(v: i64) -> Param {
        Param::Int64(v)
    }
}

impl From<i128> for Param {
    fn from(v: i128) -> Param {
        Param::Int128(v)
    }
}

impl From<chrono::NaiveTime> for Param {
    fn from(v: chrono::NaiveTime) -> Param {
        Param::Time(v)
    }
}

impl From<chrono::NaiveDate> for Param {
    fn from(v: chrono::NaiveDate) -> Param {
        Param::Date(v)
    }
}

impl From<chrono::NaiveDateTime> for Param {
    fn from(v: chrono::NaiveDateTime) -> Param {
        Param::TimeStamp(v)
    }
}

impl From<f32> for Param {
    fn from(v: f32) -> Param {
        Param::Float(v)
    }
}

impl From<f64> for Param {
    fn from(v: f64) -> Param {
        Param::Double(v)
    }
}

impl From<&[u8]> for Param {
    fn from(v: &[u8]) -> Param {
        Param::Blob(Vec::from(v))
    }
}

impl From<chrono::DateTime<chrono_tz::Tz>> for Param {
    fn from(v: chrono::DateTime<chrono_tz::Tz>) -> Param {
        Param::TimeStampTZ(v)
    }
}

impl From<Decimal> for Param {
    fn from(v: Decimal) -> Param {
        Param::Decimal(v)
    }
}

impl From<bool> for Param {
    fn from(v: bool) -> Param {
        Param::Boolean(v)
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
