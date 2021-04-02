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
use super::ResultSet;
use super::Value;

pub struct Statement<'conn> {
    conn: &'conn Connection,
    pub(crate) stmt_handle: i32,
    stmt_type: u32,
    xsqlda: Vec<XSQLVar>,
}

impl Statement<'_> {
    pub(super) fn new(
        conn: &Connection,
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

    pub fn execute_query(&mut self, params: &[Value]) -> Result<ResultSet, Error> {
        let mut wp = self.conn.wp.borrow_mut();
        wp.op_execute(self.stmt_handle, self.conn.trans_handle, params)?;
        wp.parse_op_response()?;
        // TODO: add new parameter
        Ok(ResultSet::new(self))
    }

    pub fn execute_update(&mut self, params: &[Value]) -> Result<u64, Error> {
        // TODO:
        Ok(0)
    }
}
