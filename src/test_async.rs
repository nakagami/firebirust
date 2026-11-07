// MIT License
//
// Copyright (c) 2020-2024 Hajime Nakagami<nakagami@gmail.com>
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
use super::*;
use smol;
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

async fn test_connnect_async() {
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
        "firebird://{}:{}@localhost{}/rust-firebird-test-async.fdb",
        &user,
        urlencoding::encode(&password),
        env::temp_dir().display(),
    );

    match ConnectionAsync::create_database_url(&conn_string).await {
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
            c CHAR(1024),
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
    .await
    .unwrap();

    conn.execute(
        "insert into foo(a, b, c, h) values (?, ?, ?, ?)",
        (1, "a", "b", "This is a pen"),
    )
    .await
    .unwrap();

    conn.execute(
        "insert into foo(a, b, c, e, g, i, j) values (?, 'A', 'B', '1999-01-25', '00:00:01', 0.1, 0.1)",
        (2, )
    ).await
    .unwrap();
    conn.execute("insert into foo(a, b, c, e, g, i, j) values (3, 'X', 'Y', '2001-07-05', '00:01:02', 0.2, 0.2)", ()).await.unwrap();

    {
        let expects: [Foo; 3] = [
            Foo {
                a: 1,
                b: "a".to_string(),
                c: "b".to_string(),
                d: dec!(-0.123),
                e: NaiveDate::from_ymd_opt(1967, 8, 11).unwrap(),
                f: NaiveDate::from_ymd_opt(1967, 8, 11)
                    .unwrap()
                    .and_hms_opt(23, 45, 1)
                    .unwrap(),
                g: NaiveTime::from_hms_opt(23, 45, 1).unwrap(),
                h: Some("This is a pen".to_string().into_bytes()),
                i: 0.0,
                j: 0.0,
            },
            Foo {
                a: 2,
                b: "A".to_string(),
                c: "B".to_string(),
                d: dec!(-0.123),
                e: NaiveDate::from_ymd_opt(1999, 1, 25).unwrap(),
                f: NaiveDate::from_ymd_opt(1967, 8, 11)
                    .unwrap()
                    .and_hms_opt(23, 45, 1)
                    .unwrap(),
                g: NaiveTime::from_hms_opt(0, 0, 1).unwrap(),
                h: None,
                i: 0.1,
                j: 0.1,
            },
            Foo {
                a: 3,
                b: "X".to_string(),
                c: "Y".to_string(),
                d: dec!(-0.123),
                e: NaiveDate::from_ymd_opt(2001, 7, 5).unwrap(),
                f: NaiveDate::from_ymd_opt(1967, 8, 11)
                    .unwrap()
                    .and_hms_opt(23, 45, 1)
                    .unwrap(),
                g: NaiveTime::from_hms_opt(0, 1, 2).unwrap(),
                h: None,
                i: 0.2,
                j: 0.2,
            },
        ];

        let mut stmt = conn.prepare("select * from foo").await.unwrap();

        assert_eq!(stmt.column_count(), 10);
        assert_eq!(
            stmt.column_names(),
            vec!["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"]
        );
        assert_eq!(
            stmt.column_metadata(0),
            (Some((SQL_TYPE_LONG, 0, 0, 4, false, "A", "FOO", "SYSDBA")))
        );
        assert_eq!(
            stmt.column_metadata(1),
            (Some((SQL_TYPE_VARYING, 0, 4, 120, false, "B", "FOO", "SYSDBA")))
        );

        for (i, row) in stmt.query(()).await.unwrap().enumerate() {
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
            .await
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
            e: NaiveDate::from_ymd_opt(1999, 1, 25).unwrap(),
            f: NaiveDate::from_ymd_opt(1967, 8, 11)
                .unwrap()
                .and_hms_opt(23, 45, 1)
                .unwrap(),
            g: NaiveTime::from_hms_opt(0, 0, 1).unwrap(),
            h: None,
            i: 0.1,
            j: 0.1,
        }];

        let mut stmt = conn.prepare("select * from foo where a=?").await.unwrap();
        for (i, row) in stmt.query((2,)).await.unwrap().enumerate() {
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
            .await
            .unwrap();
        for (i, foo) in foo_iter.enumerate() {
            assert_eq!(foo.unwrap(), expects[i]);
        }
    }

    // Transction
    let mut conn = ConnectionAsync::connect_url(&conn_string).await.unwrap();
    let expects: [Foo; 1] = [Foo {
        a: 2,
        b: "A".to_string(),
        c: "B".to_string(),
        d: dec!(-0.123),
        e: NaiveDate::from_ymd_opt(1999, 1, 25).unwrap(),
        f: NaiveDate::from_ymd_opt(1967, 8, 11)
            .unwrap()
            .and_hms_opt(23, 45, 1)
            .unwrap(),
        g: NaiveTime::from_hms_opt(0, 0, 1).unwrap(),
        h: None,
        i: 0.1,
        j: 0.1,
    }];

    let mut trans = conn.transaction().await.unwrap();
    trans
        .execute("delete from foo where a in (1, 3)", ())
        .await
        .unwrap();

    let mut stmt = trans.prepare("select * from foo").await.unwrap();
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
        .await
        .unwrap();
    for (i, foo) in foo_iter.enumerate() {
        assert_eq!(foo.unwrap(), expects[i]);
    }
}

