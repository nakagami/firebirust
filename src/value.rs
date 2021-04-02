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
use super::*;
use std::result::Result;

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    Null,
    Text(String),
    Varying(String),
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

impl Value {
    pub fn get_i8(&self) -> Result<Option<i8>, Error> {
        match (self) {
            Value::Null => Ok(None),
            _ => Err(Error::ValueError(ValueError::new("Can't get_i8()"))),
        }
    }

    pub fn get_i16(&self) -> Result<Option<i16>, Error> {
        match (self) {
            Value::Null => Ok(None),
            Value::Short(v) => Ok(Some(*v)),
            _ => Err(Error::ValueError(ValueError::new("Can't get_i16()"))),
        }
    }

    pub fn get_i32(&self) -> Result<Option<i32>, Error> {
        match (self) {
            Value::Null => Ok(None),
            Value::Long(v) => Ok(Some(*v)),
            _ => Err(Error::ValueError(ValueError::new("Can't get_i32()"))),
        }
    }

    pub fn get_i64(&self) -> Result<Option<i64>, Error> {
        match (self) {
            Value::Null => Ok(None),
            Value::Int64(v) => Ok(Some(*v)),
            _ => Err(Error::ValueError(ValueError::new("Can't get_i64()"))),
        }
    }

    pub fn get_f32(&self) -> Result<Option<f32>, Error> {
        match (self) {
            Value::Null => Ok(None),
            Value::Float(v) => Ok(Some(*v)),
            _ => Err(Error::ValueError(ValueError::new("Can't get_f32()"))),
        }
    }

    pub fn get_f64(&self) -> Result<Option<f64>, Error> {
        match (self) {
            Value::Null => Ok(None),
            Value::Double(v) => Ok(Some(*v)),
            _ => Err(Error::ValueError(ValueError::new("Can't get_f64()"))),
        }
    }

    pub fn get_string(&self) -> Result<Option<String>, Error> {
        match (self) {
            Value::Null => Ok(None),
            Value::Text(v) | Value::Varying(v) => Ok(Some(v.to_string())),
            _ => Err(Error::ValueError(ValueError::new("Can't get_string()"))),
        }
    }

    pub fn get_bytes(&self) -> Result<Option<Vec<u8>>, Error> {
        match (self) {
            Value::Null => Ok(None),
            Value::Blob(v) => {
                let mut blob: Vec<u8> = Vec::new();
                Ok(Some(blob))
            }
            _ => Err(Error::ValueError(ValueError::new("Can't get_bytes()"))),
        }
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Value {
        Value::Long(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Value {
        Value::Text(v.to_string())
    }
}
