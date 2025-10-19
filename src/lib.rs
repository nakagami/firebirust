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

mod connection_async;
mod statement_async;
mod transaction_async;
mod wirechannel_async;
mod wireprotocol_async;

pub use crate::connection::Connection;
pub use crate::connection_async::ConnectionAsync;
pub use crate::error::Error;
pub use crate::param::Param;
pub use crate::param::ToSqlParam;

// Column type
pub const SQL_TYPE_TEXT: u32 = 452;
pub const SQL_TYPE_VARYING: u32 = 448;
pub const SQL_TYPE_SHORT: u32 = 500;
pub const SQL_TYPE_LONG: u32 = 496;
pub const SQL_TYPE_FLOAT: u32 = 482;
pub const SQL_TYPE_DOUBLE: u32 = 480;
pub const SQL_TYPE_D_FLOAT: u32 = 530;
pub const SQL_TYPE_TIMESTAMP: u32 = 510;
pub const SQL_TYPE_BLOB: u32 = 520;
pub const SQL_TYPE_ARRAY: u32 = 540;
pub const SQL_TYPE_QUAD: u32 = 550;
pub const SQL_TYPE_TIME: u32 = 560;
pub const SQL_TYPE_DATE: u32 = 570;
pub const SQL_TYPE_INT64: u32 = 580;
pub const SQL_TYPE_INT128: u32 = 32752;
pub const SQL_TYPE_TIMESTAMP_TZ: u32 = 32754;
pub const SQL_TYPE_TIME_TZ: u32 = 32756;
pub const SQL_TYPE_DEC_FIXED: u32 = 32758;
pub const SQL_TYPE_DEC64: u32 = 32760;
pub const SQL_TYPE_DEC128: u32 = 32762;
pub const SQL_TYPE_BOOLEAN: u32 = 32764;
pub const SQL_TYPE_NULL: u32 = 32766;

// Protocol Types (accept_type)
pub(crate) const PTYPE_BATCH_SEND: u32 = 3; // Batch sends, no asynchrony
pub(crate) const PTYPE_OUT_OF_BAND: u32 = 4; // Batch sends w/ out of band notification
pub(crate) const PTYPE_LAZY_SEND: u32 = 5; // Deferred packets delivery
pub(crate) const PTYPE_MASK: u32 = 0xFF;
pub(crate) const PFLAG_COMPRESS: u32 = 0x100;
pub(crate) const PFLAG_WIN_SSPI_NEGO: u32 = 0x200;

pub(crate) const ISC_ARG_END: u32 = 0;
pub(crate) const ISC_ARG_GDS: u32 = 1;
pub(crate) const ISC_ARG_STRING: u32 = 2;
pub(crate) const ISC_ARG_CSTRING: u32 = 3;
pub(crate) const ISC_ARG_NUMBER: u32 = 4;
pub(crate) const ISC_ARG_INTERPRETED: u32 = 5;
pub(crate) const ISC_ARG_VMS: u32 = 6;
pub(crate) const ISC_ARG_UNIX: u32 = 7;
pub(crate) const ISC_ARG_DOMAIN: u32 = 8;
pub(crate) const ISC_ARG_DOS: u32 = 9;
pub(crate) const ISC_ARG_MPEXL: u32 = 10;
pub(crate) const ISC_ARG_MPEXL_IPC: u32 = 11;
pub(crate) const ISC_ARG_NEXT_MACH: u32 = 15;
pub(crate) const ISC_ARG_NETWARE: u32 = 16;
pub(crate) const ISC_ARG_WIN32: u32 = 17;
pub(crate) const ISC_ARG_WARNING: u32 = 18;
pub(crate) const ISC_ARG_SQL_STATE: u32 = 19;

pub(crate) const ISC_INFO_END: u8 = 1;
pub(crate) const ISC_INFO_TRUNCATED: u8 = 2;
pub(crate) const ISC_INFO_ERROR: u8 = 3;
pub(crate) const ISC_INFO_DATA_NOT_READY: u8 = 4;
pub(crate) const ISC_INFO_LENGTH: u8 = 126;
pub(crate) const ISC_INFO_FLAG_END: u8 = 127;