#[test]
fn test_connnect() {
    smol::block_on(test_connnect_async())
}

async fn test_select_untyped_null_async() {
    let user = match env::var("ISC_USER") {
        Ok(val) => val,
        Err(_) => "sysdba".to_string(),
    };
    let password = match env::var("ISC_PASSWORD") {
        Ok(val) => val,
        Err(_) => "masterkey".to_string(),
    };

    let conn_string = format!(
        "firebird://{}:{}@localhost{}/rust-firebird-test-async-null.fdb",
        &user,
        urlencoding::encode(&password),
        env::temp_dir().display(),
    );

    let mut conn = match ConnectionAsync::create_database_url(&conn_string).await {
        Ok(c) => c,
        Err(err) => {
            println!("{:#?}", err);
            panic!("Can't connect");
        }
    };

    let mut stmt = conn.prepare("SELECT NULL, 42 FROM RDB$DATABASE").await.unwrap();
    let mut rows = stmt.query(()).await.unwrap();
    let row = rows.next().unwrap();
    let null_val: Option<i32> = row.get(0).unwrap();
    let num_val: i32 = row.get(1).unwrap();
    assert_eq!(null_val, None);
    assert_eq!(num_val, 42);
}

#[test]
fn test_select_untyped_null() {
    smol::block_on(test_select_untyped_null_async())
}

async fn test_numeric_scaled_async() {
    let user = match env::var("ISC_USER") {
        Ok(val) => val,
        Err(_) => "sysdba".to_string(),
    };
    let password = match env::var("ISC_PASSWORD") {
        Ok(val) => val,
        Err(_) => "masterkey".to_string(),
    };

    let conn_string = format!(
        "firebird://{}:{}@localhost{}/rust-firebird-test-async-scaled.fdb",
        &user,
        urlencoding::encode(&password),
        env::temp_dir().display(),
    );

    let mut conn = match ConnectionAsync::create_database_url(&conn_string).await {
        Ok(c) => c,
        Err(err) => {
            println!("{:#?}", err);
            panic!("Can't connect");
        }
    };

    conn.execute_batch(
        r#"
        CREATE TABLE test_async_scaled (
            s NUMERIC(4, 2),
            l NUMERIC(9, 3),
            b NUMERIC(18, 4)
        )
    "#,
    )
    .await
    .unwrap();

    conn.execute(
        "INSERT INTO test_async_scaled (s, l, b) VALUES (12.34, 123456.789, 12345678901234.5678)",
        (),
    )
    .await
    .unwrap();

    let mut stmt = conn.prepare("SELECT s, l, b FROM test_async_scaled").await.unwrap();
    let mut rows = stmt.query(()).await.unwrap();
    let row = rows.next().unwrap();
    let s: Decimal = row.get(0).unwrap();
    let l: Decimal = row.get(1).unwrap();
    let b: Decimal = row.get(2).unwrap();
    assert_eq!(s, dec!(12.34));
    assert_eq!(l, dec!(123456.789));
    assert_eq!(b, dec!(12345678901234.5678));
}

#[test]
fn test_numeric_scaled() {
    smol::block_on(test_numeric_scaled_async())
}

