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
use super::xsqlvar::XSQLVar;
use super::Connection;
use super::Error;
use super::Param;
use super::Rows;
use super::Value;

pub struct Statement<'conn> {
    conn: &'conn mut Connection,
    pub(crate) stmt_handle: i32,
    stmt_type: u32,
    pub(crate) xsqlda: Vec<XSQLVar>,
}

impl Statement<'_> {
    pub(super) fn new(
        conn: &mut Connection,
        stmt_handle: i32,
        stmt_type: u32,
        xsqlda: Vec<XSQLVar>,
    ) -> Statement {
        Statement {
            conn,
            stmt_handle,
            stmt_type,
            xsqlda,
        }
    }

    pub fn query(&mut self, params: &Vec<Param>) -> Result<Rows<'_>, Error> {
        self.conn
            .wp
            .op_execute(self.stmt_handle, self.conn.trans_handle, &params)?;
        self.conn.wp.op_response()?;
        Ok(Rows::new(self))
    }

    pub fn execute(&mut self, params: &Vec<Param>) -> Result<(), Error> {
        self.query(params)?;
        Ok(())
    }

    pub fn execute_query(&mut self, params: &Vec<Param>) -> Result<Rows, Error> {
        self.conn
            .wp
            .op_execute(self.stmt_handle, self.conn.trans_handle, params)?;
        self.conn.wp.parse_op_response()?;
        // TODO: add new parameter
        Ok(Rows::new(self))
    }

    pub fn execute_update(&mut self, params: &[Value]) -> Result<u64, Error> {
        // TODO:
        Ok(0)
    }

    pub fn query_map(&mut self, param: &[Value]) -> Result<u64, Error> {
        // TODO:
        Ok(0)
    }

    fn calc_blr(&mut self) -> Vec<u8> {
        let ln = self.xsqlda.len();
        let mut blr: Vec<u8> = vec![5, 2, 4, 0, (ln & 255) as u8, (ln >> 8) as u8];

        for x in &self.xsqlda {
            blr.extend(match x.sqltype {
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
            });
            blr.extend(vec![7, 0]);
        }
        blr.extend(vec![255, 76]);

        blr
    }

    fn fetch_segment(&mut self) -> Result<(Vec<Vec<Value>>, bool), Error> {
        let blr = self.calc_blr();
        self.conn.wp.op_fetch_response(self.stmt_handle, self.conn.trans_handle, &self.xsqlda)
    }
}
