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
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::conn_params::ConnParams;
use super::error::Error;
use super::wireprotocol::*;
use super::*;

pub struct Connection {
    pub(crate) wp: Rc<RefCell<WireProtocol>>,
    pub(crate) trans_handle: i32,
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
            wp: Rc::new(RefCell::new(wp)),
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
            wp: Rc::new(RefCell::new(wp)),
            trans_handle,
            conn_params,
            conn_options,
        })
    }

    pub fn execute_immediate(&self, query: &str) -> Result<(), Error> {
        let mut wp = self.wp.borrow_mut();
        wp.op_exec_immediate(self.trans_handle, query)?;
        wp.op_response()?;
        Ok(())
    }

    pub fn execute(&self, query: &str, params: Vec<Param>) -> Result<(), Error> {
        let mut wp = self.wp.borrow_mut();
        // TODO: create statement and execute
        wp.op_exec_immediate(self.trans_handle, query)?;
        wp.op_response()?;
        Ok(())
    }

    pub fn commit(&self) -> Result<(), Error> {
        let mut wp = self.wp.borrow_mut();
        wp.op_commit_retaining(self.trans_handle)?;
        wp.op_response()?;
        Ok(())
    }

    pub fn rollback(&self) -> Result<(), Error> {
        let mut wp = self.wp.borrow_mut();
        wp.op_rollback_retaining(self.trans_handle)?;
        wp.op_response()?;
        Ok(())
    }

    pub fn prepare(&self, query: &str) -> Result<Statement, Error> {
        let mut wp = self.wp.borrow_mut();
        wp.op_allocate_statement()?;

        let mut stmt_handle = if wp.accept_type == PTYPE_LAZY_SEND {
            -1
        } else {
            let (stmt_handle, _, _) = wp.op_response()?;
            stmt_handle
        };

        wp.op_prepare_statement(stmt_handle, self.trans_handle, query)?;
        if wp.accept_type == PTYPE_LAZY_SEND && wp.lazy_response_count > 0 {
            wp.lazy_response_count -= 1;
            let (h, _, _) = wp.op_response()?;
            stmt_handle = h;
        }
        let (_, buf, _) = wp.op_response()?;
        let (stmt_type, xsqlda) = wp.parse_xsqlda(&buf, stmt_handle)?;

        Ok(Statement::new(self, stmt_handle, stmt_type, xsqlda))
    }
}
