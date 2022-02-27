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

#![allow(dead_code)]

use std::collections::HashMap;

use super::cellvalue::CellValue;
use super::conn_params::ConnParams;
use super::error::Error;
use super::param::Param;
use super::statement::Statement;
use super::transaction::*;
use super::wireprotocol::*;
use super::xsqlvar::XSQLVar;
use super::*;

pub struct Connection {
    wp: WireProtocol,
    trans_handle: i32, // transaction for operating from connection methods
    conn_params: ConnParams,
    conn_options: HashMap<String, String>,
}

impl Connection {
    pub fn connect(conn_string: &str) -> Result<Connection, Error> {
        let (conn_params, conn_options) = ConnParams::from_url(conn_string)?;
        let mut wp = WireProtocol::new(&conn_params, &conn_options)?;
        let (client_public, client_secret) = srp::get_client_seed();
        wp.op_connect(
            &conn_params.db_name,
            &conn_params.username,
            &conn_params.password,
            &conn_options,
            &client_public,
        )?;
        wp.parse_connect_response(
            &conn_params.username,
            &conn_params.password,
            &conn_options,
            &client_public,
            &client_secret,
        )?;

        wp.op_attach(
            &conn_params.db_name,
            &conn_params.username,
            &conn_params.password,
            &conn_options["role"],
        )?;
        let (db_handle, _, _) = wp.op_response()?;
        wp.db_handle = db_handle;

        wp.op_transaction(true)?;
        let (trans_handle, _, _) = wp.op_response()?;

        Ok(Connection {
            wp,
            trans_handle,
            conn_params,
            conn_options,
        })
    }

    pub fn create_database(conn_string: &str) -> Result<Connection, Error> {
        let (conn_params, conn_options) = ConnParams::from_url(conn_string)?;
        let mut wp = WireProtocol::new(&conn_params, &conn_options)?;
        let (client_public, client_secret) = srp::get_client_seed();
        wp.op_connect(
            &conn_params.db_name,
            &conn_params.username,
            &conn_params.password,
            &conn_options,
            &client_public,
        )?;
        wp.parse_connect_response(
            &conn_params.username,
            &conn_params.password,
            &conn_options,
            &client_public,
            &client_secret,
        )?;

        let page_size: u32 = conn_options["page_size"].parse().unwrap();

        wp.op_create(
            &conn_params.db_name,
            &conn_params.username,
            &conn_params.password,
            &conn_options["role"],
            page_size,
        )?;
        let (db_handle, _, _) = wp.op_response()?;
        wp.db_handle = db_handle;

        wp.op_transaction(true)?;
        let (trans_handle, _, _) = wp.op_response()?;

        Ok(Connection {
            wp,
            trans_handle,
            conn_params,
            conn_options,
        })
    }

    pub(crate) fn _execute_batch(&mut self, query: &str, trans_handle: i32) -> Result<(), Error> {
        self.wp.op_exec_immediate(trans_handle, query)?;
        self.wp.op_response()?;

        // commit automatically
        self.commit()?;

        Ok(())
    }

    pub fn execute_batch(&mut self, query: &str) -> Result<(), Error> {
        self._execute_batch(query, self.trans_handle)
    }

    pub(crate) fn _execute(
        &mut self,
        query: &str,
        params: Vec<Param>,
        trans_handle: i32,
    ) -> Result<(), Error> {
        self.wp.op_allocate_statement()?;

        let mut stmt_handle = if self.wp.accept_type == PTYPE_LAZY_SEND {
            self.wp.lazy_response_count += 1;
            -1
        } else {
            let (stmt_handle, _, _) = self.wp.op_response()?;
            stmt_handle
        };

        self.wp
            .op_prepare_statement(stmt_handle, trans_handle, query)?;
        if self.wp.accept_type == PTYPE_LAZY_SEND && self.wp.lazy_response_count > 0 {
            self.wp.lazy_response_count -= 1;
            let (h, _, _) = self.wp.op_response()?;
            stmt_handle = h;
        }
        let (_, buf, _) = self.wp.op_response()?;
        let (stmt_type, xsqlda) = self.wp.parse_xsqlda(&buf, stmt_handle)?;
        let mut stmt = Statement::new(self, trans_handle, stmt_handle, stmt_type, xsqlda, true);

        stmt.execute(params)?;

        Ok(())
    }

