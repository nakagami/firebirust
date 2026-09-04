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
use super::*;
use maplit::hashmap;
use rust_decimal;

pub(crate) struct XSQLVar {
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
                SQL_TYPE_TIMESTAMP_TZ=> 12,
                SQL_TYPE_TIME_TZ=>      8,
                SQL_TYPE_DEC64=>        8,
                SQL_TYPE_DEC128=>       16,
                SQL_TYPE_DEC_FIXED=>    16,
                SQL_TYPE_BOOLEAN=>      1,
                SQL_TYPE_NULL=>         0,
            };
            map[&self.sqltype]
        }
    }

    pub fn name(&self) -> &str {
        let map = hashmap! {
            SQL_TYPE_TEXT=>         "TEXT",
            SQL_TYPE_VARYING=>      "VARCHR",
            SQL_TYPE_SHORT=>        "SHORT",
            SQL_TYPE_LONG=>         "LONG",
            SQL_TYPE_FLOAT=>        "FLOAT",
            SQL_TYPE_TIME=>         "TIME",
            SQL_TYPE_DATE=>         "DATE",
            SQL_TYPE_DOUBLE=>       "DOUBLE",
            SQL_TYPE_TIMESTAMP=>    "TIMESTAMP",
            SQL_TYPE_BLOB=>         "BLOB",
            SQL_TYPE_ARRAY=>        "ARRAY",
            SQL_TYPE_QUAD=>         "QUAD",
            SQL_TYPE_INT64=>        "INT64",
            SQL_TYPE_INT128=>       "INT128",
            SQL_TYPE_TIMESTAMP_TZ=> "TIMESTAMP WITH TIMEZONE",
            SQL_TYPE_TIME_TZ=>      "TIME WITH TIMEZONE",
            SQL_TYPE_DEC64=>        "DEC64",
            SQL_TYPE_DEC128=>       "DEC128",
            SQL_TYPE_DEC_FIXED=>    "DEC_FIXED",
            SQL_TYPE_BOOLEAN=>      "BOOLEAN",
            SQL_TYPE_NULL=>         "NULL",
        };
        map[&self.sqltype]
    }

    pub fn value(&self, raw_value: &[u8]) -> Result<CellValue, ValueError> {
        match self.sqltype {
            SQL_TYPE_TEXT => Ok(CellValue::Text(utils::bytes_to_rtrim_str(raw_value))),
            SQL_TYPE_VARYING => Ok(CellValue::Varying(utils::bytes_to_str(raw_value))),
            SQL_TYPE_SHORT => {
                let val = utils::bytes_to_bint32(raw_value) as i16;
                if self.sqlscale < 0 {
                    Ok(CellValue::Decimal(rust_decimal::Decimal::new(
                        val as i64,
                        (self.sqlscale * -1) as u32,
                    )))
                } else if self.sqlscale > 0 {
                    let mul = 10i64
                        .checked_pow(self.sqlscale as u32)
                        .ok_or_else(|| ValueError::new("sqlscale out of range"))?;
                    Ok(CellValue::Decimal(rust_decimal::Decimal::new(
                        (val as i64)
                            .checked_mul(mul)
                            .ok_or_else(|| ValueError::new("numeric multiplication overflow"))?,
                        0,
                    )))
                } else {
                    Ok(CellValue::Short(val))
                }
            }
            SQL_TYPE_LONG => {
                let val = utils::bytes_to_bint32(raw_value);
                if self.sqlscale < 0 {
                    Ok(CellValue::Decimal(rust_decimal::Decimal::new(
                        val as i64,
                        (self.sqlscale * -1) as u32,
                    )))
                } else if self.sqlscale > 0 {
                    let mul = 10i64
                        .checked_pow(self.sqlscale as u32)
                        .ok_or_else(|| ValueError::new("sqlscale out of range"))?;
                    Ok(CellValue::Decimal(rust_decimal::Decimal::new(
                        (val as i64)
                            .checked_mul(mul)
                            .ok_or_else(|| ValueError::new("numeric multiplication overflow"))?,
                        0,
                    )))
                } else {
                    Ok(CellValue::Long(val))
                }
            }
            SQL_TYPE_INT64 => {
                let val = utils::bytes_to_bint64(raw_value);
                if self.sqlscale < 0 {
                    Ok(CellValue::Decimal(rust_decimal::Decimal::new(
                        val,
                        (self.sqlscale * -1) as u32,
                    )))
                } else if self.sqlscale > 0 {
                    let mul = 10i64
                        .checked_pow(self.sqlscale as u32)
                        .ok_or_else(|| ValueError::new("sqlscale out of range"))?;
                    Ok(CellValue::Decimal(rust_decimal::Decimal::new(
                        val.checked_mul(mul)
                            .ok_or_else(|| ValueError::new("numeric multiplication overflow"))?,
                        0,
                    )))
                } else {
                    Ok(CellValue::Int64(val))
                }
            }
            SQL_TYPE_INT128 => {
                let val = utils::bytes_to_bint128(raw_value);
                if self.sqlscale < 0 {
                    Ok(CellValue::Decimal(
                        rust_decimal::Decimal::try_from_i128_with_scale(
                            val,
                            (self.sqlscale * -1) as u32,
                        )
                        .map_err(|e| ValueError::new(&e.to_string()))?,
                    ))
                } else if self.sqlscale > 0 {
                    let mul = 10i128
                        .checked_pow(self.sqlscale as u32)
                        .ok_or_else(|| ValueError::new("sqlscale out of range"))?;
                    let scaled_val = val
                        .checked_mul(mul)
                        .ok_or_else(|| ValueError::new("numeric multiplication overflow"))?;
                    Ok(CellValue::Decimal(
                        rust_decimal::Decimal::try_from_i128_with_scale(scaled_val, 0)
                            .map_err(|e| ValueError::new(&e.to_string()))?,
                    ))
                } else {
                    Ok(CellValue::Int128(val))
                }
            }
            SQL_TYPE_NULL => Ok(CellValue::Null),
            SQL_TYPE_DATE => Ok(CellValue::Date(utils::bytes_to_naive_date(raw_value))),
            SQL_TYPE_TIME => Ok(CellValue::Time(utils::bytes_to_naive_time(raw_value))),
            SQL_TYPE_TIMESTAMP => Ok(CellValue::TimeStamp(utils::bytes_to_naive_date_time(
                raw_value,
            ))),
            SQL_TYPE_TIME_TZ => Ok(CellValue::TimeTz(utils::bytes_to_time_tz(raw_value))),
            SQL_TYPE_TIMESTAMP_TZ => Ok(CellValue::TimeStampTz(utils::bytes_to_date_time_tz(
                raw_value,
            ))),
            SQL_TYPE_FLOAT => Ok(CellValue::Float(utils::bytes_to_f32(raw_value))),
            SQL_TYPE_DOUBLE => Ok(CellValue::Double(utils::bytes_to_f64(raw_value))),
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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_xsqlvar_null() {
        let mut x = XSQLVar::new();
        x.sqltype = SQL_TYPE_NULL;
        assert_eq!(x.io_length(), 0);
        assert_eq!(x.name(), "NULL");
        assert_eq!(x.value(&[]).unwrap(), CellValue::Null);
    }

    #[test]
    fn test_xsqlvar_short_scale() {
        let mut x = XSQLVar::new();
        x.sqltype = SQL_TYPE_SHORT;
        x.sqlscale = 0;
        let bytes = utils::bint32_to_bytes(1234);
        assert_eq!(x.value(&bytes).unwrap(), CellValue::Short(1234));

        x.sqlscale = -2;
        assert_eq!(x.value(&bytes).unwrap(), CellValue::Decimal(dec!(12.34)));

        x.sqlscale = 2;
        assert_eq!(x.value(&bytes).unwrap(), CellValue::Decimal(dec!(123400)));
    }

    #[test]
    fn test_xsqlvar_long_scale() {
        let mut x = XSQLVar::new();
        x.sqltype = SQL_TYPE_LONG;
        x.sqlscale = 0;
        let bytes = utils::bint32_to_bytes(123456);
        assert_eq!(x.value(&bytes).unwrap(), CellValue::Long(123456));

        x.sqlscale = -3;
        assert_eq!(x.value(&bytes).unwrap(), CellValue::Decimal(dec!(123.456)));

        x.sqlscale = 1;
        assert_eq!(x.value(&bytes).unwrap(), CellValue::Decimal(dec!(1234560)));
    }

    #[test]
    fn test_xsqlvar_int64_scale() {
        let mut x = XSQLVar::new();
        x.sqltype = SQL_TYPE_INT64;
        x.sqlscale = 0;
        let bytes = utils::bint64_to_bytes(9876543210);
        assert_eq!(x.value(&bytes).unwrap(), CellValue::Int64(9876543210));

        x.sqlscale = -4;
        assert_eq!(x.value(&bytes).unwrap(), CellValue::Decimal(dec!(987654.3210)));

        x.sqlscale = 2;
        assert_eq!(x.value(&bytes).unwrap(), CellValue::Decimal(dec!(987654321000)));
    }

    #[test]
    fn test_xsqlvar_int128_scale() {
        let mut x = XSQLVar::new();
        x.sqltype = SQL_TYPE_INT128;
        x.sqlscale = 0;
        let bytes = utils::bint128_to_bytes(12345678901234567890123456789i128);
        assert_eq!(
            x.value(&bytes).unwrap(),
            CellValue::Int128(12345678901234567890123456789i128)
        );

        x.sqlscale = -5;
        let bytes_scaled = utils::bint128_to_bytes(12345000i128);
        assert_eq!(
            x.value(&bytes_scaled).unwrap(),
            CellValue::Decimal(dec!(123.45000))
        );

        x.sqlscale = 2;
        let bytes_pos = utils::bint128_to_bytes(5i128);
        assert_eq!(
            x.value(&bytes_pos).unwrap(),
            CellValue::Decimal(dec!(500))
        );
    }
}
