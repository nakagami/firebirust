# firebirust

firebirust is a database driver for Firebird RDBMS https://firebirdsql.org/ written in rust.

It attempts to expose an interface similar to Rusqlite https://github.com/rusqlite/rusqlite.


## Supported Firebird

Firebird 3.0+ is supported

## Add dependency

Add `firebirust` to dependencies section of Cargo.toml
```
[dependencies]
firebirust = “0.5”
```

## code example

Database connection
```
use firebirust::Connection;

let mut conn =
    Connection::connect("firebird://SYSDBA:masterkey@localhost/tmp/rust-firebird-test.fdb")
        .unwrap();
```

Execute SQL statement
```
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
```

Execute SQL statement with parameter
```
conn.execute(
    "insert into foo(a, b, c, h) values (?, ?, ?, ?)",
    (1, "a", "b", "This is a pen"),
)
.unwrap();

conn.execute(
    "insert into foo(a, b, c, e, g, i, j) values (2, 'A', 'B', '1999-01-25', '00:00:01', 0.1, 0.1)",
    (), // empty list of parameters.
)
.unwrap();

conn.commit()
```

Execute Query and get results
```
let mut stmt = conn.prepare("select * from foo").unwrap();
for row in stmt.query(()).unwrap() {
    let a:i32 = row.get(0).unwrap();
    println!("a={}", a);
}
```

Execute Query and map
```
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rust_decimal::Decimal;

#[derive(Debug)]
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

let mut stmt = conn.prepare("select * from foo where a=?").unwrap();
let foo_iter = stmt
    .query_map((1,), |row| {
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

for foo in foo_iter {
    println!("{:?}", foo);
}
```

Execute with transaction
```
let mut trans = conn.transaction().unwrap();
trans.execute(
    "delete from foo where a in (1, 3)", ())
.unwrap();
trans.commit()
```

## Experimental implementation of asynchronous/standby

Async/await is possible by using ConnectionAsync instead of Connection since version 0.5.

https://github.com/nakagami/firebirust/blob/master/src/test_async.rs
