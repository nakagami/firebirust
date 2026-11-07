// MIT License
//
// Copyright (c) 2021-2024 Hajime Nakagami<nakagami@gmail.com>
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

#![allow(dead_code)]

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use super::array::{array_type_from_blr, ArrayDimension, ArrayMeta};
use super::cellvalue::CellValue;
use super::conn_params::ConnParams;
use super::error::Error;
use super::params::Params;
use super::statement_async::StatementAsync;
use super::transaction_async::*;
use super::wireprotocol_async::*;
use super::xsqlvar::XSQLVar;
use super::*;

pub struct ConnectionAsync {
    wp: RefCell<WireProtocolAsync>,
    trans_handle: i32, // transaction for operating from connection methods
    resolving_array_meta: Cell<bool>,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub db_name: String,
    conn_options: HashMap<String, String>,
}

impl ConnectionAsync {
    pub async fn connect(
        host: &str,
        port: u16,
        db_name: &str,
        username: &str,
        password: &str,
        conn_options: &HashMap<String, String>,
    ) -> Result<ConnectionAsync, Error> {
        let mut wp = WireProtocolAsync::new(host, port, &conn_options["timezone"]).await?;
        let (client_public, client_secret) = srp::get_client_seed();
        wp.op_connect(db_name, username, password, &conn_options, &client_public)
            .await?;
        wp.parse_connect_response(
            username,
            password,
            &conn_options,
            &client_public,
            &client_secret,
        )
        .await?;

        wp.op_attach(db_name, username, password, &conn_options["role"])
            .await?;
        let (db_handle, _, _) = wp.op_response().await?;
        wp.db_handle = db_handle;

        wp.op_transaction(true).await?;
        let (trans_handle, _, _) = wp.op_response().await?;

        Ok(ConnectionAsync {
            wp: RefCell::new(wp),
            trans_handle,
            resolving_array_meta: Cell::new(false),
            host: host.to_string(),
            port,
            username: username.to_string(),
            password: password.to_string(),
            db_name: db_name.to_string(),
            conn_options: conn_options.clone(),
        })
    }

    pub async fn connect_url(conn_string: &str) -> Result<ConnectionAsync, Error> {
        let (conn_params, conn_options) = ConnParams::from_url(conn_string)?;
        ConnectionAsync::connect(
            &conn_params.host,
            conn_params.port,
            &conn_params.db_name,
            &conn_params.username,
            &conn_params.password,
            &conn_options,
        )
        .await
    }

    pub async fn create_database_url(conn_string: &str) -> Result<ConnectionAsync, Error> {
        let (conn_params, conn_options) = ConnParams::from_url(conn_string)?;
        let mut wp = WireProtocolAsync::new(
            &conn_params.host,
            conn_params.port,
            &conn_options["timezone"],
        )
        .await?;

        let (client_public, client_secret) = srp::get_client_seed();
        wp.op_connect(
            &conn_params.db_name,
            &conn_params.username,
            &conn_params.password,
            &conn_options,
            &client_public,
        )
        .await?;
        wp.parse_connect_response(
            &conn_params.username,
            &conn_params.password,
            &conn_options,
            &client_public,
            &client_secret,
        )
        .await?;

        let page_size: u32 = conn_options["page_size"].parse().unwrap();

        wp.op_create(
            &conn_params.db_name,
            &conn_params.username,
            &conn_params.password,
            &conn_options["role"],
            page_size,
        )
        .await?;
        let (db_handle, _, _) = wp.op_response().await?;
        wp.db_handle = db_handle;

        wp.op_transaction(true).await?;
        let (trans_handle, _, _) = wp.op_response().await?;

        Ok(ConnectionAsync {
            wp: RefCell::new(wp),
            trans_handle,
            resolving_array_meta: Cell::new(false),
            host: conn_params.host,
            port: conn_params.port,
            username: conn_params.username,
            password: conn_params.password,
            db_name: conn_params.db_name,
            conn_options,
        })
    }

