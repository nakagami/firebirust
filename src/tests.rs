// MIT License
//
// Copyright (c) 2020 Hajime Nakagami<nakagami@gmail.com>
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
use super::params;
use super::Connection;
use super::Param;

#[derive(Debug)]
struct FooRow {
    a: i32,
    b: String,
    c: String,
    i: f64,
    j: f32,
}

#[test]
fn test_connnect() {
    let mut conn;
    match Connection::create_database(
        "firebird://SYSDBA:masterkey@localhost/tmp/rust-firebird-test.fdb",
    ) {
        Ok(c) => {
            conn = c;
        }
        Err(err) => {
            println!("{:#?}", err);
            panic!("Can't connect");
        }
    }

    conn.execute_batch(
        r#"
        CREATE TABLE foo (
            a INTEGER NOT NULL,
            b VARCHAR(30) NOT NULL UNIQUE,
            c VARCHAR(1024),
            d DECIMAL(16,3) DEFAULT -0.123,
            e DATE DEFAULT '1967-08-11',
            f TIMESTAMP DEFAULT '1967-08-11 23:45:01',
            g TIME DEFAULT '23:45:01',
            h BLOB SUB_TYPE 1,
            i DOUBLE PRECISION DEFAULT 0.0,
            j FLOAT DEFAULT 0.0,
            PRIMARY KEY (a),
            CONSTRAINT CHECK_A CHECK (a <> 0)
        )
    "#,
    )
    .unwrap();

    conn.execute(
        "insert into foo(a, b, c, h) values (?, ?, ?, ?)",
        params![1, "a", "b", "This is a memo"],
    )
    .unwrap();

    conn.execute("insert into foo(a, b, c, e, g, i, j) values (2, 'A', 'B', '1999-01-25', '00:00:01', 0.1, 0.1)", params![]).unwrap();
    conn.execute("insert into foo(a, b, c, e, g, i, j) values (3, 'X', 'Y', '2001-07-05', '00:01:02', 0.2, 0.2)", params![]).unwrap();

    let mut stmt = conn.prepare("select * from foo").unwrap();

    for row in stmt.query(&params![]).unwrap() {
        println!("something row")
    }
}
