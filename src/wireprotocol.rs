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

use std::collections::{HashMap, HashSet};
use std::io::prelude::*;

use hex;
use num_bigint::BigInt;

use super::conn_params::ConnParams;
use super::error::{Error, FirebirdError};
use super::wirechannel::WireChannel;
use super::xsqlvar::XSQLVar;
use super::*;

const PLUGIN_NAME_LIST: &str = "Srp256,Srp";
const BUFFER_LEN: u32 = 1024;
const MAX_CHAR_LENGTH: usize = 32767;
const BLOB_SEGMENT_SIZE: usize = 32000;

// Protocol Types (accept_type)
pub const PTYPE_BATCH_SEND: i32 = 3; // Batch sends, no asynchrony
pub const PTYPE_OUT_OF_BAND: i32 = 4; // Batch sends w/ out of band notification
pub const PTYPE_LAZY_SEND: i32 = 5; // Deferred packets delivery

const ISC_ARG_END: u32 = 0;
const ISC_ARG_GDS: u32 = 1;
const ISC_ARG_STRING: u32 = 2;
const ISC_ARG_CSTRING: u32 = 3;
const ISC_ARG_NUMBER: u32 = 4;
const ISC_ARG_INTERPRETED: u32 = 5;
const ISC_ARG_VMS: u32 = 6;
const ISC_ARG_UNIX: u32 = 7;
const ISC_ARG_DOMAIN: u32 = 8;
const ISC_ARG_DOS: u32 = 9;
const ISC_ARG_MPEXL: u32 = 10;
const ISC_ARG_MPEXL_IPC: u32 = 11;
const ISC_ARG_NEXT_MACH: u32 = 15;
const ISC_ARG_NETWARE: u32 = 16;
const ISC_ARG_WIN32: u32 = 17;
const ISC_ARG_WARNING: u32 = 18;
const ISC_ARG_SQL_STATE: u32 = 19;

const ISC_INFO_END: u8 = 1;
const ISC_INFO_TRUNCATED: u8 = 2;
const ISC_INFO_ERROR: u8 = 3;
const ISC_INFO_DATA_NOT_READY: u8 = 4;
const ISC_INFO_LENGTH: u8 = 126;
const ISC_INFO_FLAG_END: u8 = 127;

const ISC_INFO_DB_ID: u8 = 4;
const ISC_INFO_READS: u8 = 5;
const ISC_INFO_WRITES: u8 = 6;
const ISC_INFO_FETCHES: u8 = 7;
const ISC_INFO_MARKS: u8 = 8;
const ISC_INFO_IMPLEMENTATION: u8 = 11;
const ISC_INFO_VERSION: u8 = 12;
const ISC_INFO_BASE_LEVEL: u8 = 13;
const ISC_INFO_PAGE_SIZE: u8 = 14;
const ISC_INFO_NUM_BUFFERS: u8 = 15;
const ISC_INFO_LIMBO: u8 = 16;
const ISC_INFO_CURRENT_MEMORY: u8 = 17;
const ISC_INFO_MAX_MEMORY: u8 = 18;
const ISC_INFO_WINDOW_TURNS: u8 = 19;
const ISC_INFO_LICENSE: u8 = 20;
const ISC_INFO_ALLOCATION: u8 = 21;
const ISC_INFO_ATTACHMENT_ID: u8 = 22;
const ISC_INFO_READ_SEQ_COUNT: u8 = 23;
const ISC_INFO_READ_IDX_COUNT: u8 = 24;
const ISC_INFO_INSERT_COUNT: u8 = 25;
const ISC_INFO_UPDATE_COUNT: u8 = 26;
const ISC_INFO_DELETE_COUNT: u8 = 27;
const ISC_INFO_BACKOUT_COUNT: u8 = 28;
const ISC_INFO_PURGE_COUNT: u8 = 29;
const ISC_INFO_EXPUNGE_COUNT: u8 = 30;
const ISC_INFO_SWEEP_INTERVAL: u8 = 31;
const ISC_INFO_ODS_VERSION: u8 = 32;
const ISC_INFO_ODS_MINOR_VERSION: u8 = 33;
const ISC_INFO_NO_RESERVE: u8 = 34;
const ISC_INFO_LOGFILE: u8 = 35;
const ISC_INFO_CUR_LOGFILE_NAME: u8 = 36;
const ISC_INFO_CUR_LOG_PART_OFFSET: u8 = 37;
const ISC_INFO_NUM_WAL_BUFFERS: u8 = 38;
const ISC_INFO_WAL_BUFFER_SIZE: u8 = 39;
const ISC_INFO_WAL_CKPT_LENGTH: u8 = 40;
const ISC_INFO_WAL_CUR_CKPT_INTERVAL: u8 = 41;
const ISC_INFO_WAL_PRV_CKPT_FNAME: u8 = 42;
const ISC_INFO_WAL_PRV_CKPT_POFFSET: u8 = 43;
const ISC_INFO_WAL_RECV_CKPT_FNAME: u8 = 44;
const ISC_INFO_WAL_RECV_CKPT_POFFSET: u8 = 45;
const ISC_INFO_WAL_GRPC_WAIT_USECS: u8 = 47;
const ISC_INFO_WAL_NUM_IO: u8 = 48;
const ISC_INFO_WAL_AVG_IO_SIZE: u8 = 49;
const ISC_INFO_WAL_NUM_COMMITS: u8 = 50;
const ISC_INFO_WAL_AVG_GRPC_SIZE: u8 = 51;
const ISC_INFO_FORCED_WRITES: u8 = 52;
const ISC_INFO_USER_NAMES: u8 = 53;
const ISC_INFO_PAGE_ERRORS: u8 = 54;
const ISC_INFO_RECORD_ERRORS: u8 = 55;
const ISC_INFO_BPAGE_ERRORS: u8 = 56;
const ISC_INFO_DPAGE_ERRORS: u8 = 57;
const ISC_INFO_IPAGE_ERRORS: u8 = 58;
const ISC_INFO_PPAGE_ERRORS: u8 = 59;
const ISC_INFO_TPAGE_ERRORS: u8 = 60;
const ISC_INFO_SET_PAGE_BUFFERS: u8 = 61;
const ISC_INFO_DB_SQL_DIALECT: u8 = 62;
const ISC_INFO_DB_READ_ONLY: u8 = 63;
const ISC_INFO_DB_SIZE_IN_PAGES: u8 = 64;
const ISC_INFO_ATT_CHARSET: u8 = 101;
const ISC_INFO_DB_CLASS: u8 = 102;
const ISC_INFO_FIREBIRD_VERSION: u8 = 103;
const ISC_INFO_OLDEST_TRANSACTION: u8 = 104;
const ISC_INFO_OLDEST_ACTIVE: u8 = 105;
const ISC_INFO_OLDEST_SNAPSHOT: u8 = 106;
const ISC_INFO_NEXT_TRANSACTION: u8 = 107;
const ISC_INFO_DB_PROVIDER: u8 = 108;
const ISC_INFO_ACTIVE_TRANSACTIONS: u8 = 109;
const ISC_INFO_ACTIVE_TRAN_COUNT: u8 = 110;
const ISC_INFO_CREATION_DATE: u8 = 111;
const ISC_INFO_DB_FILE_SIZE: u8 = 112;

