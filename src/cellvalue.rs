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
use chrono;
use chrono_tz;
use rust_decimal;
use std::result::Result;

#[derive(PartialEq, Debug, Clone)]
pub enum CellValue {
    Null,
    Text(String),
    Varying(String),
    Short(i16),
    Long(i32),
    Float(f32),
    Time(chrono::NaiveTime),
    Date(chrono::NaiveDate),
    Double(f64),
    TimeStamp(chrono::NaiveDateTime),
    BlobBinary(Vec<u8>),
    BlobText(Vec<u8>),
    Int64(i64),
    Int128(i128),
    TimeStampTz(chrono::DateTime<chrono_tz::Tz>),
    TimeTz((chrono::NaiveTime, chrono_tz::Tz)),
    Decimal(rust_decimal::Decimal),
    Boolean(bool),
}

pub trait CellValueToVal<T> {
    fn to_val(self) -> Result<T, Error>
    where
        Self: std::marker::Sized;
}

impl<T> CellValueToVal<Option<T>> for CellValue
where
    CellValue: CellValueToVal<T>,
{
    fn to_val(self) -> Result<Option<T>, Error> {
        match self {
            CellValue::Null => Ok(None),
            _ => Ok(Some(self.to_val()?)),
        }
    }
}

impl CellValueToVal<String> for CellValue {
    fn to_val(self) -> Result<String, Error> {
        match self {
            CellValue::Text(v) => Ok(v.to_string()),
            CellValue::Varying(v) => Ok(v.to_string()),
            CellValue::BlobBinary(v) => Ok(String::from_utf8_lossy(&v).to_string()),
            CellValue::BlobText(v) => Ok(String::from_utf8_lossy(&v).to_string()),
            _ => Err(Error::ValueError(ValueError::new("Can't convert string"))),
        }
    }
}

impl CellValueToVal<i64> for CellValue {
    fn to_val(self) -> Result<i64, Error> {
        match self {
            CellValue::Short(v) => Ok(v.into()),
            CellValue::Long(v) => Ok(v.into()),
            CellValue::Int64(v) => Ok(v.into()),
            _ => Err(Error::ValueError(ValueError::new("Can't convert int"))),
        }
    }
}

impl CellValueToVal<i32> for CellValue {
    fn to_val(self) -> Result<i32, Error> {
        CellValueToVal::<i64>::to_val(self).map(|i| i as i32)
    }
}

impl CellValueToVal<i16> for CellValue {
    fn to_val(self) -> Result<i16, Error> {
        CellValueToVal::<i64>::to_val(self).map(|i| i as i16)
    }
}

impl CellValueToVal<f64> for CellValue {
    fn to_val(self) -> Result<f64, Error> {
        match self {
            CellValue::Double(v) => Ok(v),
            _ => Err(Error::ValueError(ValueError::new("Can't convert double"))),
        }
    }
}

impl CellValueToVal<f32> for CellValue {
    fn to_val(self) -> Result<f32, Error> {
        match self {
            CellValue::Float(v) => Ok(v),
            _ => Err(Error::ValueError(ValueError::new("Can't convert float"))),
        }
    }
}

impl CellValueToVal<Vec<u8>> for CellValue {
    fn to_val(self) -> Result<Vec<u8>, Error> {
        match self {
            CellValue::BlobBinary(v) => Ok(v.clone()),
            CellValue::BlobText(v) => Ok(v.clone()),
            _ => Err(Error::ValueError(ValueError::new("Can't convert binary"))),
        }
    }
}

impl CellValueToVal<bool> for CellValue {
    fn to_val(self) -> Result<bool, Error> {
        match self {
            CellValue::Boolean(v) => Ok(v),
            _ => Err(Error::ValueError(ValueError::new("Can't convert bool"))),
        }
    }
}

impl CellValueToVal<chrono::NaiveTime> for CellValue {
    fn to_val(self) -> Result<chrono::NaiveTime, Error> {
        match self {
            CellValue::Time(v) => Ok(v),
            _ => Err(Error::ValueError(ValueError::new("Can't convert time"))),
        }
    }
}

impl CellValueToVal<chrono::NaiveDate> for CellValue {
    fn to_val(self) -> Result<chrono::NaiveDate, Error> {
        match self {
            CellValue::Date(v) => Ok(v),
            _ => Err(Error::ValueError(ValueError::new("Can't convert date"))),
        }
    }
}

impl CellValueToVal<chrono::NaiveDateTime> for CellValue {
    fn to_val(self) -> Result<chrono::NaiveDateTime, Error> {
        match self {
            CellValue::TimeStamp(v) => Ok(v),
            _ => Err(Error::ValueError(ValueError::new(
                "Can't convert timestamp",
            ))),
        }
    }
}

impl CellValueToVal<rust_decimal::Decimal> for CellValue {
    fn to_val(self) -> Result<rust_decimal::Decimal, Error> {
        match self {
            CellValue::Decimal(v) => Ok(v),
            _ => Err(Error::ValueError(ValueError::new("Can't convert decimal"))),
        }
    }
}
