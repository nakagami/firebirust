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

use super::error::Error;
use super::param::Param;
use super::statement::Statement;
use super::Connection;

pub struct Transaction<'conn> {
    conn: &'conn mut Connection,
    trans_handle: i32,
}

impl Transaction<'_> {
    pub fn new(conn: &mut Connection) -> Result<Transaction, Error> {
        let trans_handle = conn._begin_trans()?;
        Ok(Transaction { conn, trans_handle })
    }

    pub fn execute_batch(&mut self, query: &str) -> Result<(), Error> {
        self.conn._execute_batch(query, self.trans_handle)
    }

    pub fn execute(&mut self, query: &str, params: Vec<Param>) -> Result<(), Error> {
        self.conn._execute(query, params, self.trans_handle)
    }

    pub fn commit(&mut self) -> Result<(), Error> {
        self.conn._commit(self.trans_handle)
    }

    pub fn rollback(&mut self) -> Result<(), Error> {
        self.conn._rollback(self.trans_handle)
    }

    pub fn prepare(&mut self, query: &str) -> Result<Statement, Error> {
        self.conn._prepare(query, self.trans_handle)
    }
}

impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        self.conn.drop_transaction(self.trans_handle);
    }
}