    pub(crate) async fn _execute_batch(
        &mut self,
        query: &str,
        trans_handle: i32,
    ) -> Result<(), Error> {
        let mut wp = self.wp.borrow_mut();
        wp.op_exec_immediate(trans_handle, query).await?;
        wp.op_response().await?;

        // commit automatically
        wp.op_commit_retaining(trans_handle).await?;
        wp.op_response().await?;

        Ok(())
    }

    pub async fn execute_batch(&mut self, query: &str) -> Result<(), Error> {
        self._execute_batch(query, self.trans_handle).await
    }

    pub(crate) async fn _execute<P: Params>(
        &mut self,
        query: &str,
        params: P,
        trans_handle: i32,
    ) -> Result<(), Error> {
        let mut stmt = {
            let mut wp = self.wp.borrow_mut();
            wp.op_allocate_statement().await?;

            let mut stmt_handle = if (wp.accept_type & PTYPE_MASK) == PTYPE_LAZY_SEND {
                wp.lazy_response_count += 1;
                -1
            } else {
                let (stmt_handle, _, _) = wp.op_response().await?;
                stmt_handle
            };

            wp.op_prepare_statement(stmt_handle, trans_handle, query)
                .await?;
            if (wp.accept_type & PTYPE_MASK) == PTYPE_LAZY_SEND && wp.lazy_response_count > 0 {
                wp.lazy_response_count -= 1;
                let (h, _, _) = wp.op_response().await?;
                stmt_handle = h;
            }
            let (_, buf, _) = wp.op_response().await?;
            let (stmt_type, xsqlda) = wp.parse_xsqlda(&buf, stmt_handle).await?;

            StatementAsync::new(self, trans_handle, stmt_handle, stmt_type, xsqlda, true)
        };

        stmt.execute(params).await?;

        Ok(())
    }

    pub async fn execute<P: Params>(&mut self, query: &str, params: P) -> Result<(), Error> {
        self._execute(query, params, self.trans_handle).await
    }

    pub(crate) async fn _commit(&self, trans_handle: i32) -> Result<(), Error> {
        let mut wp = self.wp.borrow_mut();
        wp.op_commit_retaining(trans_handle).await?;
        wp.op_response().await?;
        Ok(())
    }

    pub async fn commit(&self) -> Result<(), Error> {
        self._commit(self.trans_handle).await
    }

    pub(crate) async fn _begin_trans(&mut self) -> Result<i32, Error> {
        let mut wp = self.wp.borrow_mut();
        wp.op_transaction(false).await?;
        let (trans_handle, _, _) = wp.op_response().await?;
        Ok(trans_handle)
    }

    pub(crate) async fn _rollback(&mut self, trans_handle: i32) -> Result<(), Error> {
        let mut wp = self.wp.borrow_mut();
        wp.op_rollback_retaining(trans_handle).await?;
        wp.op_response().await?;
        Ok(())
    }

    pub async fn rollback(&mut self) -> Result<(), Error> {
        self._rollback(self.trans_handle).await
    }

    async fn _prepare_raw(
        &mut self,
        query: &str,
        trans_handle: i32,
    ) -> Result<(i32, u32, Vec<XSQLVar>), Error> {
        let mut wp = self.wp.borrow_mut();
        wp.op_allocate_statement().await?;

        let mut stmt_handle = if (wp.accept_type & PTYPE_MASK) == PTYPE_LAZY_SEND {
            wp.lazy_response_count += 1;
            -1
        } else {
            let (stmt_handle, _, _) = wp.op_response().await?;
            stmt_handle
        };

        wp.op_prepare_statement(stmt_handle, trans_handle, query)
            .await?;
        if (wp.accept_type & PTYPE_MASK) == PTYPE_LAZY_SEND && wp.lazy_response_count > 0 {
            wp.lazy_response_count -= 1;
            let (h, _, _) = wp.op_response().await?;
            stmt_handle = h;
        }
        let (_, _, buf) = wp.op_response().await?;
        let (stmt_type, xsqlda) = wp.parse_xsqlda(&buf, stmt_handle).await?;
        Ok((stmt_handle, stmt_type, xsqlda))
    }