pub(crate) const ISC_INFO_DB_ID: u8 = 4;
pub(crate) const ISC_INFO_READS: u8 = 5;
pub(crate) const ISC_INFO_WRITES: u8 = 6;
pub(crate) const ISC_INFO_FETCHES: u8 = 7;
pub(crate) const ISC_INFO_MARKS: u8 = 8;
pub(crate) const ISC_INFO_IMPLEMENTATION: u8 = 11;
pub(crate) const ISC_INFO_VERSION: u8 = 12;
pub(crate) const ISC_INFO_BASE_LEVEL: u8 = 13;
pub(crate) const ISC_INFO_PAGE_SIZE: u8 = 14;
pub(crate) const ISC_INFO_NUM_BUFFERS: u8 = 15;
pub(crate) const ISC_INFO_LIMBO: u8 = 16;
pub(crate) const ISC_INFO_CURRENT_MEMORY: u8 = 17;
pub(crate) const ISC_INFO_MAX_MEMORY: u8 = 18;
pub(crate) const ISC_INFO_WINDOW_TURNS: u8 = 19;
pub(crate) const ISC_INFO_LICENSE: u8 = 20;
pub(crate) const ISC_INFO_ALLOCATION: u8 = 21;
pub(crate) const ISC_INFO_ATTACHMENT_ID: u8 = 22;
pub(crate) const ISC_INFO_READ_SEQ_COUNT: u8 = 23;
pub(crate) const ISC_INFO_READ_IDX_COUNT: u8 = 24;
pub(crate) const ISC_INFO_INSERT_COUNT: u8 = 25;
pub(crate) const ISC_INFO_UPDATE_COUNT: u8 = 26;
pub(crate) const ISC_INFO_DELETE_COUNT: u8 = 27;
pub(crate) const ISC_INFO_BACKOUT_COUNT: u8 = 28;
pub(crate) const ISC_INFO_PURGE_COUNT: u8 = 29;
pub(crate) const ISC_INFO_EXPUNGE_COUNT: u8 = 30;
pub(crate) const ISC_INFO_SWEEP_INTERVAL: u8 = 31;
pub(crate) const ISC_INFO_ODS_VERSION: u8 = 32;
pub(crate) const ISC_INFO_ODS_MINOR_VERSION: u8 = 33;
pub(crate) const ISC_INFO_NO_RESERVE: u8 = 34;
pub(crate) const ISC_INFO_LOGFILE: u8 = 35;
pub(crate) const ISC_INFO_CUR_LOGFILE_NAME: u8 = 36;
pub(crate) const ISC_INFO_CUR_LOG_PART_OFFSET: u8 = 37;
pub(crate) const ISC_INFO_NUM_WAL_BUFFERS: u8 = 38;
pub(crate) const ISC_INFO_WAL_BUFFER_SIZE: u8 = 39;
pub(crate) const ISC_INFO_WAL_CKPT_LENGTH: u8 = 40;
pub(crate) const ISC_INFO_WAL_CUR_CKPT_INTERVAL: u8 = 41;
pub(crate) const ISC_INFO_WAL_PRV_CKPT_FNAME: u8 = 42;
pub(crate) const ISC_INFO_WAL_PRV_CKPT_POFFSET: u8 = 43;
pub(crate) const ISC_INFO_WAL_RECV_CKPT_FNAME: u8 = 44;
pub(crate) const ISC_INFO_WAL_RECV_CKPT_POFFSET: u8 = 45;
pub(crate) const ISC_INFO_WAL_GRPC_WAIT_USECS: u8 = 47;
pub(crate) const ISC_INFO_WAL_NUM_IO: u8 = 48;
pub(crate) const ISC_INFO_WAL_AVG_IO_SIZE: u8 = 49;
pub(crate) const ISC_INFO_WAL_NUM_COMMITS: u8 = 50;
pub(crate) const ISC_INFO_WAL_AVG_GRPC_SIZE: u8 = 51;
pub(crate) const ISC_INFO_FORCED_WRITES: u8 = 52;
pub(crate) const ISC_INFO_USER_NAMES: u8 = 53;
pub(crate) const ISC_INFO_PAGE_ERRORS: u8 = 54;
pub(crate) const ISC_INFO_RECORD_ERRORS: u8 = 55;
pub(crate) const ISC_INFO_BPAGE_ERRORS: u8 = 56;
pub(crate) const ISC_INFO_DPAGE_ERRORS: u8 = 57;
pub(crate) const ISC_INFO_IPAGE_ERRORS: u8 = 58;
pub(crate) const ISC_INFO_PPAGE_ERRORS: u8 = 59;
pub(crate) const ISC_INFO_TPAGE_ERRORS: u8 = 60;
pub(crate) const ISC_INFO_SET_PAGE_BUFFERS: u8 = 61;
pub(crate) const ISC_INFO_DB_SQL_DIALECT: u8 = 62;
pub(crate) const ISC_INFO_DB_READ_ONLY: u8 = 63;
pub(crate) const ISC_INFO_DB_SIZE_IN_PAGES: u8 = 64;
pub(crate) const ISC_INFO_ATT_CHARSET: u8 = 101;
pub(crate) const ISC_INFO_DB_CLASS: u8 = 102;
pub(crate) const ISC_INFO_FIREBIRD_VERSION: u8 = 103;
pub(crate) const ISC_INFO_OLDEST_TRANSACTION: u8 = 104;
pub(crate) const ISC_INFO_OLDEST_ACTIVE: u8 = 105;
pub(crate) const ISC_INFO_OLDEST_SNAPSHOT: u8 = 106;
pub(crate) const ISC_INFO_NEXT_TRANSACTION: u8 = 107;
pub(crate) const ISC_INFO_DB_PROVIDER: u8 = 108;
pub(crate) const ISC_INFO_ACTIVE_TRANSACTIONS: u8 = 109;
pub(crate) const ISC_INFO_ACTIVE_TRAN_COUNT: u8 = 110;
pub(crate) const ISC_INFO_CREATION_DATE: u8 = 111;
pub(crate) const ISC_INFO_DB_FILE_SIZE: u8 = 112;

