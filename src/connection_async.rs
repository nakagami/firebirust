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

use std::cell::RefCell;
use std::collections::HashMap;

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
    conn_params: ConnParams,
    conn_options: HashMap<String, String>,
}

impl ConnectionAsync {
    pub async fn connect(conn_string: &str) -> Result<ConnectionAsync, Error> {
        let (conn_params, conn_options) = ConnParams::from_url(conn_string)?;
        let mut wp = WireProtocolAsync::new(&conn_params, &conn_options).await?;
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

        wp.op_attach(
            &conn_params.db_name,
            &conn_params.username,
            &conn_params.password,
            &conn_options["role"],
        )
        .await?;
        let (db_handle, _, _) = wp.op_response().await?;
        wp.db_handle = db_handle;

        wp.op_transaction(true).await?;
        let (trans_handle, _, _) = wp.op_response().await?;

        Ok(ConnectionAsync {
            wp: RefCell::new(wp),
            trans_handle,
            conn_params,
            conn_options,
        })
    }

    pub async fn create_database(conn_string: &str) -> Result<ConnectionAsync, Error> {
        let (conn_params, conn_options) = ConnParams::from_url(conn_string)?;
        let mut wp = WireProtocolAsync::new(&conn_params, &conn_options).await?;
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
            conn_params,
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

    pub async fn _prepare(
        &mut self,
        query: &str,
        trans_handle: i32,
    ) -> Result<StatementAsync<'_>, Error> {
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

        Ok(StatementAsync::new(
            self,
            trans_handle,
            stmt_handle,
            stmt_type,
            xsqlda,
            true, // autocommit is true
        ))
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
