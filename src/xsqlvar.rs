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

use super::error::ValueError;
use super::utils::*;
use super::*;
use maplit::hashmap;

const SQL_TYPE_TEXT: u32 = 452;
const SQL_TYPE_VARYING: u32 = 448;
const SQL_TYPE_SHORT: u32 = 500;
const SQL_TYPE_LONG: u32 = 496;
const SQL_TYPE_FLOAT: u32 = 482;
const SQL_TYPE_DOUBLE: u32 = 480;
const SQL_TYPE_D_FLOAT: u32 = 530;
const SQL_TYPE_TIMESTAMP: u32 = 510;
const SQL_TYPE_BLOB: u32 = 520;
const SQL_TYPE_ARRAY: u32 = 540;
const SQL_TYPE_QUAD: u32 = 550;
const SQL_TYPE_TIME: u32 = 560;
const SQL_TYPE_DATE: u32 = 570;
const SQL_TYPE_INT64: u32 = 580;
const SQL_TYPE_INT128: u32 = 32752;
const SQL_TYPE_TIMESTAMP_TZ: u32 = 32754;
const SQL_TYPE_TIME_TZ: u32 = 32756;
const SQL_TYPE_DEC_FIXED: u32 = 32758;
const SQL_TYPE_DEC64: u32 = 32760;
const SQL_TYPE_DEC128: u32 = 32762;
const SQL_TYPE_BOOLEAN: u32 = 32764;
const SQL_TYPE_NULL: u32 = 32766;

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
                SQL_TYPE_SHORT=>        6,
                SQL_TYPE_LONG=>         11,
                SQL_TYPE_FLOAT=>        17,
                SQL_TYPE_TIME=>         11,
                SQL_TYPE_DATE=>         10,
                SQL_TYPE_DOUBLE=>       17,
                SQL_TYPE_TIMESTAMP=>    22,
                SQL_TYPE_BLOB=>         0,
                SQL_TYPE_ARRAY=>        -1,
                SQL_TYPE_QUAD=>         20,
                SQL_TYPE_INT64=>        20,
                SQL_TYPE_INT128=>       20,
                SQL_TYPE_TIMESTAMP_TZ=> 28,
                SQL_TYPE_TIME_TZ=>      17,
                SQL_TYPE_DEC64=>        16,
                SQL_TYPE_DEC128=>       34,
                SQL_TYPE_DEC_FIXED=>    34,
                SQL_TYPE_BOOLEAN=>      5,
            };
            map[&(self.sqllen as u32)]
        }
    }

    pub fn value(&self, raw_value: &[u8]) -> Result<Value, ValueError> {
        match (self.sqltype) {
            SQL_TYPE_TEXT => Ok(Value::Text(bytes_to_str(raw_value))),
            SQL_TYPE_VARYING => Ok(Value::Varying(bytes_to_str(raw_value))),
            SQL_TYPE_SHORT => Ok(Value::Short(bytes_to_int16(raw_value))),
            SQL_TYPE_LONG => Ok(Value::Long(bytes_to_int32(raw_value))),
            SQL_TYPE_FLOAT => Ok(Value::Float(bytes_to_f32(raw_value))),
            SQL_TYPE_DOUBLE => Ok(Value::Double(bytes_to_f64(raw_value))),
            // TODO:
            _ => Err(ValueError::new("can't parse result value")),
        }
    }
}