// SQL information items
pub(crate) const ISC_INFO_SQL_SELECT: u8 = 4;
pub(crate) const ISC_INFO_SQL_BIND: u8 = 5;
pub(crate) const ISC_INFO_SQL_NUM_VARIABLES: u8 = 6;
pub(crate) const ISC_INFO_SQL_DESCRIBE_VARS: u8 = 7;
pub(crate) const ISC_INFO_SQL_DESCRIBE_END: u8 = 8;
pub(crate) const ISC_INFO_SQL_SQLDA_SEQ: u8 = 9;
pub(crate) const ISC_INFO_SQL_MESSAGE_SEQ: u8 = 10;
pub(crate) const ISC_INFO_SQL_TYPE: u8 = 11;
pub(crate) const ISC_INFO_SQL_SUB_TYPE: u8 = 12;
pub(crate) const ISC_INFO_SQL_SCALE: u8 = 13;
pub(crate) const ISC_INFO_SQL_LENGTH: u8 = 14;
pub(crate) const ISC_INFO_SQL_NULL_IND: u8 = 15;
pub(crate) const ISC_INFO_SQL_FIELD: u8 = 16;
pub(crate) const ISC_INFO_SQL_RELATION: u8 = 17;
pub(crate) const ISC_INFO_SQL_OWNER: u8 = 18;
pub(crate) const ISC_INFO_SQL_ALIAS: u8 = 19;
pub(crate) const ISC_INFO_SQL_SQLDA_START: u8 = 20;
pub(crate) const ISC_INFO_SQL_STMT_TYPE: u8 = 21;
pub(crate) const ISC_INFO_SQL_GET_PLAN: u8 = 22;
pub(crate) const ISC_INFO_SQL_RECORDS: u8 = 23;
pub(crate) const ISC_INFO_SQL_BATCH_FETCH: u8 = 24;

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
pub(crate) const ISC_DPB_VERSION1: u8 = 1;
pub(crate) const ISC_DPB_PAGE_SIZE: u8 = 4;
pub(crate) const ISC_DPB_NUM_BUFFERS: u8 = 5;
pub(crate) const ISC_DPB_FORCE_WRITE: u8 = 24;
pub(crate) const ISC_DPB_USER_NAME: u8 = 28;
pub(crate) const ISC_DPB_PASSWORD: u8 = 29;
pub(crate) const ISC_DPB_PASSWORD_ENC: u8 = 30;
pub(crate) const ISC_DPB_LC_CTYPE: u8 = 48;
pub(crate) const ISC_DPB_OVERWRITE: u8 = 54;
pub(crate) const ISC_DPB_CONNECT_TIMEOUT: u8 = 57;
pub(crate) const ISC_DPB_DUMMY_PACKET_INTERVAL: u8 = 58;
pub(crate) const ISC_DPB_SQL_ROLE_NAME: u8 = 60;
pub(crate) const ISC_DPB_SET_PAGE_BUFFERS: u8 = 61;
pub(crate) const ISC_DPB_SQL_DIALECT: u8 = 63;
pub(crate) const ISC_DPB_SET_DB_CHARSET: u8 = 68;
pub(crate) const ISC_DPB_PROCESS_ID: u8 = 71;
pub(crate) const ISC_DPB_NO_DB_TRIGGERS: u8 = 72;
pub(crate) const ISC_DPB_TRUSTED_AUTH: u8 = 73;
pub(crate) const ISC_DPB_PROCESS_NAME: u8 = 74;
pub(crate) const ISC_DPB_UTF8_FILENAME: u8 = 77;
pub(crate) const ISC_DPB_SPECIFIC_AUTH_DATA: u8 = 84;
pub(crate) const ISC_DPB_AUTH_PLUGIN_LIST: u8 = 85;
pub(crate) const ISC_DPB_AUTH_PLUGIN_NAME: u8 = 86;
pub(crate) const ISC_DPB_CONFIG: u8 = 87;
pub(crate) const ISC_DPB_NOLINGER: u8 = 88;
pub(crate) const ISC_DPB_RESET_ICU: u8 = 89;
pub(crate) const ISC_DPB_MAP_ATTACH: u8 = 90;
pub(crate) const ISC_DPB_SESSION_TIME_ZONE: u8 = 91;

