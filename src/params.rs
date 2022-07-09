// Copyright (c) 2014-2021 The rusqlite developers
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use super::error::Error;
use super::param::ToSqlParam;
use super::statement::Statement;

mod sealed {
    /// This trait exists just to ensure that the only impls of `trait Params`
    /// that are allowed are ones in this crate.
    pub trait Sealed {}
}
use sealed::Sealed;

pub trait Params: Sealed {
    #[doc(hidden)]
    fn __bind_in(self, stmt: &mut Statement<'_>) -> Result<(), Error>;
}

impl Sealed for [&(dyn ToSqlParam + Send + Sync); 0] {}
impl Params for [&(dyn ToSqlParam + Send + Sync); 0] {
    #[inline]
    fn __bind_in(self, stmt: &mut Statement<'_>) -> Result<(), Error> {
        stmt.bind_parameters(&[])
    }
}

impl Sealed for &[&dyn ToSqlParam] {}
impl Params for &[&dyn ToSqlParam] {
    #[inline]
    fn __bind_in(self, stmt: &mut Statement<'_>) -> Result<(), Error> {
        stmt.bind_parameters(self)
    }
}

// Manual impls for the empty and singleton tuple, although the rest are covered
// by macros.
impl Sealed for () {}
impl Params for () {
    #[inline]
    fn __bind_in(self, stmt: &mut Statement<'_>) -> Result<(), Error> {
        stmt.bind_parameters(&[])
    }
}