// SQL information items
const ISC_INFO_SQL_SELECT: u8 = 4;
const ISC_INFO_SQL_BIND: u8 = 5;
const ISC_INFO_SQL_NUM_VARIABLES: u8 = 6;
const ISC_INFO_SQL_DESCRIBE_VARS: u8 = 7;
const ISC_INFO_SQL_DESCRIBE_END: u8 = 8;
const ISC_INFO_SQL_SQLDA_SEQ: u8 = 9;
const ISC_INFO_SQL_MESSAGE_SEQ: u8 = 10;
const ISC_INFO_SQL_TYPE: u8 = 11;
const ISC_INFO_SQL_SUB_TYPE: u8 = 12;
const ISC_INFO_SQL_SCALE: u8 = 13;
const ISC_INFO_SQL_LENGTH: u8 = 14;
const ISC_INFO_SQL_NULL_IND: u8 = 15;
const ISC_INFO_SQL_FIELD: u8 = 16;
const ISC_INFO_SQL_RELATION: u8 = 17;
const ISC_INFO_SQL_OWNER: u8 = 18;
const ISC_INFO_SQL_ALIAS: u8 = 19;
const ISC_INFO_SQL_SQLDA_START: u8 = 20;
const ISC_INFO_SQL_STMT_TYPE: u8 = 21;
const ISC_INFO_SQL_GET_PLAN: u8 = 22;
const ISC_INFO_SQL_RECORDS: u8 = 23;
const ISC_INFO_SQL_BATCH_FETCH: u8 = 24;

//Database Parameter Block Types
const ISC_DPB_VERSION1: u8 = 1;
const ISC_DPB_PAGE_SIZE: u8 = 4;
const ISC_DPB_NUM_BUFFERS: u8 = 5;
const ISC_DPB_FORCE_WRITE: u8 = 24;
const ISC_DPB_USER_NAME: u8 = 28;
const ISC_DPB_PASSWORD: u8 = 29;
const ISC_DPB_PASSWORD_ENC: u8 = 30;
const ISC_DPB_LC_CTYPE: u8 = 48;
const ISC_DPB_OVERWRITE: u8 = 54;
const ISC_DPB_CONNECT_TIMEOUT: u8 = 57;
const ISC_DPB_DUMMY_PACKET_INTERVAL: u8 = 58;
const ISC_DPB_SQL_ROLE_NAME: u8 = 60;
const ISC_DPB_SET_PAGE_BUFFERS: u8 = 61;
const ISC_DPB_SQL_DIALECT: u8 = 63;
const ISC_DPB_SET_DB_CHARSET: u8 = 68;
const ISC_DPB_PROCESS_ID: u8 = 71;
const ISC_DPB_NO_DB_TRIGGERS: u8 = 72;
const ISC_DPB_TRUSTED_AUTH: u8 = 73;
const ISC_DPB_PROCESS_NAME: u8 = 74;
const ISC_DPB_UTF8_FILENAME: u8 = 77;
const ISC_DPB_SPECIFIC_AUTH_DATA: u8 = 84;
const ISC_DPB_AUTH_PLUGIN_LIST: u8 = 85;
const ISC_DPB_AUTH_PLUGIN_NAME: u8 = 86;
const ISC_DPB_CONFIG: u8 = 87;
const ISC_DPB_NOLINGER: u8 = 88;
const ISC_DPB_RESET_ICU: u8 = 89;
const ISC_DPB_MAP_ATTACH: u8 = 90;
const ISC_DPB_SESSION_TIME_ZONE: u8 = 91;

const OP_CONNECT: u32 = 1;
const OP_EXIT: u32 = 2;
const OP_ACCEPT: u32 = 3;
const OP_REJECT: u32 = 4;
const OP_PROTOCROL: u32 = 5;
const OP_DISCONNECT: u32 = 6;
const OP_RESPONSE: u32 = 9;
const OP_ATTACH: u32 = 19;
const OP_CREATE: u32 = 20;
const OP_DETACH: u32 = 21;
const OP_TRANSACTION: u32 = 29;
const OP_COMMIT: u32 = 30;
const OP_ROLLBACK: u32 = 31;
const OP_OPEN_BLOB: u32 = 35;
const OP_GET_SEGMENT: u32 = 36;
const OP_PUT_SEGMENT: u32 = 37;
const OP_CLOSE_BLOB: u32 = 39;
const OP_INFO_DATABASE: u32 = 40;
const OP_INFO_TRANSACTION: u32 = 42;
const OP_BATCH_SEGMENTS: u32 = 44;
const OP_QUE_EVENTS: u32 = 48;
const OP_CANCEL_EVENTS: u32 = 49;
const OP_COMMIT_RETAINING: u32 = 50;
const OP_EVENT: u32 = 52;
const OP_CONNECT_REQUEST: u32 = 53;
const OP_AUX_CONNECT: u32 = 53;
const OP_CREATE_BLOB2: u32 = 57;
const OP_ALLOCATE_STATEMENT: u32 = 62;
const OP_EXECUTE: u32 = 63;
const OP_EXEC_IMMEDIATE: u32 = 64;
const OP_FETCH: u32 = 65;
const OP_FETCH_RESPONSE: u32 = 66;
const OP_FREE_STATEMENT: u32 = 67;
const OP_PREPARE_STATEMENT: u32 = 68;
const OP_INFO_SQL: u32 = 70;
const OP_DUMMY: u32 = 71;
const OP_EXECUTE2: u32 = 76;
const OP_SQL_RESPONSE: u32 = 78;
const OP_DROP_DATABASE: u32 = 81;
const OP_SERVICE_ATTACH: u32 = 82;
const OP_SERVICE_DETACH: u32 = 83;
const OP_SERVICE_INFO: u32 = 84;
const OP_SERVICE_START: u32 = 85;
const OP_ROLLBACK_RETAINING: u32 = 86;
// FB3
const OP_UPDATE_ACCOUNT_INFO: u32 = 87;
const OP_AUTHENTICATE_USER: u32 = 88;
const OP_PARTIAL: u32 = 89;
const OP_TRUSTED_AUTH: u32 = 90;
const OP_CANCEL: u32 = 91;
const OP_CONT_AUTH: u32 = 92;
const OP_PING: u32 = 93;
const OP_ACCEPT_DATA: u32 = 94;
const OP_ABORT_AUX_CONNECTION: u32 = 95;
const OP_CRYPT: u32 = 96;
const OP_CRYPT_KEY_CALLBACK: u32 = 97;
const OP_COND_ACCEPT: u32 = 98;

const CNCT_USER: u8 = 1;
const CNCT_PASSWD: u8 = 2;
const CNCT_HOST: u8 = 4;
const CNCT_GROUP: u8 = 5;
const CNCT_USER_VERIFICATION: u8 = 6;
const CNCT_SPECIFIC_DATA: u8 = 7;
const CNCT_PLUGIN_NAME: u8 = 8;
const CNCT_LOGIN: u8 = 9;
const CNCT_PLUGIN_LIST: u8 = 10;
const CNCT_CLIENT_CRYPT: u8 = 11;

const ISC_TPB_VERSION1: u8 = 1;
const ISC_TPB_VERSION3: u8 = 3;
const ISC_TPB_CONSISTENCY: u8 = 1;
const ISC_TPB_CONCURRENCY: u8 = 2;
const ISC_TPB_SHARED: u8 = 3;
const ISC_TPB_PROTECTED: u8 = 4;
const ISC_TPB_EXCLUSIVE: u8 = 5;
const ISC_TPB_WAIT: u8 = 6;
const ISC_TPB_NOWAIT: u8 = 7;
const ISC_TPB_READ: u8 = 8;
const ISC_TPB_WRITE: u8 = 9;
const ISC_TPB_LOCK_READ: u8 = 10;
const ISC_TPB_LOCK_WRITE: u8 = 11;
const ISC_TPB_VERB_TIME: u8 = 12;
const ISC_TPB_COMMIT_TIME: u8 = 13;
const ISC_TPB_IGNORE_LIMBO: u8 = 14;
const ISC_TPB_READ_COMMITTED: u8 = 15;
const ISC_TPB_AUTOCOMMIT: u8 = 16;
const ISC_TPB_REC_VERSION: u8 = 17;
const ISC_TPB_NO_REC_VERSION: u8 = 18;
const ISC_TPB_RESTART_REQUESTS: u8 = 19;
const ISC_TPB_NO_AUTO_UNDO: u8 = 20;
const ISC_TPB_LOCK_TIMEOUT: u8 = 21;

