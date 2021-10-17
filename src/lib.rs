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
mod cellvalue;
mod conn_params;
mod connection;
mod decfloat;
mod errmsgs;
mod error;
mod param;
mod row;
mod srp;
mod statement;
mod transaction;
mod tz_map;
mod utils;
mod wirechannel;
mod wireprotocol;
mod xsqlvar;

pub use crate::connection::Connection;
pub use crate::error::Error;
pub use crate::param::Param;

#[macro_export]
macro_rules! params {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($crate::Param::from($x));
            )*
            temp_vec.push($crate::Param::Null);
            temp_vec.pop();
            temp_vec
        }
    };
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_timezone;
