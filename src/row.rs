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
use super::error::{Error, ValueError};
use super::value::CellValueToVal;
use super::{CellValue, Statement};
use std::collections::VecDeque;
use std::result::Result;

#[derive(PartialEq)]
pub struct ResultSetMetaData {}

pub struct Rows<'stmt> {
    pub(crate) stmt: &'stmt Statement<'stmt>,
    rows: VecDeque<Vec<CellValue>>,
}

impl Rows<'_> {
    pub(crate) fn new<'a>(stmt: &'a Statement, rows: VecDeque<Vec<CellValue>>) -> Rows<'a> {
        Rows { stmt, rows }
    }

    /*
        pub fn meta_data(&mut self) -> ResultSetMetaData {
          // TODO:
        }
    */
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
    row: Vec<CellValue>,
}

impl<'stmt> Row<'stmt> {
    pub fn get<T>(&self, idx: usize) -> Result<T, Error>
    where
        CellValue: CellValueToVal<T>,
    {
        if let Some(cell_value) = self.row.get(idx) {
            cell_value.clone().to_val()
        } else {
            Err(Error::ValueError(ValueError::new(
                "This index doesn't exists",
            )))
        }
    }
}