const ISC_INFO_REQ_SELECT_COUNT: u32 = 13;
const ISC_INFO_REQ_INSERT_COUNT: u32 = 14;
const ISC_INFO_REQ_UPDATE_COUNT: u32 = 15;
const ISC_INFO_REQ_DELETE_COUNT: u32 = 16;

const ISC_INFO_SVC_SVR_DB_INFO: u32 = 50;
const ISC_INFO_SVC_GET_LICENSE: u32 = 51;
const ISC_INFO_SVC_GET_LICENSE_MASK: u32 = 52;
const ISC_INFO_SVC_GET_CONFIG: u32 = 53;
const ISC_INFO_SVC_VERSION: u32 = 54;
const ISC_INFO_SVC_SERVER_VERSION: u32 = 55;
const ISC_INFO_SVC_IMPLEMENTATION: u32 = 56;
const ISC_INFO_SVC_CAPABILITIES: u32 = 57;
const ISC_INFO_SVC_USER_DBPATH: u32 = 58;
const ISC_INFO_SVC_GET_ENV: u32 = 59;
const ISC_INFO_SVC_GET_ENV_LOCK: u32 = 60;
const ISC_INFO_SVC_GET_ENV_MSG: u32 = 61;
const ISC_INFO_SVC_LINE: u32 = 62;
const ISC_INFO_SVC_TO_EOF: u32 = 63;
const ISC_INFO_SVC_TIMEOUT: u32 = 64;
const ISC_INFO_SVC_GET_LICENSED_USERS: u32 = 65;
const ISC_INFO_SVC_LIMBO_TRANS: u32 = 66;
const ISC_INFO_SVC_RUNNING: u32 = 67;
const ISC_INFO_SVC_GET_USERS: u32 = 68;

// Transaction informatino items
const ISC_INFO_TRA_ID: u32 = 4;
const ISC_INFO_TRA_OLDEST_INTERESTING: u32 = 5;
const ISC_INFO_TRA_OLDEST_SNAPSHOT: u32 = 6;
const ISC_INFO_TRA_OLDEST_ACTIVE: u32 = 7;
const ISC_INFO_TRA_ISOLATION: u32 = 8;
const ISC_INFO_TRA_ACCESS: u32 = 9;
const ISC_INFO_TRA_LOCK_TIMEOUT: u32 = 10;

const ISC_INFO_SQL_STMT_SELECT: u32 = 1;
const ISC_INFO_SQL_STMT_INSERT: u32 = 2;
const ISC_INFO_SQL_STMT_UPDATE: u32 = 3;
const ISC_INFO_SQL_STMT_DELETE: u32 = 4;
const ISC_INFO_SQL_STMT_DDL: u32 = 5;
const ISC_INFO_SQL_STMT_GET_SEGMENT: u32 = 6;
const ISC_INFO_SQL_STMT_PUT_SEGMENT: u32 = 7;
const ISC_INFO_SQL_STMT_EXEC_PROCEDURE: u32 = 8;
const ISC_INFO_SQL_STMT_START_TRANS: u32 = 9;
const ISC_INFO_SQL_STMT_COMMIT: u32 = 10;
const ISC_INFO_SQL_STMT_ROLLBACK: u32 = 11;
const ISC_INFO_SQL_STMT_SELECT_FOR_UPD: u32 = 12;
const ISC_INFO_SQL_STMT_SET_GENERATOR: u32 = 13;
const ISC_INFO_SQL_STMT_SAVEPOINT: u32 = 14;

macro_rules! debug_print {
    ($( $args:expr ),*) => { println!( $( $args ),* ); }
//    ($( $args:expr ),*) => {}
}

fn info_sql_select_describe_vars() -> [u8; 13] {
    [
        ISC_INFO_SQL_SELECT,
        ISC_INFO_SQL_DESCRIBE_VARS,
        ISC_INFO_SQL_SQLDA_SEQ,
        ISC_INFO_SQL_TYPE,
        ISC_INFO_SQL_SUB_TYPE,
        ISC_INFO_SQL_SCALE,
        ISC_INFO_SQL_LENGTH,
        ISC_INFO_SQL_NULL_IND,
        ISC_INFO_SQL_FIELD,
        ISC_INFO_SQL_RELATION,
        ISC_INFO_SQL_OWNER,
        ISC_INFO_SQL_ALIAS,
        ISC_INFO_SQL_DESCRIBE_END,
    ]
}

fn get_srp_client_public_bytes(client_public: &BigInt) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::new();

    let hex_string = hex::encode(&utils::big_int_to_bytes(client_public));
    let b = &hex_string.as_bytes();
    if b.len() > 254 {
        v.push(CNCT_SPECIFIC_DATA);
        v.push(255);
        v.push(0);
        v.write(&b[..254]).unwrap();

        v.push(CNCT_SPECIFIC_DATA);
        v.push((b.len() - 254 + 1) as u8);
        v.push(1);
        v.write(&b[254..]).unwrap();
    } else {
        v.push(CNCT_SPECIFIC_DATA);
        v.push(b.len() as u8 + 1);
        v.push(0);
        v.write(&b).unwrap();
    }
    v
}

#[derive(Debug)]
pub struct WireProtocol {
    write_buf: Vec<u8>,

    channel: WireChannel,
    host: String,
    port: u16,
    username: String,
    password: String,

    pub(crate) db_handle: i32,

    protocol_version: i32,
    accept_architecture: i32,
    pub(crate) accept_type: i32,
    pub(crate) lazy_response_count: i32,

    accept_plugin_name: String,
    auth_data: Option<Vec<u8>>,

    // Time Zone
    timezone: String,
}

impl WireProtocol {
    pub fn new(
        params: &ConnParams,
        option_params: &HashMap<String, String>,
    ) -> Result<WireProtocol, Error> {
        Ok(WireProtocol {
            write_buf: Vec::new(),
            channel: WireChannel::new(&params.host, params.port)?,
            host: params.host.to_string(),
            port: params.port,
            username: params.username.to_string(),
            password: params.password.to_string(),
            db_handle: -1,
            protocol_version: -1,
            accept_architecture: -1,
            accept_type: -1,
            lazy_response_count: 0,
            accept_plugin_name: "".to_string(),
            auth_data: None,
            timezone: option_params["timezone"].to_string(),
        })
    }

    fn pack_u32(&mut self, n: u32) -> () {
        self.write_buf.write(&n.to_be_bytes()).unwrap();
    }

    fn pack_bytes(&mut self, b: &[u8]) -> () {
        self.write_buf.write(&utils::xdr_bytes(b)).unwrap();
    }

    fn pack_str(&mut self, s: &str) -> () {
        self.write_buf
            .write(&utils::xdr_bytes(s.as_bytes()))
            .unwrap();
    }

    fn append_bytes(&mut self, b: &[u8]) -> () {
        self.write_buf.write(b).unwrap();
    }

