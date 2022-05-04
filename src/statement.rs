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
#![allow(dead_code)]
use super::cellvalue::CellValue;
use super::params::Param;
use super::row::{MappedRows, Row, Rows};
use super::xsqlvar::*;
use super::Connection;
use super::Error;
use maplit::hashmap;
use std::collections::VecDeque;

const ISC_INFO_SQL_STMT_SELECT: u32 = 1;
const ISC_INFO_SQL_STMT_INSERT: u32 = 2;
const ISC_INFO_SQL_STMT_UPDATE: u32 = 3;
const ISC_INFO_SQL_STMT_DELETE: u32 = 4;
const ISC_INFO_SQL_STMT_DDL: u32 = 5;
const ISC_INFO_SQL_STMT_GET_SEGMENT: u32 = 6;
const ISC_INFO_SQL_STMT_PUT_SEGMENT: u32 = 7;
const ISC_INFO_SQL_STMT_EXEC_PROCEDURE: u32 = 8;
const ISC_INFO_SQL_STMT_START_TRANS: u32 = 9;
const ISC_INFO_SQL_STMT_COMMIT: u32 = 10;
const ISC_INFO_SQL_STMT_ROLLBACK: u32 = 11;
const ISC_INFO_SQL_STMT_SELECT_FOR_UPD: u32 = 12;
const ISC_INFO_SQL_STMT_SET_GENERATOR: u32 = 13;
const ISC_INFO_SQL_STMT_SAVEPOINT: u32 = 14;

const DSQL_CLOSE: i32 = 1;
const DSQL_DROP: i32 = 2;

pub struct Statement<'conn> {
    conn: &'conn Connection,
    pub(crate) trans_handle: i32,
    pub(crate) stmt_handle: i32,
    stmt_type: u32,
    pub(crate) xsqlda: Vec<XSQLVar>,
    autocommit: bool,
}

impl Statement<'_> {
    pub(super) fn new(
        conn: &Connection,
        trans_handle: i32,
        stmt_handle: i32,
        stmt_type: u32,
        xsqlda: Vec<XSQLVar>,
        autocommit: bool,
    ) -> Statement {
        Statement {
            conn,
            trans_handle,
            stmt_handle,
            stmt_type,
            xsqlda,
            autocommit,
        }
    }

    fn fetch_records(&self, trans_handle: i32) -> Result<VecDeque<Vec<CellValue>>, Error> {
        let mut rows = VecDeque::new();
        let blr = self.calc_blr();

        loop {
            let (rows_segment, more_data) =
                self.conn._fetch(self.stmt_handle, &blr, &self.xsqlda)?;
            rows.extend(rows_segment);
            if !more_data {
                break;
            }
        }

        for row in rows.iter_mut() {
            for cell in row.iter_mut() {
                match cell {
                    CellValue::BlobBinary(blob_id) => {
                        let blob = self.conn._get_blob_segments(&blob_id, trans_handle);
                        *cell = CellValue::BlobBinary(blob.unwrap());
                    }
                    CellValue::BlobText(blob_id) => {
                        let blob = self.conn._get_blob_segments(&blob_id, trans_handle);
                        *cell = CellValue::BlobText(blob.unwrap());
                    }
                    _ => {}
                }
            }
        }

        Ok(rows)
    }

    pub fn query(&self, params: Vec<Param>) -> Result<Rows<'_>, Error> {
        self.conn
            ._execute_query(self.stmt_handle, self.trans_handle, &params)?;
        let mut rows: VecDeque<Vec<CellValue>> = VecDeque::new();
        if self.stmt_type == ISC_INFO_SQL_STMT_SELECT {
            rows = self.fetch_records(self.trans_handle)?;
            self.conn._free_statement(self.stmt_handle, DSQL_CLOSE);
        } else if self.autocommit {
            // commit automatically
            self.conn.commit()?;
        }

        Ok(Rows::new(self, rows))
    }

    pub fn query_map<T, F>(&mut self, params: Vec<Param>, f: F) -> Result<MappedRows<'_, F>, Error>
    where
        F: FnMut(&Row<'_>) -> Result<T, Error>,
    {
        self.query(params).map(|rows| rows.mapped(f))
    }

    pub fn execute(&mut self, params: Vec<Param>) -> Result<(), Error> {
        self.query(params)?;
        Ok(())
    }

    fn calc_blr(&self) -> Vec<u8> {
        let ln = self.xsqlda.len() * 2;
        let mut blr: Vec<u8> = vec![5, 2, 4, 0, (ln & 255) as u8, (ln >> 8) as u8];

        for x in &self.xsqlda {
            let map = hashmap! {
                SQL_TYPE_VARYING => vec![37, (x.sqllen & 255) as u8, (x.sqllen >> 8) as u8],
                SQL_TYPE_TEXT => vec![14, (x.sqllen & 255) as u8, (x.sqllen >> 8) as u8],
                SQL_TYPE_LONG => vec![8, x.sqlscale as u8],
                SQL_TYPE_SHORT => vec![7, x.sqlscale as u8],
                SQL_TYPE_INT64 => vec![16, x.sqlscale as u8],
                SQL_TYPE_INT128 => vec![26, x.sqlscale as u8],
                SQL_TYPE_QUAD => vec![9, x.sqlscale as u8],
                SQL_TYPE_DEC_FIXED => vec![26, x.sqlscale as u8],
                SQL_TYPE_DOUBLE => vec![27],
                SQL_TYPE_FLOAT => vec![10],
                SQL_TYPE_D_FLOAT => vec![11],
                SQL_TYPE_DATE => vec![12],
                SQL_TYPE_TIME => vec![13],
                SQL_TYPE_TIMESTAMP => vec![35],
                SQL_TYPE_BLOB => vec![9, 0],
                SQL_TYPE_ARRAY => vec![9, 0],
                SQL_TYPE_BOOLEAN => vec![23],
                SQL_TYPE_DEC64 => vec![24],
                SQL_TYPE_DEC128 => vec![25],
                SQL_TYPE_TIME_TZ => vec![28],
                SQL_TYPE_TIMESTAMP_TZ => vec![29],
            };
            blr.extend(&map[&x.sqltype]);
            blr.extend(vec![7, 0]);
        }
        blr.extend(vec![255, 76]);

        blr
    }
}

impl Drop for Statement<'_> {
    fn drop(&mut self) {
        self.conn._free_statement(self.stmt_handle, DSQL_DROP);
    }
}
