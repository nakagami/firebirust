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

mod cellvalue;
mod conn_params;
mod connection;
mod crypt_translater;
mod decfloat;
mod errmsgs;
mod error;
mod param;
mod params;
mod row;
mod srp;
mod statement;
mod transaction;
mod tz_map;
mod utils;
mod wirechannel;
mod wireprotocol;
mod xsqlvar;

pub use crate::connection::Connection;
pub use crate::error::Error;
pub use crate::param::Param;
pub use crate::param::ToSqlParam;

// Protocol Types (accept_type)
const PTYPE_BATCH_SEND: i32 = 3; // Batch sends, no asynchrony
const PTYPE_OUT_OF_BAND: i32 = 4; // Batch sends w/ out of band notification
const PTYPE_LAZY_SEND: i32 = 5; // Deferred packets delivery

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

// statement
pub(crate) const ISC_INFO_SQL_STMT_SELECT: u32 = 1;
pub(crate) const ISC_INFO_SQL_STMT_INSERT: u32 = 2;
pub(crate) const ISC_INFO_SQL_STMT_UPDATE: u32 = 3;
pub(crate) const ISC_INFO_SQL_STMT_DELETE: u32 = 4;
pub(crate) const ISC_INFO_SQL_STMT_DDL: u32 = 5;
pub(crate) const ISC_INFO_SQL_STMT_GET_SEGMENT: u32 = 6;
pub(crate) const ISC_INFO_SQL_STMT_PUT_SEGMENT: u32 = 7;
pub(crate) const ISC_INFO_SQL_STMT_EXEC_PROCEDURE: u32 = 8;
pub(crate) const ISC_INFO_SQL_STMT_START_TRANS: u32 = 9;
pub(crate) const ISC_INFO_SQL_STMT_COMMIT: u32 = 10;
pub(crate) const ISC_INFO_SQL_STMT_ROLLBACK: u32 = 11;
pub(crate) const ISC_INFO_SQL_STMT_SELECT_FOR_UPD: u32 = 12;
pub(crate) const ISC_INFO_SQL_STMT_SET_GENERATOR: u32 = 13;
pub(crate) const ISC_INFO_SQL_STMT_SAVEPOINT: u32 = 14;

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
#[macro_export]
macro_rules! params {
    () => {
        &[] as &[&dyn $crate::ToSqlParam]
    };
    ($($param:expr),*) => {
        &[$(&$crate::Param::from($param) as &dyn $crate::ToSqlParam),*] as &[&dyn $crate::ToSqlParam]
    };
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_timezone;
