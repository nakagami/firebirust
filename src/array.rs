// MIT License
//
// Copyright (c) 2026 Hajime Nakagami<nakagami@gmail.com>
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

use super::cellvalue::CellValue;
use super::error::{Error, ValueError};
use super::xsqlvar::XSQLVar;
use super::*;

pub const ISC_SDL_VERSION1: u8 = 1;
pub const ISC_SDL_RELATION: u8 = 2;
pub const ISC_SDL_FIELD: u8 = 4;
pub const ISC_SDL_STRUCT: u8 = 6;
pub const ISC_SDL_VARIABLE: u8 = 7;
pub const ISC_SDL_SCALAR: u8 = 8;
pub const ISC_SDL_TINY_INTEGER: u8 = 9;
pub const ISC_SDL_SHORT_INTEGER: u8 = 10;
pub const ISC_SDL_LONG_INTEGER: u8 = 11;
pub const ISC_SDL_DO2: u8 = 34;
pub const ISC_SDL_DO1: u8 = 35;
pub const ISC_SDL_ELEMENT: u8 = 36;
pub const ISC_SDL_EOC: u8 = 255;

pub const OP_GET_SLICE: u32 = 58;
pub const OP_PUT_SLICE: u32 = 59;
pub const OP_SLICE_RESPONSE: u32 = 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayDimension {
    pub lower_bound: i32,
    pub length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayMeta {
    pub table_name: String,
    pub field_name: String,
    pub type_id: u32,
    pub blr_type_id: u8,
    pub scale: i32,
    pub length: u32,
    pub field_bytes: u32,
    pub sub_type: i32,
    pub field_source: String,
    pub dimensions: Vec<ArrayDimension>,
}

pub fn array_type_from_blr(blr: i64) -> Result<u32, Error> {
    match blr {
        37 | 46 => Ok(SQL_TYPE_VARYING),
        14 | 40 | 41 | 45 => Ok(SQL_TYPE_TEXT),
        7 => Ok(SQL_TYPE_SHORT),
        8 => Ok(SQL_TYPE_LONG),
        9 => Ok(SQL_TYPE_QUAD),
        16 => Ok(SQL_TYPE_INT64),
        27 => Ok(SQL_TYPE_DOUBLE),
        10 => Ok(SQL_TYPE_FLOAT),
        11 => Ok(SQL_TYPE_D_FLOAT),
        12 => Ok(SQL_TYPE_DATE),
        13 => Ok(SQL_TYPE_TIME),
        28 => Ok(SQL_TYPE_TIME_TZ),
        35 => Ok(SQL_TYPE_TIMESTAMP),
        29 => Ok(SQL_TYPE_TIMESTAMP_TZ),
        23 => Ok(SQL_TYPE_BOOLEAN),
        24 => Ok(SQL_TYPE_DEC64),
        25 => Ok(SQL_TYPE_DEC128),
        26 => Ok(SQL_TYPE_INT128),
        _ => Err(Error::ValueError(ValueError::new(&format!(
            "unknown array element blr type {}",
            blr
        )))),
    }
}

pub fn round_up_4(n: usize) -> usize {
    let r = n % 4;
    if r != 0 {
        n + 4 - r
    } else {
        n
    }
}

pub fn array_element_stride(meta: &ArrayMeta) -> usize {
    match meta.type_id {
        SQL_TYPE_SHORT => 2,
        SQL_TYPE_LONG | SQL_TYPE_FLOAT | SQL_TYPE_DATE | SQL_TYPE_TIME => 4,
        SQL_TYPE_INT64 | SQL_TYPE_DOUBLE | SQL_TYPE_TIMESTAMP | SQL_TYPE_DEC64 => 8,
        SQL_TYPE_INT128 | SQL_TYPE_DEC128 | SQL_TYPE_DEC_FIXED => 16,
        SQL_TYPE_BOOLEAN => 1,
        SQL_TYPE_TEXT => meta.length as usize,
        SQL_TYPE_VARYING => 2 + meta.length as usize,
        _ => meta.length as usize,
    }
}

pub fn array_get_element_stride(meta: &ArrayMeta) -> usize {
    match meta.type_id {
        SQL_TYPE_TEXT => meta.field_bytes as usize,
        SQL_TYPE_VARYING => 2 + meta.field_bytes as usize,
        _ => array_element_stride(meta),
    }
}

pub fn all_elements_count(dims: &[ArrayDimension]) -> usize {
    if dims.is_empty() {
        return 0;
    }
    let mut count = 1;
    for d in dims {
        count *= d.length;
    }
    count
}

pub fn array_slice_get_length(meta: &ArrayMeta, dims: &[ArrayDimension]) -> usize {
    let count = all_elements_count(dims);
    count * array_get_element_stride(meta)
}

pub fn array_slice_app_length(meta: &ArrayMeta, dims: &[ArrayDimension]) -> usize {
    let count = all_elements_count(dims);
    count * array_element_stride(meta)
}

fn append_dimension_bound(sdl: &mut Vec<u8>, v: i32) {
    if (-128..=127).contains(&v) {
        sdl.push(ISC_SDL_TINY_INTEGER);
        sdl.push(v as u8);
    } else if (-32768..=32767).contains(&v) {
        sdl.push(ISC_SDL_SHORT_INTEGER);
        sdl.extend_from_slice(&(v as i16).to_le_bytes());
    } else {
        sdl.push(ISC_SDL_LONG_INTEGER);
        sdl.extend_from_slice(&v.to_le_bytes());
    }
}

pub fn generate_sdl(meta: &ArrayMeta, dims: &[ArrayDimension], for_get: bool) -> Vec<u8> {
    let mut sdl = Vec::with_capacity(64);
    sdl.push(ISC_SDL_VERSION1);
    sdl.push(ISC_SDL_STRUCT);
    sdl.push(1);
    sdl.push(meta.blr_type_id);

    match meta.blr_type_id {
        7 | 8 | 9 | 16 | 26 => {
            sdl.push(meta.scale as u8);
        }
        14 | 37 | 40 | 41 | 45 | 46 => {
            let text_len = if for_get {
                meta.field_bytes
            } else {
                meta.length
            };
            sdl.extend_from_slice(&(text_len as u16).to_le_bytes());
        }
        _ => {}
    }

    let tbl = meta.table_name.as_bytes();
    sdl.push(ISC_SDL_RELATION);
    sdl.push(tbl.len() as u8);
    sdl.extend_from_slice(tbl);

    let fld = meta.field_name.as_bytes();
    sdl.push(ISC_SDL_FIELD);
    sdl.push(fld.len() as u8);
    sdl.extend_from_slice(fld);

    for (i, d) in dims.iter().enumerate() {
        if d.lower_bound == 1 {
            sdl.push(ISC_SDL_DO1);
            sdl.push(i as u8);
        } else {
            sdl.push(ISC_SDL_DO2);
            sdl.push(i as u8);
            append_dimension_bound(&mut sdl, d.lower_bound);
        }
        append_dimension_bound(&mut sdl, d.lower_bound + d.length as i32 - 1);
    }

    sdl.push(ISC_SDL_ELEMENT);
    sdl.push(1);
    sdl.push(ISC_SDL_SCALAR);
    sdl.push(0);
    sdl.push(dims.len() as u8);
    for i in 0..dims.len() {
        sdl.push(ISC_SDL_VARIABLE);
        sdl.push(i as u8);
    }
    sdl.push(ISC_SDL_EOC);
    sdl
}

pub fn slice_wire_element_size(meta: &ArrayMeta) -> (usize, bool) {
    match meta.type_id {
        SQL_TYPE_INT128 | SQL_TYPE_DEC128 | SQL_TYPE_DEC_FIXED => (16, false),
        SQL_TYPE_DEC64 | SQL_TYPE_INT64 | SQL_TYPE_DOUBLE | SQL_TYPE_TIMESTAMP => (8, false),
        SQL_TYPE_VARYING => (meta.field_bytes as usize, true),
        SQL_TYPE_TEXT => (round_up_4(meta.field_bytes as usize), false),
        _ => (4, false),
    }
}

pub fn put_slice_wire_element_size(meta: &ArrayMeta) -> usize {
    if meta.type_id == SQL_TYPE_TEXT {
        return round_up_4(meta.field_bytes as usize);
    }
    let (size, varying) = slice_wire_element_size(meta);
    if varying {
        4
    } else {
        size
    }
}

pub fn pad_slice_wire(meta: &ArrayMeta, data: &[u8], actual_count: usize) -> Vec<u8> {
    let declared = all_elements_count(&meta.dimensions);
    let mut out = data.to_vec();
    if declared > actual_count {
        let missing = declared - actual_count;
        let per = put_slice_wire_element_size(meta);
        out.resize(out.len() + missing * per, 0);
    }
    out
}

pub fn format_array_output(dims: &[ArrayDimension], flat: Vec<CellValue>) -> CellValue {
    if dims.is_empty() {
        return CellValue::Array(flat);
    }
    let mut current = flat;
    for d in dims[1..].iter().rev() {
        let chunk_len = d.length;
        if chunk_len == 0 {
            break;
        }
        let mut next = Vec::new();
        for chunk in current.chunks(chunk_len) {
            next.push(CellValue::Array(chunk.to_vec()));
        }
        current = next;
    }
    CellValue::Array(current)
}

pub fn decode_slice_elements(meta: &ArrayMeta, data: &[u8]) -> Result<CellValue, Error> {
    if data.is_empty() {
        return Ok(CellValue::Array(Vec::new()));
    }
    let total_count = all_elements_count(&meta.dimensions);
    let (elem_len, varying) = slice_wire_element_size(meta);

    let mut elem_xsqlvar = XSQLVar::new();
    elem_xsqlvar.sqltype = meta.type_id;
    elem_xsqlvar.sqlscale = meta.scale;
    elem_xsqlvar.sqlsubtype = meta.sub_type;
    elem_xsqlvar.sqllen = meta.length as i32;

    let mut flat = Vec::with_capacity(total_count);
    let mut pos = 0;
    while pos < data.len() && flat.len() < total_count {
        if varying {
            if data.len() - pos < 4 {
                return Err(Error::ValueError(ValueError::new("array body truncated")));
            }
            let l = utils::bytes_to_buint32(&data[pos..pos + 4]) as usize;
            pos += 4;
            if pos + l > data.len() {
                return Err(Error::ValueError(ValueError::new(&format!(
                    "array varchar element length {} exceeds data (pos={}, data.len()={})",
                    l, pos, data.len()
                ))));
            }
            let raw = &data[pos..pos + l];
            let v = elem_xsqlvar.value(raw)?;
            flat.push(v);
            pos += l;
        } else {
            if pos + elem_len > data.len() {
                return Err(Error::ValueError(ValueError::new("array body truncated")));
            }
            let mut raw = &data[pos..pos + elem_len];
            pos += elem_len;
            if meta.type_id == SQL_TYPE_BOOLEAN {
                raw = &raw[..1];
            }
            let v = elem_xsqlvar.value(raw)?;
            flat.push(v);
        }
    }

    Ok(format_array_output(&meta.dimensions, flat))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_array_output_1d() {
        let dims = vec![ArrayDimension {
            lower_bound: 1,
            length: 3,
        }];
        let flat = vec![
            CellValue::Long(10),
            CellValue::Long(20),
            CellValue::Long(30),
        ];
        let res = format_array_output(&dims, flat.clone());
        assert_eq!(res, CellValue::Array(flat));
    }

    #[test]
    fn test_format_array_output_2d() {
        let dims = vec![
            ArrayDimension {
                lower_bound: 1,
                length: 2,
            },
            ArrayDimension {
                lower_bound: 1,
                length: 3,
            },
        ];
        let flat = vec![
            CellValue::Long(1),
            CellValue::Long(2),
            CellValue::Long(3),
            CellValue::Long(4),
            CellValue::Long(5),
            CellValue::Long(6),
        ];
        let res = format_array_output(&dims, flat);
        assert_eq!(
            res,
            CellValue::Array(vec![
                CellValue::Array(vec![
                    CellValue::Long(1),
                    CellValue::Long(2),
                    CellValue::Long(3)
                ]),
                CellValue::Array(vec![
                    CellValue::Long(4),
                    CellValue::Long(5),
                    CellValue::Long(6)
                ]),
            ])
        );
    }

    #[test]
    fn test_generate_sdl_int_1d() {
        let meta = ArrayMeta {
            table_name: "T".to_string(),
            field_name: "INTS".to_string(),
            type_id: SQL_TYPE_LONG,
            blr_type_id: 8,
            scale: 0,
            length: 4,
            field_bytes: 4,
            sub_type: 0,
            field_source: "RDB$1".to_string(),
            dimensions: vec![ArrayDimension {
                lower_bound: 1,
                length: 5,
            }],
        };
        let sdl = generate_sdl(&meta, &meta.dimensions, false);
        let want = vec![
            1, 6, 1, 8, 0,
            2, 1, b'T',
            4, 4, b'I', b'N', b'T', b'S',
            35, 0, 9, 5,
            36, 1, 8, 0, 1, 7, 0,
            255,
        ];
        assert_eq!(sdl, want);
    }
}
