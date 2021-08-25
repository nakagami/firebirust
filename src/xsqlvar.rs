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

#![allow(dead_code)]
use super::cellvalue::CellValue;
use super::decfloat;
use super::error::ValueError;
use super::utils::*;
use maplit::hashmap;
use rust_decimal;

pub const SQL_TYPE_TEXT: u32 = 452;
pub const SQL_TYPE_VARYING: u32 = 448;
pub const SQL_TYPE_SHORT: u32 = 500;
pub const SQL_TYPE_LONG: u32 = 496;
pub const SQL_TYPE_FLOAT: u32 = 482;
pub const SQL_TYPE_DOUBLE: u32 = 480;
pub const SQL_TYPE_D_FLOAT: u32 = 530;
pub const SQL_TYPE_TIMESTAMP: u32 = 510;
pub const SQL_TYPE_BLOB: u32 = 520;
pub const SQL_TYPE_ARRAY: u32 = 540;
pub const SQL_TYPE_QUAD: u32 = 550;
pub const SQL_TYPE_TIME: u32 = 560;
pub const SQL_TYPE_DATE: u32 = 570;
pub const SQL_TYPE_INT64: u32 = 580;
pub const SQL_TYPE_INT128: u32 = 32752;
pub const SQL_TYPE_TIMESTAMP_TZ: u32 = 32754;
pub const SQL_TYPE_TIME_TZ: u32 = 32756;
pub const SQL_TYPE_DEC_FIXED: u32 = 32758;
pub const SQL_TYPE_DEC64: u32 = 32760;
pub const SQL_TYPE_DEC128: u32 = 32762;
pub const SQL_TYPE_BOOLEAN: u32 = 32764;
pub const SQL_TYPE_NULL: u32 = 32766;

pub struct XSQLVar {
    pub sqltype: u32,
    pub sqlscale: i32,
    pub sqlsubtype: i32,
    pub sqllen: i32,
    pub null_ok: bool,
    pub fieldname: String,
    pub relname: String,
    pub ownname: String,
    pub aliasname: String,
}

impl XSQLVar {
    pub fn new() -> XSQLVar {
        XSQLVar {
            sqltype: 0,
            sqlscale: 0,
            sqlsubtype: 0,
            sqllen: 0,
            null_ok: false,
            fieldname: "".to_string(),
            relname: "".to_string(),
            ownname: "".to_string(),
            aliasname: "".to_string(),
        }
    }
    pub fn io_length(&self) -> isize {
        if self.sqltype == SQL_TYPE_TEXT {
            self.sqllen as isize
        } else {
            let map = hashmap! {
                SQL_TYPE_TEXT=>         -1,
                SQL_TYPE_VARYING=>      -1,
                SQL_TYPE_SHORT=>        4,
                SQL_TYPE_LONG=>         4,
                SQL_TYPE_FLOAT=>        4,
                SQL_TYPE_TIME=>         4,
                SQL_TYPE_DATE=>         4,
                SQL_TYPE_DOUBLE=>       8,
                SQL_TYPE_TIMESTAMP=>    8,
                SQL_TYPE_BLOB=>         8,
                SQL_TYPE_ARRAY=>        8,
                SQL_TYPE_QUAD=>         8,
                SQL_TYPE_INT64=>        8,
                SQL_TYPE_INT128=>       16,
                SQL_TYPE_TIMESTAMP_TZ=> 10,
                SQL_TYPE_TIME_TZ=>      6,
                SQL_TYPE_DEC64=>        8,
                SQL_TYPE_DEC128=>       16,
                SQL_TYPE_DEC_FIXED=>    16,
                SQL_TYPE_BOOLEAN=>      1,
            };
            map[&self.sqltype]
        }
    }

    pub fn value(&self, raw_value: &[u8]) -> Result<CellValue, ValueError> {
        match self.sqltype {
            SQL_TYPE_TEXT => Ok(CellValue::Text(bytes_to_str(raw_value))),
            SQL_TYPE_VARYING => Ok(CellValue::Varying(bytes_to_str(raw_value))),
            SQL_TYPE_SHORT => Ok(CellValue::Short(bytes_to_bint16(raw_value))),
            SQL_TYPE_LONG => Ok(CellValue::Long(bytes_to_bint32(raw_value))),
            SQL_TYPE_INT64 => Ok(if self.sqlscale < 0 {
                CellValue::Decimal(rust_decimal::Decimal::new(
                    bytes_to_bint64(raw_value),
                    (self.sqlscale * -1) as u32,
                ))
            } else if self.sqlscale > 0 {
                CellValue::Decimal(rust_decimal::Decimal::new(
                    bytes_to_bint64(raw_value) * (self.sqlscale as i64),
                    0,
                ))
            } else {
                CellValue::Int64(bytes_to_bint64(raw_value))
            }),
            SQL_TYPE_INT128 => Ok(if self.sqlscale < 0 {
                CellValue::Decimal(rust_decimal::Decimal::new(
                    bytes_to_bint64(raw_value),
                    (self.sqlscale * -1) as u32,
                ))
            } else if self.sqlscale > 0 {
                CellValue::Decimal(rust_decimal::Decimal::new(
                    (bytes_to_bint128(raw_value) as i64) * (self.sqlscale as i64),
                    0,
                ))
            } else {
                CellValue::Int128(bytes_to_bint128(raw_value))
            }),
            SQL_TYPE_DATE => Ok(CellValue::Date(bytes_to_naive_date(raw_value))),
            SQL_TYPE_TIME => Ok(CellValue::Time(bytes_to_naive_time(raw_value))),
            SQL_TYPE_TIMESTAMP => Ok(CellValue::TimeStamp(bytes_to_naive_date_time(raw_value))),
            // SQL_TYPE_TIME_TZ => Ok(CellValue::Time(bytes_to_naive_time(raw_value))),
            // SQL_TYPE_TIMESTAMP_TZ => Ok(CellValue::TimeStamp(bytes_to_naive_date_time(raw_value))),
            SQL_TYPE_FLOAT => Ok(CellValue::Float(bytes_to_f32(raw_value))),
            SQL_TYPE_DOUBLE => Ok(CellValue::Double(bytes_to_f64(raw_value))),
            SQL_TYPE_BOOLEAN => Ok(CellValue::Boolean(raw_value[0] != 0)),
            SQL_TYPE_BLOB => Ok(if self.sqlsubtype == 1 {
                CellValue::BlobText(raw_value.to_vec())
            } else {
                CellValue::BlobBinary(raw_value.to_vec())
            }),
            SQL_TYPE_DEC_FIXED => Ok(CellValue::Decimal(decfloat::decimal_fixed_to_decimal(
                raw_value,
                self.sqlscale,
            )?)),
            SQL_TYPE_DEC64 => Ok(CellValue::Decimal(decfloat::decimal64_to_decimal(
                raw_value,
            )?)),
            SQL_TYPE_DEC128 => Ok(CellValue::Decimal(decfloat::decimal128_to_decimal(
                raw_value,
            )?)),
            _ => Err(ValueError::new(&format!(
                "can't parse result value:{}",
                self.sqltype
            ))),
        }
    }
}