async fn test_array_select_async() {
    let user = match env::var("ISC_USER") {
        Ok(val) => val,
        Err(_) => "sysdba".to_string(),
    };
    let password = match env::var("ISC_PASSWORD") {
        Ok(val) => val,
        Err(_) => "masterkey".to_string(),
    };

    let conn_string = format!(
        "firebird://{}:{}@localhost{}/rust-firebird-test-array-async.fdb",
        &user,
        urlencoding::encode(&password),
        env::temp_dir().display(),
    );

    let mut conn = match ConnectionAsync::create_database_url(&conn_string).await {
        Ok(c) => c,
        Err(err) => {
            println!("{:#?}", err);
            panic!("Can't connect");
        }
    };

    conn.execute_batch(
        r#"
        CREATE TABLE test_arr_async (
            id INTEGER NOT NULL PRIMARY KEY,
            arr_int INTEGER [1:3],
            arr_str VARCHAR(10) [1:2],
            arr_dbl DOUBLE PRECISION [1:2],
            arr_2d INTEGER [1:2, 1:3]
        )
    "#,
    )
    .await
    .unwrap();

    let meta_int = conn.resolve_array_meta("TEST_ARR_ASYNC", "ARR_INT").await.unwrap().unwrap();
    let meta_str = conn.resolve_array_meta("TEST_ARR_ASYNC", "ARR_STR").await.unwrap().unwrap();
    let meta_dbl = conn.resolve_array_meta("TEST_ARR_ASYNC", "ARR_DBL").await.unwrap().unwrap();
    let meta_2d = conn.resolve_array_meta("TEST_ARR_ASYNC", "ARR_2D").await.unwrap().unwrap();

    let mut data_int = Vec::new();
    for v in [100i32, 200, 300] {
        data_int.extend_from_slice(&v.to_be_bytes());
    }
    let oid_int = conn.put_array_slice(&meta_int, &data_int, 3).await.unwrap();

    let mut data_str = Vec::new();
    for s in ["async", "firebird"] {
        data_str.extend_from_slice(&(s.len() as u32).to_be_bytes());
        data_str.extend_from_slice(s.as_bytes());
        let pad = (4 - (s.len() % 4)) % 4;
        data_str.extend(std::iter::repeat(0).take(pad));
    }
    let oid_str = conn.put_array_slice(&meta_str, &data_str, 2).await.unwrap();

    let mut data_dbl = Vec::new();
    for v in [2.5f64, 4.75f64] {
        data_dbl.extend_from_slice(&v.to_be_bytes());
    }
    let oid_dbl = conn.put_array_slice(&meta_dbl, &data_dbl, 2).await.unwrap();

    let mut data_2d = Vec::new();
    for v in [10i32, 20, 30, 40, 50, 60] {
        data_2d.extend_from_slice(&v.to_be_bytes());
    }
    let oid_2d = conn.put_array_slice(&meta_2d, &data_2d, 6).await.unwrap();

    let oid_int_arr: [u8; 8] = oid_int.as_slice().try_into().unwrap();
    let oid_str_arr: [u8; 8] = oid_str.as_slice().try_into().unwrap();
    let oid_dbl_arr: [u8; 8] = oid_dbl.as_slice().try_into().unwrap();
    let oid_2d_arr: [u8; 8] = oid_2d.as_slice().try_into().unwrap();

    conn.execute(
        "INSERT INTO test_arr_async (id, arr_int, arr_str, arr_dbl, arr_2d) VALUES (1, ?, ?, ?, ?)",
        (
            Param::Quad(oid_int_arr),
            Param::Quad(oid_str_arr),
            Param::Quad(oid_dbl_arr),
            Param::Quad(oid_2d_arr),
        ),
    )
    .await
    .unwrap();

    conn.execute(
        "INSERT INTO test_arr_async (id, arr_int, arr_str, arr_dbl, arr_2d) VALUES (2, NULL, NULL, NULL, NULL)",
        (),
    )
    .await
    .unwrap();

    let mut stmt = conn
        .prepare("SELECT id, arr_int, arr_str, arr_dbl, arr_2d FROM test_arr_async ORDER BY id")
        .await
        .unwrap();
    let mut rows = stmt.query(()).await.unwrap();

    // Row 1
    let row1 = rows.next().unwrap();
    let id1: i32 = row1.get(0).unwrap();
    let arr_int: Vec<i32> = row1.get(1).unwrap();
    let arr_str: Vec<String> = row1.get(2).unwrap();
    let arr_dbl: Vec<f64> = row1.get(3).unwrap();
    let arr_2d: CellValue = row1.get(4).unwrap();

    assert_eq!(id1, 1);
    assert_eq!(arr_int, vec![100, 200, 300]);
    assert_eq!(
        arr_str,
        vec!["async".to_string(), "firebird".to_string()]
    );
    assert_eq!(arr_dbl, vec![2.5, 4.75]);
    assert_eq!(
        arr_2d,
        CellValue::Array(vec![
            CellValue::Array(vec![
                CellValue::Long(10),
                CellValue::Long(20),
                CellValue::Long(30)
            ]),
            CellValue::Array(vec![
                CellValue::Long(40),
                CellValue::Long(50),
                CellValue::Long(60)
            ]),
        ])
    );

    // Row 2 (NULLs)
    let row2 = rows.next().unwrap();
    let id2: i32 = row2.get(0).unwrap();
    let arr_int_null: Option<Vec<i32>> = row2.get(1).unwrap();
    let arr_str_null: Option<Vec<String>> = row2.get(2).unwrap();

    assert_eq!(id2, 2);
    assert_eq!(arr_int_null, None);
    assert_eq!(arr_str_null, None);
}

