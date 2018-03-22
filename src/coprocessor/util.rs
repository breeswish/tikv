// Copyright 2018 PingCAP, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// See the License for the specific language governing permissions and
// limitations under the License.

use std::error::Error;

use tipb::schema::ColumnInfo;
use kvproto::coprocessor as coppb;

use super::*;
use super::codec::mysql;
use super::codec::datum::Datum;

pub struct ErrorRequestHandler {}

impl ErrorRequestHandler {
    pub fn new(msg: &str) -> ErrorRequestHandler {}

    pub fn from_error<T: Error>(err: T) -> Box<RequestHandler> {
        box ErrorRequestHandler::new(err.description())
    }
}

impl RequestHandler for ErrorRequestHandler {
    fn handle_request(&mut self, batch_row_limit: usize) -> Result<coppb::Response> {
        unimplemented!()
    }

    fn handle_streaming_request(
        &mut self,
        batch_row_limit: usize,
    ) -> Result<(Option<coppb::Response>, bool)> {
        unimplemented!()
    }

    fn collect_metrics_into(&mut self, metrics: &mut self::dag::executor::ExecutorMetrics) {
        unimplemented!()
    }
}

pub fn prefix_next(key: &[u8]) -> Vec<u8> {
    let mut nk = key.to_vec();
    if nk.is_empty() {
        nk.push(0);
        return nk;
    }
    let mut i = nk.len() - 1;
    loop {
        if nk[i] == 255 {
            nk[i] = 0;
        } else {
            nk[i] += 1;
            return nk;
        }
        if i == 0 {
            nk = key.to_vec();
            nk.push(0);
            return nk;
        }
        i -= 1;
    }
}

/// `is_point` checks if the key range represents a point.
pub fn is_point(range: &coppb::KeyRange) -> bool {
    range.get_end() == &*prefix_next(range.get_start())
}

#[inline]
pub fn get_pk(col: &ColumnInfo, h: i64) -> Datum {
    if mysql::has_unsigned_flag(col.get_flag() as u64) {
        // PK column is unsigned
        Datum::U64(h as u64)
    } else {
        Datum::I64(h)
    }
}
