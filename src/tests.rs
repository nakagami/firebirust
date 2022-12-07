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
use super::Connection;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::env;
use urlencoding;

#[derive(PartialEq, Debug)]
struct Foo {
    a: i32,
    b: String,
    c: String,
    d: Decimal,
    e: NaiveDate,
    f: NaiveDateTime,
    g: NaiveTime,
    h: Option<Vec<u8>>,
    i: f64,
    j: f32,
}

#[test]
fn test_connnect() {
    let user = match env::var("ISC_USER") {
        Ok(val) => val,
        Err(_) => "sysdba".to_string(),
    };
    let password = match env::var("ISC_PASSWORD") {
        Ok(val) => val,
        Err(_) => "masterkey".to_string(),
    };

    let mut conn;
    let conn_string = format!(
        "firebird://{}:{}@localhost/tmp/rust-firebird-test.fdb",
        &user,
        urlencoding::encode(&password)
    );

    match Connection::create_database(&conn_string) {
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
        (1, "a", "b", "This is a pen"),
    )
    .unwrap();

    conn.execute(
        "insert into foo(a, b, c, e, g, i, j) values (?, 'A', 'B', '1999-01-25', '00:00:01', 0.1, 0.1)",
        (2, )
    )
    .unwrap();
    conn.execute("insert into foo(a, b, c, e, g, i, j) values (3, 'X', 'Y', '2001-07-05', '00:01:02', 0.2, 0.2)", ()).unwrap();

    {
        let expects: [Foo; 3] = [
            Foo {
                a: 1,
                b: "a".to_string(),
                c: "b".to_string(),
                d: dec!(-0.123),
                e: NaiveDate::from_ymd(1967, 8, 11),
                f: NaiveDate::from_ymd(1967, 8, 11).and_hms(23, 45, 1),
                g: NaiveTime::from_hms(23, 45, 1),
                h: Some("This is a pen".to_string().into_bytes()),
                i: 0.0,
                j: 0.0,
            },
            Foo {
                a: 2,
                b: "A".to_string(),
                c: "B".to_string(),
                d: dec!(-0.123),
                e: NaiveDate::from_ymd(1999, 1, 25),
                f: NaiveDate::from_ymd(1967, 8, 11).and_hms(23, 45, 1),
                g: NaiveTime::from_hms(0, 0, 1),
                h: None,
                i: 0.1,
                j: 0.1,
            },
            Foo {
                a: 3,
                b: "X".to_string(),
                c: "Y".to_string(),
                d: dec!(-0.123),
                e: NaiveDate::from_ymd(2001, 7, 5),
                f: NaiveDate::from_ymd(1967, 8, 11).and_hms(23, 45, 1),
                g: NaiveTime::from_hms(0, 1, 2),
                h: None,
                i: 0.2,
                j: 0.2,
            },
        ];

        let mut stmt = conn.prepare("select * from foo").unwrap();
        for (i, row) in stmt.query(()).unwrap().enumerate() {
            let foo = Foo {
                a: row.get(0).unwrap(),
                b: row.get(1).unwrap(),
                c: row.get(2).unwrap(),
                d: row.get(3).unwrap(),
                e: row.get(4).unwrap(),
                f: row.get(5).unwrap(),
                g: row.get(6).unwrap(),
                h: row.get(7).unwrap(),
                i: row.get(8).unwrap(),
                j: row.get(9).unwrap(),
            };
            assert_eq!(foo, expects[i]);
        }

        let foo_iter = stmt
            .query_map((), |row| {
                Ok(Foo {
                    a: row.get(0).unwrap(),
                    b: row.get(1).unwrap(),
                    c: row.get(2).unwrap(),
                    d: row.get(3).unwrap(),
                    e: row.get(4).unwrap(),
                    f: row.get(5).unwrap(),
                    g: row.get(6).unwrap(),
                    h: row.get(7).unwrap(),
                    i: row.get(8).unwrap(),
                    j: row.get(9).unwrap(),
                })
            })
            .unwrap();
        for (i, foo) in foo_iter.enumerate() {
            assert_eq!(foo.unwrap(), expects[i]);
        }
    }

    {
        let expects: [Foo; 1] = [Foo {
            a: 2,
            b: "A".to_string(),
            c: "B".to_string(),
            d: dec!(-0.123),
            e: NaiveDate::from_ymd(1999, 1, 25),
            f: NaiveDate::from_ymd(1967, 8, 11).and_hms(23, 45, 1),
            g: NaiveTime::from_hms(0, 0, 1),
            h: None,
            i: 0.1,
            j: 0.1,
        }];

        let mut stmt = conn.prepare("select * from foo where a=?").unwrap();
        for (i, row) in stmt.query((2,)).unwrap().enumerate() {
            let foo = Foo {
                a: row.get(0).unwrap(),
                b: row.get(1).unwrap(),
                c: row.get(2).unwrap(),
                d: row.get(3).unwrap(),
                e: row.get(4).unwrap(),
                f: row.get(5).unwrap(),
                g: row.get(6).unwrap(),
                h: row.get(7).unwrap(),
                i: row.get(8).unwrap(),
                j: row.get(9).unwrap(),
            };
            assert_eq!(foo, expects[i]);
        }

        let foo_iter = stmt
            .query_map((2,), |row| {
                Ok(Foo {
                    a: row.get(0).unwrap(),
                    b: row.get(1).unwrap(),
                    c: row.get(2).unwrap(),
                    d: row.get(3).unwrap(),
                    e: row.get(4).unwrap(),
                    f: row.get(5).unwrap(),
                    g: row.get(6).unwrap(),
                    h: row.get(7).unwrap(),
                    i: row.get(8).unwrap(),
                    j: row.get(9).unwrap(),
                })
            })
            .unwrap();
        for (i, foo) in foo_iter.enumerate() {
            assert_eq!(foo.unwrap(), expects[i]);
        }
    }

    // Transction
    let mut conn = Connection::connect(&conn_string).unwrap();
    let expects: [Foo; 1] = [Foo {
        a: 2,
        b: "A".to_string(),
        c: "B".to_string(),
        d: dec!(-0.123),
        e: NaiveDate::from_ymd(1999, 1, 25),
        f: NaiveDate::from_ymd(1967, 8, 11).and_hms(23, 45, 1),
        g: NaiveTime::from_hms(0, 0, 1),
        h: None,
        i: 0.1,
        j: 0.1,
    }];

    let mut trans = conn.transaction().unwrap();
    trans
        .execute("delete from foo where a in (1, 3)", ())
        .unwrap();

    let mut stmt = trans.prepare("select * from foo").unwrap();
    let foo_iter = stmt
        .query_map((), |row| {
            Ok(Foo {
                a: row.get(0).unwrap(),
                b: row.get(1).unwrap(),
                c: row.get(2).unwrap(),
                d: row.get(3).unwrap(),
                e: row.get(4).unwrap(),
                f: row.get(5).unwrap(),
                g: row.get(6).unwrap(),
                h: row.get(7).unwrap(),
                i: row.get(8).unwrap(),
                j: row.get(9).unwrap(),
            })
        })
        .unwrap();
    for (i, foo) in foo_iter.enumerate() {
        assert_eq!(foo.unwrap(), expects[i]);
    }
}