#[test]
fn test_array_select_async_run() {
    smol::block_on(test_array_select_async())
}

async fn test_array_char_and_quoted_ident_async() {
    let user = match env::var("ISC_USER") {
        Ok(val) => val,
        Err(_) => "sysdba".to_string(),
    };
    let password = match env::var("ISC_PASSWORD") {
        Ok(val) => val,
        Err(_) => "masterkey".to_string(),
    };

    let conn_string = format!(
        "firebird://{}:{}@localhost{}/rust-firebird-test-array-quoted-async.fdb",
        &user,
        urlencoding::encode(&password),
        env::temp_dir().display(),
    );

    let mut conn = match ConnectionAsync::create_database_url(&conn_string).await {
        Ok(c) => c,
        Err(err) => {
            println!("{:#?}", err);
            panic!("Can't connect");
        }
    };

    conn.execute_batch(
        r#"
        CREATE TABLE "test_case_arr_async" (
            id INTEGER NOT NULL PRIMARY KEY,
            "char_Col" CHAR(5) [1:2]
        )
    "#,
    )
    .await
    .unwrap();

    let meta = conn
        .resolve_array_meta("test_case_arr_async", "char_Col")
        .await
        .unwrap()
        .unwrap();

    let mut data_char = Vec::new();
    for s in ["abc", "xy"] {
        let mut elem = format!("{:5}", s).into_bytes();
        let pad = (4 - (elem.len() % 4)) % 4;
        elem.extend(std::iter::repeat(0).take(pad));
        data_char.extend_from_slice(&elem);
    }
    let oid = conn.put_array_slice(&meta, &data_char, 2).await.unwrap();
    let oid_arr: [u8; 8] = oid.as_slice().try_into().unwrap();

    conn.execute(
        r#"INSERT INTO "test_case_arr_async" (id, "char_Col") VALUES (1, ?)"#,
        (Param::Quad(oid_arr),),
    )
    .await
    .unwrap();

    conn.execute(
        r#"INSERT INTO "test_case_arr_async" (id, "char_Col") VALUES (2, ?)"#,
        (Param::Quad([0u8; 8]),),
    )
    .await
    .unwrap();

    let mut stmt = conn
        .prepare(r#"SELECT id, "char_Col" FROM "test_case_arr_async" ORDER BY id"#)
        .await
        .unwrap();
    let mut rows = stmt.query(()).await.unwrap();

    let row1 = rows.next().unwrap();
    let id1: i32 = row1.get(0).unwrap();
    let arr1: Vec<String> = row1.get(1).unwrap();
    assert_eq!(id1, 1);
    assert_eq!(arr1, vec!["abc".to_string(), "xy".to_string()]);

    let row2 = rows.next().unwrap();
    let id2: i32 = row2.get(0).unwrap();
    let arr2: Vec<String> = row2.get(1).unwrap();
    assert_eq!(id2, 2);
    assert_eq!(arr2, Vec::<String>::new());
}

#[test]
fn test_array_char_and_quoted_ident_async_run() {
    smol::block_on(test_array_char_and_quoted_ident_async())
}