    pub fn execute(&mut self, query: &str, params: Vec<Param>) -> Result<(), Error> {
        self._execute(query, params, self.trans_handle)
    }

    pub(crate) fn _commit(&mut self, trans_handle: i32) -> Result<(), Error> {
        self.wp.op_commit_retaining(trans_handle)?;
        self.wp.op_response()?;
        Ok(())
    }

    pub fn commit(&mut self) -> Result<(), Error> {
        self._commit(self.trans_handle)
    }

    pub(crate) fn _begin_trans(&mut self) -> Result<i32, Error> {
        self.wp.op_transaction(false)?;
        let (trans_handle, _, _) = self.wp.op_response()?;
        Ok(trans_handle)
    }

    pub(crate) fn _rollback(&mut self, trans_handle: i32) -> Result<(), Error> {
        self.wp.op_rollback_retaining(trans_handle)?;
        self.wp.op_response()?;
        Ok(())
    }

    pub fn rollback(&mut self) -> Result<(), Error> {
        self._rollback(self.trans_handle)
    }

    pub fn _prepare(&mut self, query: &str, trans_handle: i32) -> Result<Statement, Error> {
        self.wp.op_allocate_statement()?;

        let mut stmt_handle = if self.wp.accept_type == PTYPE_LAZY_SEND {
            self.wp.lazy_response_count += 1;
            -1
        } else {
            let (stmt_handle, _, _) = self.wp.op_response()?;
            stmt_handle
        };

        self.wp
            .op_prepare_statement(stmt_handle, trans_handle, query)?;
        if self.wp.accept_type == PTYPE_LAZY_SEND && self.wp.lazy_response_count > 0 {
            self.wp.lazy_response_count -= 1;
            let (h, _, _) = self.wp.op_response()?;
            stmt_handle = h;
        }
        let (_, _, buf) = self.wp.op_response()?;
        let (stmt_type, xsqlda) = self.wp.parse_xsqlda(&buf, stmt_handle)?;

        Ok(Statement::new(
            self,
            trans_handle,
            stmt_handle,
            stmt_type,
            xsqlda,
            true, // autocommit is true
        ))
    }

    pub fn prepare(&mut self, query: &str) -> Result<Statement, Error> {
        self._prepare(query, self.trans_handle)
    }

    pub fn transaction(&mut self) -> Result<Transaction, Error> {
        Transaction::new(self)
    }

    // methods for Statement

    pub(crate) fn execute_query(
        &mut self,
        stmt_handle: i32,
        trans_handle: i32,
        params: &Vec<Param>,
    ) -> Result<(), Error> {
        self.wp.op_execute(stmt_handle, trans_handle, params)?;
        self.wp.op_response()?;
        Ok(())
    }

    pub(crate) fn fetch(
        &mut self,
        stmt_handle: i32,
        blr: &Vec<u8>,
        xsqlda: &[XSQLVar],
    ) -> Result<(Vec<Vec<CellValue>>, bool), Error> {
        self.wp.op_fetch(stmt_handle, &blr)?;
        self.wp.op_fetch_response(xsqlda)
    }

    pub(crate) fn get_blob_segments(
        &mut self,
        blob_id: &Vec<u8>,
        trans_handle: i32,
    ) -> Result<Vec<u8>, Error> {
        self.wp.get_blob_segments(blob_id, trans_handle)
    }

    pub(crate) fn free_statement(&mut self, stmt_handle: i32, drop_type: i32) -> () {
        self.wp.op_free_statement(stmt_handle, drop_type).unwrap();
        if self.wp.accept_type == PTYPE_LAZY_SEND {
            self.wp.lazy_response_count += 1;
        } else {
            self.wp.op_response().unwrap();
        }
    }

    // methods for Transaction
    pub(crate) fn drop_transaction(&mut self, trans_handle: i32) -> () {
        self.wp.op_rollback(trans_handle).unwrap();
        self.wp.op_response().unwrap();
    }
}
