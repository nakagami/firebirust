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
use super::Connection;
use chrono::TimeZone;
use chrono::{NaiveDate, NaiveTime};
use chrono_tz;
use std::env;
use std::str::FromStr;
use urlencoding;

#[derive(PartialEq, Debug)]
struct TzTest {
    id: i32,
    t: (NaiveTime, chrono_tz::Tz),
    ts: chrono::DateTime<chrono_tz::Tz>,
}

#[test]
#[ignore]
fn test_timezone() {
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
        "firebird://{}:{}@localhost/tmp/rust-firebird-test-timezone.fdb?timezone=Asia/Tokyo",
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
        CREATE TABLE tz_test (
            id INTEGER NOT NULL,
            t TIME WITH TIME ZONE DEFAULT '12:34:56',
            ts TIMESTAMP WITH TIME ZONE DEFAULT '1967-08-11 23:45:01',
            PRIMARY KEY (id)
        )
    "#,
    )
    .unwrap();

    conn.execute("insert into tz_test (id) values (1)",[])
        .unwrap();
    conn.execute(
        "insert into tz_test (id, t, ts) values (2, '12:34:56 Asia/Seoul', '1967-08-11 23:45:01.0000 Asia/Seoul')",
        ()
    )
    .unwrap();
    conn.execute(
        "insert into tz_test (id, t, ts) values (3, '03:34:56 UTC', '1967-08-11 14:45:01.0000 UTC')",
        ()
    )
    .unwrap();

    let expects: [TzTest; 3] = [
        TzTest {
            id: 1,
            t: (
                NaiveTime::from_hms(12, 34, 56),
                chrono_tz::Tz::from_str("Asia/Tokyo").unwrap(),
            ),
            ts: chrono_tz::Tz::from_str("Asia/Tokyo")
                .unwrap()
                .from_local_datetime(&NaiveDate::from_ymd(1967, 8, 11).and_hms(23, 45, 1))
                .unwrap(),
        },
        TzTest {
            id: 2,
            t: (
                NaiveTime::from_hms(12, 34, 56),
                chrono_tz::Tz::from_str("Asia/Seoul").unwrap(),
            ),
            ts: chrono_tz::Tz::from_str("Asia/Seoul")
                .unwrap()
                .from_local_datetime(&NaiveDate::from_ymd(1967, 8, 11).and_hms(23, 45, 1))
                .unwrap(),
        },
        TzTest {
            id: 3,
            t: (
                NaiveTime::from_hms(3, 34, 56),
                chrono_tz::Tz::from_str("UTC").unwrap(),
            ),
            ts: chrono_tz::Tz::from_str("UTC")
                .unwrap()
                .from_local_datetime(&NaiveDate::from_ymd(1967, 8, 11).and_hms(14, 45, 1))
                .unwrap(),
        },
    ];

    let mut stmt = conn.prepare("select * from tz_test").unwrap();
    for (i, row) in stmt.query(()).unwrap().enumerate() {
        let r = TzTest {
            id: row.get(0).unwrap(),
            t: row.get(1).unwrap(),
            ts: row.get(2).unwrap(),
        };
        assert_eq!(r, expects[i]);
    }
}