pub(crate) const OP_CONNECT: u32 = 1;
pub(crate) const OP_EXIT: u32 = 2;
pub(crate) const OP_ACCEPT: u32 = 3;
pub(crate) const OP_REJECT: u32 = 4;
pub(crate) const OP_PROTOCROL: u32 = 5;
pub(crate) const OP_DISCONNECT: u32 = 6;
pub(crate) const OP_RESPONSE: u32 = 9;
pub(crate) const OP_ATTACH: u32 = 19;
pub(crate) const OP_CREATE: u32 = 20;
pub(crate) const OP_DETACH: u32 = 21;
pub(crate) const OP_TRANSACTION: u32 = 29;
pub(crate) const OP_COMMIT: u32 = 30;
pub(crate) const OP_ROLLBACK: u32 = 31;
pub(crate) const OP_OPEN_BLOB: u32 = 35;
pub(crate) const OP_GET_SEGMENT: u32 = 36;
pub(crate) const OP_PUT_SEGMENT: u32 = 37;
pub(crate) const OP_CLOSE_BLOB: u32 = 39;
pub(crate) const OP_INFO_DATABASE: u32 = 40;
pub(crate) const OP_INFO_TRANSACTION: u32 = 42;
pub(crate) const OP_BATCH_SEGMENTS: u32 = 44;
pub(crate) const OP_QUE_EVENTS: u32 = 48;
pub(crate) const OP_CANCEL_EVENTS: u32 = 49;
pub(crate) const OP_COMMIT_RETAINING: u32 = 50;
pub(crate) const OP_EVENT: u32 = 52;
pub(crate) const OP_CONNECT_REQUEST: u32 = 53;
pub(crate) const OP_OPEN_BLOB2: u32 = 56;
pub(crate) const OP_CREATE_BLOB2: u32 = 57;
pub(crate) const OP_ALLOCATE_STATEMENT: u32 = 62;
pub(crate) const OP_EXECUTE: u32 = 63;
pub(crate) const OP_EXEC_IMMEDIATE: u32 = 64;
pub(crate) const OP_FETCH: u32 = 65;
pub(crate) const OP_FETCH_RESPONSE: u32 = 66;
pub(crate) const OP_FREE_STATEMENT: u32 = 67;
pub(crate) const OP_PREPARE_STATEMENT: u32 = 68;
pub(crate) const OP_INFO_SQL: u32 = 70;
pub(crate) const OP_DUMMY: u32 = 71;
pub(crate) const OP_EXECUTE2: u32 = 76;
pub(crate) const OP_SQL_RESPONSE: u32 = 78;
pub(crate) const OP_DROP_DATABASE: u32 = 81;
pub(crate) const OP_SERVICE_ATTACH: u32 = 82;
pub(crate) const OP_SERVICE_DETACH: u32 = 83;
pub(crate) const OP_SERVICE_INFO: u32 = 84;
pub(crate) const OP_SERVICE_START: u32 = 85;
pub(crate) const OP_ROLLBACK_RETAINING: u32 = 86;
// FB3
pub(crate) const OP_UPDATE_ACCOUNT_INFO: u32 = 87;
pub(crate) const OP_AUTHENTICATE_USER: u32 = 88;
pub(crate) const OP_PARTIAL: u32 = 89;
pub(crate) const OP_TRUSTED_AUTH: u32 = 90;
pub(crate) const OP_CANCEL: u32 = 91;
pub(crate) const OP_CONT_AUTH: u32 = 92;
pub(crate) const OP_PING: u32 = 93;
pub(crate) const OP_ACCEPT_DATA: u32 = 94;
pub(crate) const OP_ABORT_AUX_CONNECTION: u32 = 95;
pub(crate) const OP_CRYPT: u32 = 96;
pub(crate) const OP_CRYPT_KEY_CALLBACK: u32 = 97;
pub(crate) const OP_COND_ACCEPT: u32 = 98;

