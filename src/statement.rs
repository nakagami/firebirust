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
    xsqlda: Vec<XSQLVar>,
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

    pub fn execute(&mut self, params: &Vec<Param>) -> Result<(), Error> {
        self.conn
            .wp
            .op_execute(self.stmt_handle, self.conn.trans_handle, &params)?;
        self.conn.wp.op_response()?;
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
}
