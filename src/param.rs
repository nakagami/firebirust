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

use super::*;
use chrono;
use chrono::{Datelike, Timelike};
use chrono_tz;
use rust_decimal::Decimal;
use std::io::prelude::*;

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

pub trait ToSqlParam {
    fn to_value_blr_isnull(&self) -> (Vec<u8>, Vec<u8>, bool);
}

impl ToSqlParam for Param {
    fn to_value_blr_isnull(&self) -> (Vec<u8>, Vec<u8>, bool) {
        let mut value: Vec<u8> = Vec::new();
        let mut blr: Vec<u8> = Vec::new();
        let mut isnull = false;
        match self {
            Param::Null => {
                blr.write(&[14, 0, 0]).unwrap();
                isnull = true;
            }
            Param::Text(s) => {
                let b = s.as_bytes();
                let (b, v) = utils::bytes_to_blr(b);
                value.write(&v).unwrap();
                blr.write(&b).unwrap();
            }
            Param::Short(n) => {
                value.write(&utils::bint32_to_bytes(*n as i32)).unwrap();
                blr.write(&[8, 0]).unwrap();
            }
            Param::Long(n) => {
                value.write(&utils::bint32_to_bytes(*n)).unwrap();
                blr.write(&[8, 0]).unwrap();
            }
            Param::Int64(n) => {
                value.write(&utils::bint64_to_bytes(*n)).unwrap();
                blr.write(&[16, 0]).unwrap();
            }
            Param::Int128(n) => {
                value.write(&utils::bint128_to_bytes(*n)).unwrap();
                blr.write(&[26, 0]).unwrap();
            }
            Param::Time(t) => {
                value
                    .write(&utils::convert_time(
                        t.hour(),
                        t.minute(),
                        t.second(),
                        t.nanosecond(),
                    ))
                    .unwrap();
                blr.write(&[13]).unwrap();
            }
            Param::Date(d) => {
                value
                    .write(&utils::convert_date(d.year(), d.month(), d.day()))
                    .unwrap();
                blr.write(&[12]).unwrap();
            }
            Param::TimeStamp(dt) => {
                let d = dt.date();
                let t = dt.time();
                value
                    .write(&utils::convert_date(d.year(), d.month(), d.day()))
                    .unwrap();
                value
                    .write(&utils::convert_time(
                        t.hour() as u32,
                        t.minute(),
                        t.second(),
                        t.nanosecond(),
                    ))
                    .unwrap();
                blr.write(&[35]).unwrap();
            }
            Param::Float(f) => {
                value.write(&utils::f32_to_bytes(*f)).unwrap();
                blr.write(&[10]).unwrap();
            }
            Param::Double(d) => {
                value.write(&utils::f64_to_bytes(*d)).unwrap();
                blr.write(&[27]).unwrap();
            }
            Param::Blob(b) => {
                let (b, v) = utils::bytes_to_blr(b);
                value.write(&v).unwrap();
                blr.write(&b).unwrap();
            }
            Param::TimeStampTZ(_dt_tz) => {
                // TODO:
            }
            Param::Decimal(d) => {
                let s = d.to_string();
                let b = s.as_bytes();
                let (b, v) = utils::bytes_to_blr(b);
                value.write(&v).unwrap();
                blr.write(&b).unwrap();
            }
            Param::Boolean(b) => {
                if *b {
                    value.write(&[1, 0, 0, 0]).unwrap();
                } else {
                    value.write(&[0, 0, 0, 0]).unwrap();
                }
                blr.write(&[23]).unwrap();
            }
        }
        (value, blr, isnull)
    }
}

macro_rules! to_sql_param(
    ($t:ty) => (
        impl ToSqlParam for $t {
            #[inline]
            fn to_value_blr_isnull(&self) -> (Vec<u8>, Vec<u8>, bool) {
                Param::from(*self).to_value_blr_isnull()
            }
        }
    )
);

to_sql_param!(&str);
to_sql_param!(i16);
to_sql_param!(i32);
to_sql_param!(i64);
to_sql_param!(i128);
to_sql_param!(chrono::NaiveTime);
to_sql_param!(chrono::NaiveDate);
to_sql_param!(f32);
to_sql_param!(f64);
to_sql_param!(&[u8]);
to_sql_param!(chrono::DateTime<chrono_tz::Tz>);
to_sql_param!(Decimal);
to_sql_param!(bool);