pub(crate) const CNCT_USER: u8 = 1;
pub(crate) const CNCT_PASSWD: u8 = 2;
pub(crate) const CNCT_HOST: u8 = 4;
pub(crate) const CNCT_GROUP: u8 = 5;
pub(crate) const CNCT_USER_VERIFICATION: u8 = 6;
pub(crate) const CNCT_SPECIFIC_DATA: u8 = 7;
pub(crate) const CNCT_PLUGIN_NAME: u8 = 8;
pub(crate) const CNCT_LOGIN: u8 = 9;
pub(crate) const CNCT_PLUGIN_LIST: u8 = 10;
pub(crate) const CNCT_CLIENT_CRYPT: u8 = 11;

pub(crate) const ISC_TPB_VERSION1: u8 = 1;
pub(crate) const ISC_TPB_VERSION3: u8 = 3;
pub(crate) const ISC_TPB_CONSISTENCY: u8 = 1;
pub(crate) const ISC_TPB_CONCURRENCY: u8 = 2;
pub(crate) const ISC_TPB_SHARED: u8 = 3;
pub(crate) const ISC_TPB_PROTECTED: u8 = 4;
pub(crate) const ISC_TPB_EXCLUSIVE: u8 = 5;
pub(crate) const ISC_TPB_WAIT: u8 = 6;
pub(crate) const ISC_TPB_NOWAIT: u8 = 7;
pub(crate) const ISC_TPB_READ: u8 = 8;
pub(crate) const ISC_TPB_WRITE: u8 = 9;
pub(crate) const ISC_TPB_LOCK_READ: u8 = 10;
pub(crate) const ISC_TPB_LOCK_WRITE: u8 = 11;
pub(crate) const ISC_TPB_VERB_TIME: u8 = 12;
pub(crate) const ISC_TPB_COMMIT_TIME: u8 = 13;
pub(crate) const ISC_TPB_IGNORE_LIMBO: u8 = 14;
pub(crate) const ISC_TPB_READ_COMMITTED: u8 = 15;
pub(crate) const ISC_TPB_AUTOCOMMIT: u8 = 16;
pub(crate) const ISC_TPB_REC_VERSION: u8 = 17;
pub(crate) const ISC_TPB_NO_REC_VERSION: u8 = 18;
pub(crate) const ISC_TPB_RESTART_REQUESTS: u8 = 19;
pub(crate) const ISC_TPB_NO_AUTO_UNDO: u8 = 20;
pub(crate) const ISC_TPB_LOCK_TIMEOUT: u8 = 21;