    fn uid(
        &self,
        username: &str,
        password: &str,
        auth_plugin_name: &str,
        wire_crypt: bool,
        client_public: &BigInt,
    ) -> Vec<u8> {
        let mut v: Vec<u8> = Vec::new();
        let specific_data = get_srp_client_public_bytes(client_public);

        v.push(CNCT_LOGIN);
        v.push(username.len() as u8);
        v.write(username.as_bytes()).unwrap();
        v.push(CNCT_PLUGIN_NAME);
        v.push(auth_plugin_name.len() as u8);
        v.write(auth_plugin_name.as_bytes()).unwrap();
        v.push(CNCT_PLUGIN_LIST);
        v.push(PLUGIN_NAME_LIST.len() as u8);
        v.write(PLUGIN_NAME_LIST.as_bytes()).unwrap();
        v.write(&specific_data).unwrap();
        v.push(CNCT_CLIENT_CRYPT);
        v.push(4);
        if wire_crypt {
            v.push(1);
        } else {
            v.push(0);
        }
        v.push(0);
        v.push(0);
        v.push(0);
        v.push(CNCT_USER_VERIFICATION);
        v.push(0);

        v
    }

    fn send_packets(&mut self) -> Result<(), Error> {
        self.channel.write(&self.write_buf)?;
        &self.write_buf.clear();
        Ok(())
    }

    fn suspend_buffer(&mut self) -> Vec<u8> {
        let mut v: Vec<u8> = Vec::new();
        v.append(&mut self.write_buf);
        v
    }

    fn resume_buffer(&mut self, buf: &Vec<u8>) -> () {
        self.write_buf.write(buf).unwrap();
    }

    fn recv_packets(&mut self, n: usize) -> Result<Vec<u8>, Error> {
        Ok(self.channel.read(n)?)
    }

    fn recv_packets_alignment(&mut self, n: usize) -> Result<Vec<u8>, Error> {
        let mut padding = n % 4;
        if padding > 0 {
            padding = 4 - padding;
        }
        let v = self.channel.read(n)?;
        if padding > 0 {
            self.channel.read(padding)?;
        }
        Ok(v)
    }

    fn parse_status_vector(&mut self) -> Result<(HashSet<u32>, i32, String), Error> {
        let mut sql_code: i32 = 0;
        let mut gds_code: u32 = 0;
        let mut gds_codes: HashSet<u32> = HashSet::new();
        let mut num_arg = 0;
        let mut message = String::new();

        let mut n = utils::bytes_to_buint32(&self.recv_packets(4)?);
        while n != ISC_ARG_END {
            match n {
                ISC_ARG_GDS => {
                    gds_code = utils::bytes_to_buint32(&self.recv_packets(4)?);
                    if gds_code != 0 {
                        gds_codes.insert(gds_code);
                        message.push_str(errmsgs::error_message_by_id(gds_code));
                        num_arg = 0;
                    }
                }
                ISC_ARG_NUMBER => {
                    let num = utils::bytes_to_buint32(&self.recv_packets(4)?);
                    if gds_code == 335544436 {
                        sql_code = num as i32;
                    }
                    num_arg += 1;
                    let place_folder = format!("@{}", num_arg);
                    message = message.replace(&place_folder, &num.to_string());
                }
                ISC_ARG_STRING => {
                    let nbytes = utils::bytes_to_buint32(&self.recv_packets(4)?);
                    let s = utils::bytes_to_str(&self.recv_packets_alignment(nbytes as usize)?);
                    num_arg += 1;
                    let place_folder = format!("@{}", num_arg);
                    message = message.replace(&place_folder, &s);
                }
                ISC_ARG_INTERPRETED => {
                    let nbytes = utils::bytes_to_buint32(&self.recv_packets(4)?);
                    let s = utils::bytes_to_str(&self.recv_packets_alignment(nbytes as usize)?);
                    message.push_str(&s);
                }
                ISC_ARG_SQL_STATE => {
                    let nbytes = utils::bytes_to_buint32(&self.recv_packets(4)?);
                    self.recv_packets_alignment(nbytes as usize)?; // skip status code
                }
                _ => break,
            }

            n = utils::bytes_to_buint32(&self.recv_packets(4)?);
        }

        Ok((gds_codes, sql_code, message))
    }

    pub(crate) fn parse_op_response(&mut self) -> Result<(i32, Vec<u8>, Vec<u8>), Error> {
        let h: i32 = utils::bytes_to_buint32(&self.recv_packets(4)?) as i32;
        let oid: Vec<u8> = self.recv_packets(8)?;
        let nbytes = utils::bytes_to_buint32(&self.recv_packets(4)?);
        let buf: Vec<u8> = self.recv_packets_alignment(nbytes as usize)?;

        let (gds_codes, sql_code, message) = self.parse_status_vector()?;

        if gds_codes.len() > 0 || sql_code != 0 {
            Err(Error::FirebirdError(FirebirdError::new(&message, sql_code)))
        } else {
            Ok((h, oid, buf))
        }
    }

    pub fn parse_connect_response(
        &mut self,
        username: &str,
        password: &str,
        options: &HashMap<String, String>,
        client_public: &BigInt,
        client_secret: &BigInt,
    ) -> Result<(), Error> {
        let mut opcode = utils::bytes_to_buint32(&self.recv_packets(4)?);
        while opcode == OP_DUMMY {
            opcode = utils::bytes_to_buint32(&self.recv_packets(4)?);
        }

        if opcode == OP_REJECT {
            return Err(Error::FirebirdError(FirebirdError::new("op_reject", 0)));
        }
        if opcode == OP_RESPONSE {
            self.parse_op_response()?; // error
            panic!("connection error"); // not reach
        }

        self.protocol_version = self.recv_packets(1)?[0] as i32;
        self.recv_packets(3)?;
        self.accept_architecture = utils::bytes_to_buint32(&self.recv_packets(4)?) as i32;
        self.accept_type = utils::bytes_to_buint32(&self.recv_packets(4)?) as i32;

        assert!(opcode == OP_COND_ACCEPT || opcode == OP_ACCEPT_DATA);

        let mut ln: usize = utils::bytes_to_buint32(&self.recv_packets(4)?) as usize;
        let mut data = self.recv_packets_alignment(ln)?;

        ln = utils::bytes_to_buint32(&self.recv_packets(4)?) as usize;
        self.accept_plugin_name =
            String::from_utf8_lossy(&self.recv_packets_alignment(ln)?).to_string();

        // is_authenticated == 0
        assert_eq!(utils::bytes_to_buint32(&self.recv_packets(4)?), 0);

        // skip keys
        ln = utils::bytes_to_buint32(&self.recv_packets(4)?) as usize;
        &self.recv_packets_alignment(ln);

        assert!(&self.accept_plugin_name == "Srp" || &self.accept_plugin_name == "Srp256");

        if data.len() == 0 {
            self.op_cont_auth(&utils::big_int_to_bytes(client_public))?;
            assert_eq!(
                utils::bytes_to_buint32(&self.recv_packets(4)?),
                OP_CONT_AUTH
            );

            ln = utils::bytes_to_buint32(&self.recv_packets_alignment(4)?) as usize;
            data = self.recv_packets_alignment(ln)?;

            ln = utils::bytes_to_buint32(&self.recv_packets_alignment(4)?) as usize;
            &self.recv_packets_alignment(ln)?; // plugin_name

            ln = utils::bytes_to_buint32(&self.recv_packets_alignment(4)?) as usize;
            &self.recv_packets_alignment(ln)?; // plugin_name_list

            // skip keys
            ln = utils::bytes_to_buint32(&self.recv_packets(4)?) as usize;
            &self.recv_packets_alignment(ln);
        }
        ln = utils::bytes_to_uint16(&data[..2]) as usize;
        let servre_salt = &data[2..2 + ln];
        let server_public =
            utils::big_int_from_hex_string(&utils::bytes_to_str(&data[4 + ln..]).as_bytes());
        let (auth_data, session_key) = srp::get_client_proof(
            &self.username.to_uppercase(),
            &self.password,
            servre_salt,
            client_public,
            &server_public,
            &client_secret,
            &self.accept_plugin_name,
        );

        if opcode == OP_COND_ACCEPT {
            self.op_cont_auth(&auth_data)?;
            self.op_response()?;
        }

        if options["wire_crypt"] == "true" {
            self.op_crypt()?;
            self.channel.set_arc4_key(&session_key);
            self.op_response()?;
        } else {
            self.auth_data = Some(auth_data); // use in op_attach(), op_create()
        }

        Ok(())
    }

