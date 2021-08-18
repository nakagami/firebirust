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
use super::{Error, Statement, Value};
use std::collections::VecDeque;
use std::result::Result;

#[derive(PartialEq)]
pub struct ResultSetMetaData {}

pub struct Rows<'stmt> {
    pub(crate) stmt: &'stmt Statement<'stmt>,
    rows: VecDeque<Vec<Value>>,
}

impl Rows<'_> {
    pub(crate) fn new<'a>(stmt: &'a Statement, rows: VecDeque<Vec<Value>>) -> Rows<'a> {
        Rows {
            stmt,
            rows,
        }
    }

    /*
        pub fn meta_data(&mut self) -> ResultSetMetaData {
          // TODO:
        }
    */

    pub fn get_i8(&self, i: u64) -> Result<Option<i8>, Error> {
        self.rows[0][i as usize].get_i8()
    }

    pub fn get_i16(&self, i: u64) -> Result<Option<i16>, Error> {
        self.rows[0][i as usize].get_i16()
    }

    pub fn get_i32(&self, i: u64) -> Result<Option<i32>, Error> {
        self.rows[0][i as usize].get_i32()
    }

    pub fn get_i64(&self, i: u64) -> Result<Option<i64>, Error> {
        self.rows[0][i as usize].get_i64()
    }

    pub fn get_f32(&self, i: u64) -> Result<Option<f32>, Error> {
        self.rows[0][i as usize].get_f32()
    }

    pub fn get_f64(&self, i: u64) -> Result<Option<f64>, Error> {
        self.rows[0][i as usize].get_f64()
    }

    pub fn get_string(&self, i: u64) -> Result<Option<String>, Error> {
        self.rows[0][i as usize].get_string()
    }

    pub fn get_bytes(&self, i: u64) -> Result<Option<Vec<u8>>, Error> {
        self.rows[0][i as usize].get_bytes()
    }

    // TODO: get other types
}

impl<'stmt> Iterator for Rows<'stmt> {
    type Item = Row<'stmt>;

    fn next(&mut self) -> Option<Row<'stmt>> {
        match self.rows.pop_front() {
            Some(row) => Some(Row {
                stmt: self.stmt,
                row: row,
            }),
            None => None,
        }
    }
}

pub struct Row<'stmt> {
    pub(crate) stmt: &'stmt Statement<'stmt>,
    row: Vec<Value>,
}