    async fn _prepare_simple(
        &mut self,
        query: &str,
        trans_handle: i32,
    ) -> Result<StatementAsync<'_>, Error> {
        let (stmt_handle, stmt_type, xsqlda) = self._prepare_raw(query, trans_handle).await?;
        Ok(StatementAsync::new(
            self,
            trans_handle,
            stmt_handle,
            stmt_type,
            xsqlda,
            true,
        ))
    }

    pub async fn _prepare(
        &mut self,
        query: &str,
        trans_handle: i32,
    ) -> Result<StatementAsync<'_>, Error> {
        let (stmt_handle, stmt_type, mut xsqlda) = self._prepare_raw(query, trans_handle).await?;
        self._resolve_array_meta(trans_handle, &mut xsqlda).await?;

        Ok(StatementAsync::new(
            self,
            trans_handle,
            stmt_handle,
            stmt_type,
            xsqlda,
            true, // autocommit is true
        ))
    }

    pub(crate) async fn _resolve_array_meta(
        &mut self,
        trans_handle: i32,
        xsqlda: &mut [XSQLVar],
    ) -> Result<(), Error> {
        if self.resolving_array_meta.get() {
            return Ok(());
        }
        let has_array = xsqlda.iter().any(|x| x.sqltype == SQL_TYPE_ARRAY);
        if !has_array {
            return Ok(());
        }

        self.resolving_array_meta.set(true);
        let res = self._do_resolve_array_meta(trans_handle, xsqlda).await;
        self.resolving_array_meta.set(false);
        res
    }

    async fn _do_resolve_array_meta(
        &mut self,
        trans_handle: i32,
        xsqlda: &mut [XSQLVar],
    ) -> Result<(), Error> {
        for x in xsqlda.iter_mut() {
            if x.sqltype != SQL_TYPE_ARRAY || x.relname.is_empty() || x.fieldname.is_empty() {
                continue;
            }
            let table_name = x.relname.trim().to_string();
            let field_name = x.fieldname.trim().to_string();

            let query = format!(
                "SELECT \
                 Y.RDB$FIELD_TYPE, \
                 Y.RDB$FIELD_SCALE, \
                 Y.RDB$FIELD_LENGTH, \
                 CASE WHEN Y.RDB$FIELD_TYPE IN (14, 37) THEN Y.RDB$CHARACTER_SET_ID ELSE Y.RDB$FIELD_SUB_TYPE END, \
                 CASE WHEN Y.RDB$FIELD_TYPE IN (14, 37) THEN Y.RDB$CHARACTER_LENGTH ELSE Y.RDB$FIELD_LENGTH END, \
                 X.RDB$FIELD_SOURCE \
                 FROM RDB$RELATION_FIELDS X, RDB$FIELDS Y \
                 WHERE X.RDB$FIELD_SOURCE = Y.RDB$FIELD_NAME \
                   AND X.RDB$RELATION_NAME = '{}' \
                   AND X.RDB$FIELD_NAME = '{}'",
                table_name.replace('\'', "''"),
                field_name.replace('\'', "''")
            );

            let row_opt = {
                let mut stmt = self._prepare_simple(&query, trans_handle).await?;
                let mut qres = stmt.query(()).await?;
                qres.next()
            };

            let row = match row_opt {
                Some(r) => r,
                None => continue,
            };

            let blr_type_id: i64 = row.get(0)?;
            let scale: i64 = row.get(1)?;
            let field_bytes: i64 = row.get(2)?;
            let sub_type: i64 = row.get::<Option<i64>>(3)?.unwrap_or(0);
            let length: i64 = row.get::<Option<i64>>(4)?.unwrap_or(field_bytes);
            let field_source: String = row.get(5)?;
            let field_source = field_source.trim().to_string();

            let dim_query = format!(
                "SELECT X.RDB$LOWER_BOUND, X.RDB$UPPER_BOUND \
                 FROM RDB$FIELD_DIMENSIONS X \
                 WHERE X.RDB$FIELD_NAME = '{}' \
                 ORDER BY X.RDB$DIMENSION ASC",
                field_source.replace('\'', "''")
            );

            let mut dims = Vec::new();
            {
                let mut dim_stmt = self._prepare_simple(&dim_query, trans_handle).await?;
                let mut dim_qres = dim_stmt.query(()).await?;
                while let Some(r) = dim_qres.next() {
                    let lower: i64 = r.get(0)?;
                    let upper: i64 = r.get(1)?;
                    let length = if upper >= lower {
                        (upper - lower + 1) as usize
                    } else {
                        0
                    };
                    dims.push(ArrayDimension {
                        lower_bound: lower as i32,
                        length,
                    });
                }
            }

            let type_id = array_type_from_blr(blr_type_id)?;
            x.array_meta = Some(ArrayMeta {
                table_name,
                field_name,
                type_id,
                blr_type_id: blr_type_id as u8,
                scale: scale as i32,
                length: length as u32,
                field_bytes: field_bytes as u32,
                sub_type: sub_type as i32,
                field_source,
                dimensions: dims,
            });
        }
        Ok(())
    }

    pub(crate) async fn _get_array_slice(
        &self,
        trans_handle: i32,
        array_id: &[u8],
        meta: &ArrayMeta,
    ) -> Result<CellValue, Error> {
        let mut wp = self.wp.borrow_mut();
        wp.get_array_slice(trans_handle, array_id, meta).await
    }

    pub async fn put_array_slice(
        &mut self,
        meta: &ArrayMeta,
        data: &[u8],
        element_count: usize,
    ) -> Result<Vec<u8>, Error> {
        let mut wp = self.wp.borrow_mut();
        wp.put_array_slice(self.trans_handle, meta, data, element_count).await
    }

    pub async fn resolve_array_meta(
        &mut self,
        table_name: &str,
        field_name: &str,
    ) -> Result<Option<ArrayMeta>, Error> {
        let mut var = XSQLVar {
            sqltype: SQL_TYPE_ARRAY,
            relname: table_name.to_string(),
            fieldname: field_name.to_string(),
            ..Default::default()
        };
        self._resolve_array_meta(self.trans_handle, std::slice::from_mut(&mut var)).await?;
        Ok(var.array_meta)
    }

    pub async fn prepare(&mut self, query: &str) -> Result<StatementAsync<'_>, Error> {
        self._prepare(query, self.trans_handle).await
    }

    pub async fn transaction(&mut self) -> Result<TransactionAsync<'_>, Error> {
        TransactionAsync::new(self).await
    }

    // methods for Statement

    pub(crate) async fn _execute_statement(
        &self,
        trans_handle: i32,
        stmt_handle: i32,
        stmt_type: u32,
        params: &[(Vec<u8>, Vec<u8>, bool)],
    ) -> Result<usize, Error> {
        let mut wp = self.wp.borrow_mut();
        wp.op_execute(stmt_handle, trans_handle, params).await?;
        wp.op_response().await?;
        Ok(wp.rowcount(stmt_handle, stmt_type).await?)
    }

    pub(crate) async fn _fetch(
        &self,
        stmt_handle: i32,
        blr: &Vec<u8>,
        xsqlda: &[XSQLVar],
    ) -> Result<(Vec<Vec<CellValue>>, bool), Error> {
        let mut wp = self.wp.borrow_mut();
        wp.op_fetch(stmt_handle, &blr).await?;
        wp.op_fetch_response(xsqlda).await
    }

    pub(crate) async fn _get_blob_segments(
        &self,
        blob_id: &Vec<u8>,
        trans_handle: i32,
    ) -> Result<Vec<u8>, Error> {
        let mut wp = self.wp.borrow_mut();
        wp.get_blob_segments(blob_id, trans_handle).await
    }

    pub(crate) async fn _free_statement(&self, stmt_handle: i32, drop_type: i32) -> () {
        let mut wp = self.wp.borrow_mut();
        wp.op_free_statement(stmt_handle, drop_type).await.unwrap();
        if (wp.accept_type & PTYPE_MASK) == PTYPE_LAZY_SEND {
            wp.lazy_response_count += 1;
        } else {
            wp.op_response().await.unwrap();
        }
    }

    // methods for Transaction
    pub(crate) async fn drop_transaction(&self, trans_handle: i32) -> () {
        let mut wp = self.wp.borrow_mut();
        wp.op_rollback(trans_handle).await.unwrap();
        wp.op_response().await.unwrap();
    }
}
