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

use super::error::{Error, ValueError};
use super::params;
use std::result::Result;

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

impl Param {
    pub fn get_i8(&self) -> Result<Option<i8>, Error> {
        match self {
            Param::Null => Ok(None),
            _ => Err(Error::ValueError(ValueError::new("Can't get_i8()"))),
        }
    }

    pub fn get_i16(&self) -> Result<Option<i16>, Error> {
        match self {
            Param::Null => Ok(None),
            Param::Short(v) => Ok(Some(*v)),
            _ => Err(Error::ValueError(ValueError::new("Can't get_i16()"))),
        }
    }

    pub fn get_i32(&self) -> Result<Option<i32>, Error> {
        match self {
            Param::Null => Ok(None),
            Param::Long(v) => Ok(Some(*v)),
            _ => Err(Error::ValueError(ValueError::new("Can't get_i32()"))),
        }
    }

    pub fn get_i64(&self) -> Result<Option<i64>, Error> {
        match self {
            Param::Null => Ok(None),
            Param::Int64(v) => Ok(Some(*v)),
            _ => Err(Error::ValueError(ValueError::new("Can't get_i64()"))),
        }
    }

    pub fn get_f32(&self) -> Result<Option<f32>, Error> {
        match self {
            Param::Null => Ok(None),
            Param::Float(v) => Ok(Some(*v)),
            _ => Err(Error::ValueError(ValueError::new("Can't get_f32()"))),
        }
    }

    pub fn get_f64(&self) -> Result<Option<f64>, Error> {
        match self {
            Param::Null => Ok(None),
            Param::Double(v) => Ok(Some(*v)),
            _ => Err(Error::ValueError(ValueError::new("Can't get_f64()"))),
        }
    }

    pub fn get_string(&self) -> Result<Option<String>, Error> {
        match self {
            Param::Null => Ok(None),
            Param::Text(v) => Ok(Some(v.to_string())),
            _ => Err(Error::ValueError(ValueError::new("Can't get_string()"))),
        }
    }

    pub fn get_bytes(&self) -> Result<Option<Vec<u8>>, Error> {
        match self {
            Param::Null => Ok(None),
            Param::Blob(v) => {
                let mut blob: Vec<u8> = Vec::new();
                Ok(Some(blob))
            }
            _ => Err(Error::ValueError(ValueError::new("Can't get_bytes()"))),
        }
    }
}

impl From<i32> for Param {
    fn from(v: i32) -> Param {
        Param::Long(v)
    }
}

impl From<&str> for Param {
    fn from(v: &str) -> Param {
        Param::Text(v.to_string())
    }
}

#[test]
fn test_params() {
    let params = vec![Param::from(1i32), Param::from("foo"), Param::Null];
    assert_eq!(params, params![1i32, "foo", Param::Null]);
    assert_eq!(params[0].get_i32().unwrap(), Some(1));
    assert_eq!(params[1].get_string().unwrap(), Some("foo".to_string()));
    assert_eq!(params[2].get_string().unwrap(), None);
}