    fn parse_select_items(&mut self, buf: &[u8], xsqlda: &mut [XSQLVar]) -> Result<isize, Error> {
        let mut index: usize = 0;
        let mut i: usize = 0;
        let mut item = buf[i];
        while item != ISC_INFO_END {
            i += 1;
            match item {
                ISC_INFO_SQL_SQLDA_SEQ => {
                    let ln: usize = utils::bytes_to_uint16(&buf[i..i + 2]) as usize;
                    i += 2;
                    index = utils::bytes_to_uint32(&buf[i..i + ln]) as usize;
                    i += ln;
                }
                ISC_INFO_SQL_TYPE => {
                    let ln: usize = utils::bytes_to_uint16(&buf[i..i + 2]) as usize;
                    i += 2;
                    let mut sqltype = utils::bytes_to_uint32(&buf[i..i + ln]);
                    if sqltype % 2 != 0 {
                        sqltype -= 1;
                    }
                    xsqlda[index - 1].sqltype = sqltype;
                    i += ln;
                }
                ISC_INFO_SQL_SUB_TYPE => {
                    let ln: usize = utils::bytes_to_uint16(&buf[i..i + 2]) as usize;
                    i += 2;
                    let sqlsubtype = utils::bytes_to_uint32(&buf[i..i + ln]);
                    i += ln;
                }
                ISC_INFO_SQL_SCALE => {
                    let ln: usize = utils::bytes_to_uint16(&buf[i..i + 2]) as usize;
                    i += 2;
                    xsqlda[index - 1].sqlsubtype = utils::bytes_to_uint32(&buf[i..i + ln]) as i32;
                    i += ln;
                }
                ISC_INFO_SQL_LENGTH => {
                    let ln: usize = utils::bytes_to_uint16(&buf[i..i + 2]) as usize;
                    i += 2;
                    xsqlda[index - 1].sqllen = utils::bytes_to_uint32(&buf[i..i + ln]) as i32;
                    i += ln;
                }
                ISC_INFO_SQL_NULL_IND => {
                    let ln: usize = utils::bytes_to_uint16(&buf[i..i + 2]) as usize;
                    i += 2;
                    xsqlda[index - 1].null_ok = utils::bytes_to_uint32(&buf[i..i + ln]) != 0;
                    i += ln;
                }
                ISC_INFO_SQL_FIELD => {
                    let ln: usize = utils::bytes_to_uint16(&buf[i..i + 2]) as usize;
                    i += 2;
                    xsqlda[index - 1].fieldname = utils::bytes_to_str(&buf[i..i + ln]);
                    i += ln;
                }
                ISC_INFO_SQL_RELATION => {
                    let ln: usize = utils::bytes_to_uint16(&buf[i..i + 2]) as usize;
                    i += 2;
                    xsqlda[index - 1].relname = utils::bytes_to_str(&buf[i..i + ln]);
                    i += ln;
                }
                ISC_INFO_SQL_OWNER => {
                    let ln: usize = utils::bytes_to_uint16(&buf[i..i + 2]) as usize;
                    i += 2;
                    xsqlda[index - 1].ownname = utils::bytes_to_str(&buf[i..i + ln]);
                    i += ln;
                }
                ISC_INFO_SQL_ALIAS => {
                    let ln: usize = utils::bytes_to_uint16(&buf[i..i + 2]) as usize;
                    i += 2;
                    xsqlda[index - 1].aliasname = utils::bytes_to_str(&buf[i..i + ln]);
                    i += ln;
                }
                ISC_INFO_TRUNCATED => return Ok(index as isize),
                ISC_INFO_SQL_DESCRIBE_END => { /* NOTHING */ }
                _ => panic!("protocol sequence fail!"),
            }

            item = buf[i]
        }

        Ok(-1)
    }

    pub fn parse_xsqlda(
        &mut self,
        buf: &[u8],
        stmt_handle: i32,
    ) -> Result<(u32, Vec<XSQLVar>), Error> {
        let mut xsqlda: Vec<XSQLVar> = Vec::new();
        let mut stmt_type = 0;

        let mut i: usize = 0;
        while i < buf.len() {
            if buf[i] == ISC_INFO_SQL_STMT_TYPE && buf[i + 1] == 4 && buf[i + 1] == 0 {
                i += 1;
                let ln: usize = utils::bytes_to_uint16(&buf[i..i + 2]) as usize;
                i += 2;
                stmt_type = utils::bytes_to_uint32(&buf[i..i + ln]);
                i += ln;
            } else if buf[i] == ISC_INFO_SQL_SELECT && buf[i + 1] == ISC_INFO_SQL_DESCRIBE_VARS {
                i += 2;
                let ln: usize = utils::bytes_to_uint16(&buf[i..i + 2]) as usize;
                i += 2;
                let col_len = utils::bytes_to_uint32(&buf[i..i + ln]) as usize;
                for _ in 0..col_len {
                    xsqlda.push(XSQLVar::new());
                }
                let mut next_index: i16 =
                    self.parse_select_items(&buf[i + ln..], &mut xsqlda)? as i16;
                while next_index > 0 {
                    // more describe vars
                    let mut vars: Vec<u8> = Vec::new();
                    vars.push(ISC_INFO_SQL_SQLDA_START);
                    vars.push(2);
                    vars.write(&utils::int16_to_bytes(next_index as u16))?;
                    vars.write(&info_sql_select_describe_vars())?;
                    self.op_info_sql(stmt_handle, &vars)?;
                    let (_, _, buf) = self.op_response()?;
                    let ln: usize = utils::bytes_to_uint16(&buf[0..4]) as usize;
                    next_index = self.parse_select_items(&buf[4 + ln..], &mut xsqlda)? as i16;
                }
            } else {
                break;
            }
        }

        Ok((stmt_type, xsqlda))
    }

    fn get_blob_segments(
        &mut self,
        blob_id: &Vec<u8>,
        trans_handle: i32,
    ) -> Result<Vec<u8>, Error> {
        let buf = self.suspend_buffer();

        let mut blob: Vec<u8> = Vec::new();
        self.op_open_blob(blob_id, trans_handle)?;
        let (blob_handle, _, _) = self.op_response()?;
        let mut more_data: i32 = 1;
        while more_data != 2 {
            self.op_get_segment(blob_handle)?;
            let (more_data2, _, buf) = self.op_response()?;
            more_data = more_data2;
            let mut i: usize = 0;
            while i < buf.len() {
                let ln: usize = utils::bytes_to_uint16(&buf[i..i + 2]) as usize;
                blob.write(&buf[i + 2..i + 2 + ln])?;
                i += ln + 2;
            }
        }

        self.op_close_blob(blob_handle)?;
        if self.accept_type == PTYPE_LAZY_SEND {
            self.lazy_response_count += 1;
        } else {
            self.op_response()?;
        }

        self.resume_buffer(&buf);
        Ok(blob)
    }

    pub fn op_connect(
        &mut self,
        db_name: &str,
        username: &str,
        password: &str,
        options: &HashMap<String, String>,
        client_public: &BigInt,
    ) -> Result<(), Error> {
        debug_print!("op_connect()");
        self.pack_u32(OP_CONNECT);
        self.pack_u32(OP_ATTACH);
        self.pack_u32(3); // CONNECT_VERSION3
        self.pack_u32(1); // Arch Type(GENERIC)
        self.pack_str(db_name);
        self.pack_u32(1); // protocol count
        self.pack_bytes(&self.uid(
            &username,
            password,
            &options["auth_plugin_name"],
            options["wire_crypt"] == "true",
            client_public,
        ));
        // PROTOCOL_VERSION, Arch type (Generic=1), min, max, weight = 13, 1, 0, 5, 8
        self.append_bytes(&hex::decode("ffff800d00000001000000000000000500000008").unwrap());
        self.send_packets()?;

        Ok(())
    }

