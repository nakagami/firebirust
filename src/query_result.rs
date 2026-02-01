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
use super::cellvalue::{CellValue, CellValueToVal};
use super::error::{Error, ValueError};
use std::collections::VecDeque;
use std::result::Result;

pub struct QueryResult {
    rows: VecDeque<Vec<CellValue>>,
    rows_affected: usize,
}

impl QueryResult {
    pub(crate) fn new(rows: VecDeque<Vec<CellValue>>, rows_affected: usize) -> QueryResult {
        QueryResult {
            rows,
            rows_affected,
        }
    }

    pub fn mapped<F, B>(self, f: F) -> MappedRows<F>
    where
        F: FnMut(&Row) -> Result<B, Error>,
    {
        MappedRows {
            result: self,
            map: f,
        }
    }
}

impl Iterator for QueryResult {
    type Item = Row;

    fn next(&mut self) -> Option<Row> {
        match self.rows.pop_front() {
            Some(row) => Some(Row { row: row }),
            None => None,
        }
    }
}

pub struct Row {
    row: Vec<CellValue>,
}

impl Row {
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

pub struct MappedRows<F> {
    result: QueryResult,
    map: F,
}

impl<T, F> Iterator for MappedRows<F>
where
    F: FnMut(&Row) -> Result<T, Error>,
{
    type Item = Result<T, Error>;

    fn next(&mut self) -> Option<Result<T, Error>> {
        let map = &mut self.map;
        let row = self.result.next();
        match row {
            Some(r) => Some(map(&r)),
            None => None,
        }
    }
}