pub(crate) const ISC_INFO_REQ_SELECT_COUNT: u32 = 13;
pub(crate) const ISC_INFO_REQ_INSERT_COUNT: u32 = 14;
pub(crate) const ISC_INFO_REQ_UPDATE_COUNT: u32 = 15;
pub(crate) const ISC_INFO_REQ_DELETE_COUNT: u32 = 16;

pub(crate) const ISC_INFO_SVC_SVR_DB_INFO: u32 = 50;
pub(crate) const ISC_INFO_SVC_GET_LICENSE: u32 = 51;
pub(crate) const ISC_INFO_SVC_GET_LICENSE_MASK: u32 = 52;
pub(crate) const ISC_INFO_SVC_GET_CONFIG: u32 = 53;
pub(crate) const ISC_INFO_SVC_VERSION: u32 = 54;
pub(crate) const ISC_INFO_SVC_SERVER_VERSION: u32 = 55;
pub(crate) const ISC_INFO_SVC_IMPLEMENTATION: u32 = 56;
pub(crate) const ISC_INFO_SVC_CAPABILITIES: u32 = 57;
pub(crate) const ISC_INFO_SVC_USER_DBPATH: u32 = 58;
pub(crate) const ISC_INFO_SVC_GET_ENV: u32 = 59;
pub(crate) const ISC_INFO_SVC_GET_ENV_LOCK: u32 = 60;
pub(crate) const ISC_INFO_SVC_GET_ENV_MSG: u32 = 61;
pub(crate) const ISC_INFO_SVC_LINE: u32 = 62;
pub(crate) const ISC_INFO_SVC_TO_EOF: u32 = 63;
pub(crate) const ISC_INFO_SVC_TIMEOUT: u32 = 64;
pub(crate) const ISC_INFO_SVC_GET_LICENSED_USERS: u32 = 65;
pub(crate) const ISC_INFO_SVC_LIMBO_TRANS: u32 = 66;
pub(crate) const ISC_INFO_SVC_RUNNING: u32 = 67;
pub(crate) const ISC_INFO_SVC_GET_USERS: u32 = 68;

// Transaction informatino items
pub(crate) const ISC_INFO_TRA_ID: u32 = 4;
pub(crate) const ISC_INFO_TRA_OLDEST_INTERESTING: u32 = 5;
pub(crate) const ISC_INFO_TRA_OLDEST_SNAPSHOT: u32 = 6;
pub(crate) const ISC_INFO_TRA_OLDEST_ACTIVE: u32 = 7;
pub(crate) const ISC_INFO_TRA_ISOLATION: u32 = 8;
pub(crate) const ISC_INFO_TRA_ACCESS: u32 = 9;
pub(crate) const ISC_INFO_TRA_LOCK_TIMEOUT: u32 = 10;
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
mod test_async;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_timezone;