    pub fn op_create(
        &mut self,
        db_name: &str,
        username: &str,
        password: &str,
        role: &str,
        page_size: u32,
    ) -> Result<(), Error> {
        debug_print!("op_create()");
        let encode = b"UTF8";

        let mut dpb: Vec<u8> = Vec::new();
        dpb.push(ISC_DPB_VERSION1);
        dpb.push(ISC_DPB_SET_DB_CHARSET);
        dpb.push(encode.len() as u8);
        dpb.write(encode)?;
        dpb.push(ISC_DPB_LC_CTYPE);
        dpb.push(encode.len() as u8);
        dpb.write(encode)?;

        dpb.push(ISC_DPB_USER_NAME);
        dpb.push(username.len() as u8);
        dpb.write(username.as_bytes())?;

        if role != "" {
            dpb.push(ISC_DPB_SQL_ROLE_NAME);
            dpb.push(role.len() as u8);
            dpb.write(role.as_bytes())?;
        }

        dpb.push(ISC_DPB_SQL_DIALECT);
        dpb.write(&[4, 3, 0, 0, 0])?;
        dpb.push(ISC_DPB_FORCE_WRITE);
        dpb.write(&[4, 1, 0, 0, 0])?;
        dpb.push(ISC_DPB_OVERWRITE);
        dpb.write(&[4, 1, 0, 0, 0])?;
        dpb.push(ISC_DPB_PAGE_SIZE);
        dpb.push(4);
        dpb.write(&utils::uint32_to_bytes(page_size))?;

        if let Some(data) = &self.auth_data {
            let specific_auth_data = hex::encode(data);
            dpb.push(ISC_DPB_SPECIFIC_AUTH_DATA);
            dpb.push(specific_auth_data.len() as u8);
            dpb.write(&specific_auth_data.as_bytes())?;
        }
        if &self.timezone != "" {
            dpb.push(ISC_DPB_SESSION_TIME_ZONE);
            dpb.push(self.timezone.len() as u8);
            dpb.write(&self.timezone.as_bytes())?;
        }

        self.pack_u32(OP_CREATE);
        self.pack_u32(0); // Database Object ID
        self.pack_str(db_name);
        self.pack_bytes(&dpb);
        self.send_packets()?;

        Ok(())
    }

    pub fn op_attach(
        &mut self,
        db_name: &str,
        username: &str,
        password: &str,
        role: &str,
    ) -> Result<(), Error> {
        debug_print!("op_attach()");
        let encode = b"UTF8";

        let mut dpb: Vec<u8> = Vec::new();
        dpb.push(ISC_DPB_VERSION1);

        dpb.push(ISC_DPB_SQL_DIALECT);
        dpb.write(&[4, 3, 0, 0, 0])?;

        dpb.push(ISC_DPB_LC_CTYPE);
        dpb.push(encode.len() as u8);
        dpb.write(encode)?;

        dpb.push(ISC_DPB_USER_NAME);
        dpb.push(username.len() as u8);
        dpb.write(username.as_bytes())?;
        if role != "" {
            dpb.push(ISC_DPB_SQL_ROLE_NAME);
            dpb.push(role.len() as u8);
            dpb.write(role.as_bytes())?;
        }

        if let Some(data) = &self.auth_data {
            let specific_auth_data = hex::encode(data);
            dpb.push(ISC_DPB_SPECIFIC_AUTH_DATA);
            dpb.push(specific_auth_data.len() as u8);
            dpb.write(&specific_auth_data.as_bytes())?;
        }
        if &self.timezone != "" {
            dpb.push(ISC_DPB_SESSION_TIME_ZONE);
            dpb.push(self.timezone.len() as u8);
            dpb.write(&self.timezone.as_bytes())?;
        }

        self.pack_u32(OP_ATTACH);
        self.pack_u32(0); // Database Object ID
        self.pack_str(db_name);
        self.pack_bytes(&dpb);
        self.send_packets()?;

        Ok(())
    }

    pub fn op_cont_auth(&mut self, auth_data: &Vec<u8>) -> Result<(), Error> {
        debug_print!("op_cont_auth()");
        self.pack_u32(OP_CONT_AUTH);
        self.pack_bytes(&hex::encode(auth_data).as_bytes());
        self.pack_str(&self.accept_plugin_name.to_string());
        self.pack_str(PLUGIN_NAME_LIST);
        self.pack_str("");
        self.send_packets()?;

        Ok(())
    }

    pub fn op_crypt(&mut self) -> Result<(), Error> {
        debug_print!("op_crypt()");
        self.pack_u32(OP_CRYPT);
        self.pack_str("Arc4");
        self.pack_str("Symmetric");
        self.send_packets()?;

        Ok(())
    }

    pub fn op_drop_database(&mut self) -> Result<(), Error> {
        debug_print!("op_drop_database()");
        self.pack_u32(OP_DROP_DATABASE);
        self.pack_u32(self.db_handle as u32);
        self.send_packets()?;

        Ok(())
    }

    pub fn op_transaction(&mut self, is_autocommit: bool) -> Result<(), Error> {
        debug_print!("op_transaction()");
        let tpb: Vec<u8> = if is_autocommit {
            vec![
                ISC_TPB_VERSION3,
                ISC_TPB_WRITE,
                ISC_TPB_WAIT,
                ISC_TPB_AUTOCOMMIT,
            ]
        } else {
            vec![
                ISC_TPB_VERSION3,
                ISC_TPB_WRITE,
                ISC_TPB_WAIT,
                ISC_TPB_READ_COMMITTED,
                ISC_TPB_NO_REC_VERSION,
            ]
        };

        self.pack_u32(OP_TRANSACTION);
        self.pack_u32(self.db_handle as u32);
        self.pack_bytes(&tpb);
        self.send_packets()?;

        Ok(())
    }

    pub fn op_commit(&mut self, trans_handle: i32) -> Result<(), Error> {
        debug_print!("op_commit()");
        self.pack_u32(OP_COMMIT);
        self.pack_u32(trans_handle as u32);
        self.send_packets()?;

        Ok(())
    }

    pub fn op_commit_retaining(&mut self, trans_handle: i32) -> Result<(), Error> {
        debug_print!("op_commit_retaining()");
        self.pack_u32(OP_COMMIT_RETAINING);
        self.pack_u32(trans_handle as u32);
        self.send_packets()?;

        Ok(())
    }

    pub fn op_rollback(&mut self, trans_handle: i32) -> Result<(), Error> {
        debug_print!("op_rollback()");
        self.pack_u32(OP_ROLLBACK);
        self.pack_u32(trans_handle as u32);
        self.send_packets()?;

        Ok(())
    }

    pub fn op_rollback_retaining(&mut self, trans_handle: i32) -> Result<(), Error> {
        debug_print!("op_rollback_retaining()");
        self.pack_u32(OP_ROLLBACK_RETAINING);
        self.pack_u32(trans_handle as u32);
        self.send_packets()?;

        Ok(())
    }

    pub fn op_allocate_statement(&mut self) -> Result<(), Error> {
        debug_print!("op_allocate_statement()");
        self.pack_u32(OP_ALLOCATE_STATEMENT);
        self.pack_u32(self.db_handle as u32);
        self.send_packets()?;

        Ok(())
    }

