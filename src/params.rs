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
use super::statement_async::StatementAsync;

mod sealed {
    /// This trait exists just to ensure that the only impls of `trait Params`
    /// that are allowed are ones in this crate.
    pub trait Sealed {}
}
use sealed::Sealed;

pub trait Params: Sealed {
    #[doc(hidden)]
    fn __bind_in(self, stmt: &mut Statement<'_>) -> Result<(), Error>;
    #[doc(hidden)]
    fn __bind_in_async(self, stmt: &mut StatementAsync<'_>) -> Result<(), Error>;
}

impl Sealed for [&(dyn ToSqlParam + Send + Sync); 0] {}
impl Params for [&(dyn ToSqlParam + Send + Sync); 0] {
    #[inline]
    fn __bind_in(self, stmt: &mut Statement<'_>) -> Result<(), Error> {
        stmt.bind_parameters(&[])
    }
    #[inline]
    fn __bind_in_async(self, stmt: &mut StatementAsync<'_>) -> Result<(), Error> {
        stmt.bind_parameters(&[])
    }
}

impl Sealed for &[&dyn ToSqlParam] {}
impl Params for &[&dyn ToSqlParam] {
    #[inline]
    fn __bind_in(self, stmt: &mut Statement<'_>) -> Result<(), Error> {
        stmt.bind_parameters(self)
    }
    #[inline]
    fn __bind_in_async(self, stmt: &mut StatementAsync<'_>) -> Result<(), Error> {
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
    #[inline]
    fn __bind_in_async(self, stmt: &mut StatementAsync<'_>) -> Result<(), Error> {
        stmt.bind_parameters(&[])
    }
}

macro_rules! single_tuple_impl {
    ($count:literal : $(($field:tt $ftype:ident)),* $(,)?) => {
        impl<$($ftype,)*> Sealed for ($($ftype,)*) where $($ftype: ToSqlParam,)* {}
        impl<$($ftype,)*> Params for ($($ftype,)*) where $($ftype: ToSqlParam,)* {
            fn __bind_in(self, stmt: &mut Statement<'_>) -> Result<(), Error> {
                stmt.reset_parameter($count)?;
                $({
                    stmt.put_parameter(self.$field)?;
                })+
                Ok(())
            }
            fn __bind_in_async(self, stmt: &mut StatementAsync<'_>) -> Result<(), Error> {
                stmt.reset_parameter($count)?;
                $({
                    stmt.put_parameter(self.$field)?;
                })+
                Ok(())
            }
        }
    }
}

single_tuple_impl!(1: (0 A));
single_tuple_impl!(2: (0 A), (1 B));
single_tuple_impl!(3: (0 A), (1 B), (2 C));
single_tuple_impl!(4: (0 A), (1 B), (2 C), (3 D));
single_tuple_impl!(5: (0 A), (1 B), (2 C), (3 D), (4 E));
single_tuple_impl!(6: (0 A), (1 B), (2 C), (3 D), (4 E), (5 F));
single_tuple_impl!(7: (0 A), (1 B), (2 C), (3 D), (4 E), (5 F), (6 G));
single_tuple_impl!(8: (0 A), (1 B), (2 C), (3 D), (4 E), (5 F), (6 G), (7 H));
single_tuple_impl!(9: (0 A), (1 B), (2 C), (3 D), (4 E), (5 F), (6 G), (7 H), (8 I));
single_tuple_impl!(10: (0 A), (1 B), (2 C), (3 D), (4 E), (5 F), (6 G), (7 H), (8 I), (9 J));
single_tuple_impl!(11: (0 A), (1 B), (2 C), (3 D), (4 E), (5 F), (6 G), (7 H), (8 I), (9 J), (10 K));
single_tuple_impl!(12: (0 A), (1 B), (2 C), (3 D), (4 E), (5 F), (6 G), (7 H), (8 I), (9 J), (10 K), (11 L));
single_tuple_impl!(13: (0 A), (1 B), (2 C), (3 D), (4 E), (5 F), (6 G), (7 H), (8 I), (9 J), (10 K), (11 L), (12 M));
single_tuple_impl!(14: (0 A), (1 B), (2 C), (3 D), (4 E), (5 F), (6 G), (7 H), (8 I), (9 J), (10 K), (11 L), (12 M), (13 N));
single_tuple_impl!(15: (0 A), (1 B), (2 C), (3 D), (4 E), (5 F), (6 G), (7 H), (8 I), (9 J), (10 K), (11 L), (12 M), (13 N), (14 O));
single_tuple_impl!(16: (0 A), (1 B), (2 C), (3 D), (4 E), (5 F), (6 G), (7 H), (8 I), (9 J), (10 K), (11 L), (12 M), (13 N), (14 O), (15 P));
