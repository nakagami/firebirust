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

use std::{error, fmt, io};
use url::ParseError;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    FirebirdError(FirebirdError),
    ValueError(ValueError),
    ParamError(ParamError),
    UrlError(UrlError),
}

impl From<UrlError> for Error {
    fn from(err: UrlError) -> Error {
        Error::UrlError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<FirebirdError> for Error {
    fn from(x: FirebirdError) -> Error {
        Error::FirebirdError(x)
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct FirebirdError {
    pub message: String,
    pub sql_code: i32,
}

impl FirebirdError {
    pub fn new(message: &str, sql_code: i32) -> FirebirdError {
        let message = message.to_string();
        FirebirdError { message, sql_code }
    }
}

impl From<ValueError> for Error {
    fn from(x: ValueError) -> Error {
        Error::ValueError(x)
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct ValueError {
    pub message: String,
}

impl ValueError {
    pub fn new(message: &str) -> ValueError {
        ValueError {
            message: message.to_string(),
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct ParamError {
    pub message: String,
}

impl ParamError {
    pub fn new(message: &str) -> ParamError {
        ParamError {
            message: message.to_string(),
        }
    }
}

#[derive(Eq, PartialEq, Clone)]
pub enum UrlError {
    ParseError(ParseError),
    UnsupportedScheme(String),
    /// (feature_name, parameter_name)
    FeatureRequired(String, String),
    /// (feature_name, value)
    InvalidValue(String, String),
    UnknownParameter(String),
    BadUrl,
}

impl error::Error for UrlError {
    fn description(&self) -> &str {
        "Database connection URL error"
    }
}

impl fmt::Display for UrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            UrlError::ParseError(ref err) => write!(f, "URL ParseError {{ {} }}", err),
            UrlError::UnsupportedScheme(ref s) => write!(f, "URL scheme `{}' is not supported", s),
            UrlError::FeatureRequired(ref feature, ref parameter) => write!(
                f,
                "Url parameter `{}' requires {} feature",
                parameter, feature
            ),
            UrlError::InvalidValue(ref parameter, ref value) => write!(
                f,
                "Invalid value `{}' for URL parameter `{}'",
                value, parameter
            ),
            UrlError::UnknownParameter(ref parameter) => {
                write!(f, "Unknown URL parameter `{}'", parameter)
            }
            UrlError::BadUrl => write!(f, "Invalid or incomplete connection URL"),
        }
    }
}

impl fmt::Debug for UrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl From<ParseError> for UrlError {
    fn from(x: ParseError) -> UrlError {
        UrlError::ParseError(x)
    }
}