    pub fn op_info_transaction(&mut self, trans_handle: i32, b: &[u8]) -> Result<(), Error> {
        debug_print!("op_info_transaction()");
        self.pack_u32(OP_INFO_TRANSACTION);
        self.pack_u32(trans_handle as u32);
        self.pack_u32(0);
        self.pack_bytes(b);
        self.pack_u32(BUFFER_LEN);
        self.send_packets()?;

        Ok(())
    }

    pub fn op_info_database(&mut self, bs: &[u8]) -> Result<(), Error> {
        debug_print!("op_info_database()");
        self.pack_u32(OP_INFO_DATABASE);
        self.pack_u32(self.db_handle as u32);
        self.pack_u32(0);
        self.pack_bytes(bs);
        self.pack_u32(BUFFER_LEN);
        self.send_packets()?;

        Ok(())
    }

    pub fn op_free_statement(&mut self, stmt_handle: i32, mode: i32) -> Result<(), Error> {
        debug_print!("op_free_statement()");
        self.pack_u32(OP_FREE_STATEMENT);
        self.pack_u32(stmt_handle as u32);
        self.pack_u32(mode as u32);
        self.send_packets()?;

        Ok(())
    }

    pub fn op_prepare_statement(
        &mut self,
        stmt_handle: i32,
        trans_handle: i32,
        query: &str,
    ) -> Result<(), Error> {
        debug_print!("op_prepare_statement()");
        let mut bs: Vec<u8> = Vec::new();
        bs.push(ISC_INFO_SQL_STMT_TYPE);
        bs.write(&info_sql_select_describe_vars())?;

        self.pack_u32(OP_PREPARE_STATEMENT);
        self.pack_u32(trans_handle as u32);
        self.pack_u32(stmt_handle as u32);
        self.pack_u32(3); // dialect = 3
        self.pack_str(query);
        self.pack_bytes(&bs);
        self.pack_u32(BUFFER_LEN);
        self.send_packets()?;

        Ok(())
    }

    pub fn op_info_sql(&mut self, stmt_handle: i32, vars: &[u8]) -> Result<(), Error> {
        debug_print!("op_info_sql()");
        self.pack_u32(OP_INFO_SQL);
        self.pack_u32(stmt_handle as u32);
        self.pack_u32(0);
        self.pack_bytes(&vars);
        self.pack_u32(BUFFER_LEN);
        self.send_packets()?;

        Ok(())
    }

    pub fn op_execute(
        &mut self,
        stmt_handle: i32,
        trans_handle: i32,
        params: &Vec<Param>,
    ) -> Result<(), Error> {
        debug_print!("op_execute()");
        self.pack_u32(OP_EXECUTE);
        self.pack_u32(stmt_handle as u32);
        self.pack_u32(trans_handle as u32);

        if params.len() == 0 {
            self.pack_u32(0);
            self.pack_u32(0);
            self.pack_u32(0);
        } else {
            let (blr, values) = self.params_to_blr(trans_handle, params)?;
            // println!("blr={:?}", &blr);
            // println!("values={:?}", &values);
            self.pack_bytes(&blr);
            self.pack_u32(0);
            self.pack_u32(1);
            self.append_bytes(&values);
        }
        self.send_packets()?;
        Ok(())
    }

    pub fn op_execute2(
        &mut self,
        stmt_handle: i32,
        trans_handle: i32,
        params: &Vec<Param>,
        output_blr: &[u8],
    ) -> Result<(), Error> {
        debug_print!("op_execute2()");
        self.pack_u32(OP_EXECUTE2);
        self.pack_u32(stmt_handle as u32);
        self.pack_u32(trans_handle as u32);

        if params.len() == 0 {
            self.pack_u32(0);
            self.pack_u32(0);
            self.pack_u32(0);
        } else {
            let (blr, values) = self.params_to_blr(trans_handle, params)?;
            self.pack_bytes(&blr);
            self.pack_u32(0);
            self.pack_u32(1);
            self.append_bytes(&values);
        }
        self.send_packets()?;
        Ok(())
    }

    pub fn op_exec_immediate(&mut self, trans_handle: i32, query: &str) -> Result<(), Error> {
        debug_print!("op_exec_immediate()");
        let desc_items: Vec<u8> = vec![];

        self.pack_u32(OP_EXEC_IMMEDIATE);
        self.pack_u32(trans_handle as u32);
        self.pack_u32(self.db_handle as u32);
        self.pack_u32(3); // dialect = 3
        self.pack_str(query);
        self.pack_bytes(&desc_items);
        self.pack_u32(BUFFER_LEN);
        self.send_packets()?;
        Ok(())
    }

    pub fn op_fetch(&mut self, stmt_handle: i32, blr: &Vec<u8>) -> Result<(), Error> {
        debug_print!("op_fetch()");
        self.pack_u32(OP_FETCH);
        self.pack_u32(stmt_handle as u32);
        self.pack_bytes(blr);
        self.pack_u32(0);
        self.pack_u32(400);
        self.send_packets()?;
        Ok(())
    }

    pub fn op_fetch_response(
        &mut self,
        stmt_handle: i32,
        trans_handle: i32,
        xsqlda: &[XSQLVar],
    ) -> Result<(Vec<Vec<Value>>, bool), Error> {
        debug_print!("op_fetch_response()");
        let mut opcode = utils::bytes_to_buint32(&self.recv_packets(4)?);
        while opcode == OP_DUMMY {
            opcode = utils::bytes_to_buint32(&self.recv_packets(4)?);
        }
        while opcode == OP_RESPONSE && self.lazy_response_count > 0 {
            self.lazy_response_count -= 1;
            self.parse_op_response()?;
            opcode = utils::bytes_to_buint32(&self.recv_packets(4)?);
        }

        if opcode != OP_FETCH_RESPONSE {
            self.parse_op_response()?;
            panic!("op fetch response error"); // not reach
        }

        let mut status = utils::bytes_to_buint32(&self.recv_packets(4)?);
        let mut count = utils::bytes_to_buint32(&self.recv_packets(4)?);
        let mut rows: Vec<Vec<Value>> = Vec::new();
        let xsqlda_len = xsqlda.len();

        while count > 0 {
            let mut n = xsqlda_len / 8;
            if xsqlda_len % 8 != 0 {
                n += 1;
            }
            let mut null_indicator: u128 = 0;
            let b = &self.recv_packets_alignment(n)?;
            for c in b.iter().rev() {
                null_indicator <<= 8;
                null_indicator += *c as u128;
            }

            let mut row: Vec<Value> = Vec::with_capacity(xsqlda_len);
            for (i, x) in xsqlda.iter().enumerate() {
                if (null_indicator & (1 << i)) != 0 {
                    row.push(Value::Null)
                } else {
                    let ln = if x.io_length() < 0 {
                        utils::bytes_to_buint32(&self.recv_packets(4)?) as usize
                    } else {
                        x.io_length() as usize
                    };
                    let raw_value = self.recv_packets_alignment(ln as usize)?;
                    row.push(x.value(&raw_value)?);
                }
            }
            rows.push(row);
            let _op_code = utils::bytes_to_buint32(&self.recv_packets(4)?);
            status = utils::bytes_to_buint32(&self.recv_packets(4)?);
            count = utils::bytes_to_buint32(&self.recv_packets(4)?);
        }

        Ok((rows, status != 100))
    }

    pub fn op_detach(&mut self) -> Result<(), Error> {
        debug_print!("op_detatch()");
        self.pack_u32(OP_DETACH);
        self.pack_u32(self.db_handle as u32);
        self.send_packets()?;
        Ok(())
    }

    pub fn op_open_blob(&mut self, blob_id: &Vec<u8>, trans_handle: i32) -> Result<(), Error> {
        debug_print!("op_open_blob()");
        self.pack_u32(OP_OPEN_BLOB);
        self.pack_u32(trans_handle as u32);
        self.append_bytes(blob_id);
        self.send_packets()?;
        Ok(())
    }

    pub fn op_create_blob2(&mut self, trans_handle: i32) -> Result<(), Error> {
        debug_print!("op_create_blob2()");
        self.pack_u32(OP_CREATE_BLOB2);
        self.pack_u32(0);
        self.pack_u32(trans_handle as u32);
        self.pack_u32(0);
        self.pack_u32(0);
        self.send_packets()?;
        Ok(())
    }

    pub fn op_get_segment(&mut self, blob_handle: i32) -> Result<(), Error> {
        debug_print!("op_get_segment()");
        self.pack_u32(OP_GET_SEGMENT);
        self.pack_u32(blob_handle as u32);
        self.pack_u32(BUFFER_LEN);
        self.pack_u32(0);
        self.send_packets()?;

        Ok(())
    }

    pub fn op_put_segment(&mut self, blob_handle: i32, seg_data: &[u8]) -> Result<(), Error> {
        debug_print!("op_put_segment()");
        let ln = seg_data.len();
        self.pack_u32(OP_PUT_SEGMENT);
        self.pack_u32(blob_handle as u32);
        self.pack_u32(ln as u32);
        self.pack_u32(ln as u32);
        self.append_bytes(seg_data);
        let pad_length: usize = (4 - ln) & 3;
        self.append_bytes(&vec![0; pad_length]);
        self.send_packets()?;

        Ok(())
    }

    pub fn op_batch_segments(&mut self, blob_handle: i32, seg_data: &Vec<u8>) -> Result<(), Error> {
        debug_print!("op_batch_segments()");
        let ln = seg_data.len();
        self.pack_u32(OP_BATCH_SEGMENTS);
        self.pack_u32(blob_handle as u32);
        self.pack_u32(ln as u32);
        self.pack_u32(ln as u32);
        self.pack_bytes(&utils::int16_to_bytes(ln as u16));
        self.pack_bytes(seg_data);
        let pad_length: usize = (4 - (ln + 2)) & 3;
        self.append_bytes(&vec![0; pad_length]);
        self.send_packets()?;

        Ok(())
    }

    pub fn op_close_blob(&mut self, blob_handle: i32) -> Result<(), Error> {
        debug_print!("op_close_blob()");
        self.pack_u32(OP_CLOSE_BLOB);
        self.pack_u32(blob_handle as u32);
        self.send_packets()?;
        Ok(())
    }

    pub fn op_response(&mut self) -> Result<(i32, Vec<u8>, Vec<u8>), Error> {
        debug_print!("op_response()");
        let mut opcode = utils::bytes_to_buint32(&self.recv_packets(4)?);
        while opcode == OP_DUMMY {
            opcode = utils::bytes_to_buint32(&self.recv_packets(4)?);
        }
        while opcode == OP_RESPONSE && self.lazy_response_count > 0 {
            self.lazy_response_count -= 1;
            self.parse_op_response()?;
            opcode = utils::bytes_to_buint32(&self.recv_packets(4)?);
        }

        if opcode != OP_RESPONSE {
            Err(Error::FirebirdError(FirebirdError::new(
                "Authentication error",
                0,
            )))
        } else {
            self.parse_op_response()
        }
    }

    pub fn op_sql_response(&mut self, xsqlda: &[XSQLVar]) -> Result<Vec<Value>, Error> {
        debug_print!("op_sql_response()");
        let xsqlda_len = xsqlda.len();
        let mut row: Vec<Value> = Vec::with_capacity(xsqlda_len);
        let mut opcode = utils::bytes_to_buint32(&self.recv_packets(4)?);
        while opcode == OP_DUMMY {
            opcode = utils::bytes_to_buint32(&self.recv_packets(4)?);
        }
        if opcode == OP_RESPONSE {
            self.parse_op_response()?; // error
            panic!("sql response error"); // not reach
        }

        let count = utils::bytes_to_buint32(&self.recv_packets(4)?);
        if count != 0 {
            let mut n = xsqlda_len / 8;
            if xsqlda.len() % 8 != 0 {
                n += 1;
            }

            let mut null_indicator: u128 = 0;
            let b = &self.recv_packets_alignment(n)?;
            for c in b.iter().rev() {
                null_indicator <<= 8;
                null_indicator += *c as u128
            }
            for x in xsqlda.iter() {
                let ln = if x.io_length() < 0 {
                    utils::bytes_to_buint32(&self.recv_packets(4)?) as usize
                } else {
                    x.io_length() as usize
                };
                let raw_value = self.recv_packets_alignment(ln as usize)?;
                row.push(x.value(&raw_value)?);
            }
        }

        Ok(row)
    }

    pub fn create_blob(&mut self, value: &[u8], trans_handle: i32) -> Result<Vec<u8>, Error> {
        let buf = self.suspend_buffer();
        let blob_handle: i32;
        let blob_id: Vec<u8>;
        self.op_create_blob2(trans_handle)?;

        match self.op_response() {
            Ok((h, id, _)) => {
                blob_handle = h;
                blob_id = id;
            }
            Err(e) => {
                self.resume_buffer(&buf);
                return Err(e);
            }
        }

        let mut i: usize = 0;
        while i < value.len() {
            let mut end: usize = i + BLOB_SEGMENT_SIZE;
            if end > value.len() {
                end = value.len()
            }
            self.op_put_segment(blob_handle, &value[i..end])?;
            self.op_response()?;
            i += BLOB_SEGMENT_SIZE;
        }

        self.resume_buffer(&buf);
        self.op_close_blob(blob_handle)?;
        self.op_response()?;

        Ok(blob_id)
    }

    pub fn params_to_blr(
        &mut self,
        trans_handle: i32,
        params: &Vec<Param>,
    ) -> Result<(Vec<u8>, Vec<u8>), Error> {
        // Convert parameter array to BLR and values format.
        let mut values_list: Vec<u8> = Vec::new();
        let mut blr_list: Vec<u8> = Vec::new();
        let ln = params.len() * 2;
        let blr = vec![5, 2, 4, 0, (ln & 0xFF) as u8, ((ln >> 8) & 0xFF) as u8];
        blr_list.write(&blr)?;

        let mut null_indicator: u128 = 0;
        for (i, p) in params.iter().enumerate() {
            if *p == Param::Null {
                null_indicator |= 1 << i;
            }
        }

        let mut n = params.len() / 8;
        if params.len() % 8 != 0 {
            n += 1;
        }
        if (n % 4) != 0 {
            // padding
            n += 4 - n % 4;
        }

        for _ in 0..n {
            values_list.push((null_indicator & 255) as u8);
            null_indicator >>= 8;
        }

        for p in params.iter() {
            let v: Vec<u8> = Vec::new();
            match p {
                Param::Null => {
                    values_list.write(&[14, 0, 0])?;
                }
                Param::Text(s) => {
                    let b = s.as_bytes();
                    let (blr, v) = utils::bytes_to_blr(b);
                    values_list.write(&v)?;
                    blr_list.write(&blr)?;
                }
                Param::Long(n) => {
                    values_list.write(&utils::bint32_to_bytes(*n))?;
                    blr_list.write(&[8, 0])?;
                }
                _ => {
                    // TODO:
                    panic!("another to parameter");
                }
            }
            blr_list.write(&[7, 0])?;
            values_list.write(&v)?;
        }

        blr_list.write(&[255, 76])?;
        Ok((blr_list, values_list))
    }
}

impl Drop for WireProtocol {
    fn drop(&mut self) {
        self.op_detach().unwrap();
        let _ = self.op_response();
    }
}
