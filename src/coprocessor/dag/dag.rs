// Copyright 2017 PingCAP, Inc.
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

use std::mem;
use std::sync::Arc;

use tipb::schema::ColumnInfo;
use tipb::select::{Chunk, DAGRequest, SelectResponse};
use kvproto::coprocessor::{KeyRange, Response};
use protobuf::{Message as PbMsg, RepeatedField};

use coprocessor::codec::mysql;
use coprocessor::codec::datum::{Datum, DatumEncoder};
use coprocessor::select::xeval::EvalContext;
use coprocessor::{Error, Result};
use coprocessor::endpoint::{get_pk, prefix_next, to_pb_error, ReqContext};
use storage::{Snapshot, SnapshotStore, Statistics};

use super::executor::{build_exec, Executor, Row};

pub struct DAGContext {
    columns: Arc<Vec<ColumnInfo>>,
    has_aggr: bool,
    req_ctx: ReqContext,
    exec: Box<Executor>,
    output_offsets: Vec<u32>,
    batch_row_limit: usize,
    chunks_per_stream: usize,
    chunks: Vec<Chunk>,
}

impl DAGContext {
    pub fn new(
        mut req: DAGRequest,
        ranges: Vec<KeyRange>,
        snap: Box<Snapshot>,
        req_ctx: ReqContext,
        batch_row_limit: usize,
        chunks_per_stream: usize,
    ) -> Result<DAGContext> {
        let eval_ctx = Arc::new(box_try!(EvalContext::new(
            req.get_time_zone_offset(),
            req.get_flags()
        )));
        let store = SnapshotStore::new(
            snap,
            req.get_start_ts(),
            req_ctx.isolation_level,
            req_ctx.fill_cache,
        );

        let dag_executor = build_exec(req.take_executors().into_vec(), store, ranges, eval_ctx)?;
        Ok(DAGContext {
            columns: dag_executor.columns,
            has_aggr: dag_executor.has_aggr,
            req_ctx: req_ctx,
            exec: dag_executor.exec,
            output_offsets: req.take_output_offsets(),
            batch_row_limit: batch_row_limit,
            chunks_per_stream: chunks_per_stream,
            chunks: Vec::new(),
        })
    }

    pub fn handle_request(&mut self, streaming: bool) -> Result<(Response, bool)> {
        let mut record_cnt = 0;
        let (mut first_row, mut start_key) = (true, None);
        loop {
            match self.exec.next() {
                Ok(Some(row)) => {
                    self.req_ctx.check_if_outdated()?;

                    if first_row {
                        first_row = false;
                        start_key = self.exec.take_last_key();
                    }

                    let mut stream_result = None;
                    if self.chunks.is_empty() || record_cnt >= self.batch_row_limit {
                        if streaming && self.chunks.len() >= self.chunks_per_stream {
                            let start_key = start_key.take();
                            let end_key = self.exec.take_last_key();
                            stream_result = Some(self.make_response(true, start_key, end_key));
                        }
                        self.chunks.push(Chunk::new());
                        record_cnt = 0;
                    }
                    record_cnt += 1;

                    let chunk = self.chunks.last_mut().unwrap();
                    if self.has_aggr {
                        chunk.mut_rows_data().extend_from_slice(&row.data.value);
                    } else {
                        let value = inflate_cols(&row, &self.columns, &self.output_offsets)?;
                        chunk.mut_rows_data().extend_from_slice(&value);
                    }
                    if let Some(stream_result) = stream_result {
                        return stream_result;
                    }
                }
                Ok(None) => {
                    let end_key = self.exec.take_last_key();
                    return self.make_response(false, start_key, end_key);
                }
                Err(e) => if let Error::Other(_) = e {
                    let mut resp = Response::new();
                    let mut sel_resp = SelectResponse::new();
                    sel_resp.set_error(to_pb_error(&e));
                    resp.set_data(box_try!(sel_resp.write_to_bytes()));
                    resp.set_other_error(format!("{}", e));
                    return Ok((resp, false));
                } else {
                    return Err(e);
                },
            }
        }
    }

    fn make_response(
        &mut self,
        remain: bool,
        start_key: Option<Vec<u8>>,
        end_key: Option<Vec<u8>>,
    ) -> Result<(Response, bool)> {
        let chunks = mem::replace(&mut self.chunks, Vec::new());
        let mut resp = Response::new();
        let mut sel_resp = SelectResponse::new();
        sel_resp.set_chunks(RepeatedField::from_vec(chunks));
        let data = box_try!(sel_resp.write_to_bytes());
        resp.set_data(data);

        let (start, end) = match (start_key, end_key) {
            (Some(start_key), Some(end_key)) => if start_key > end_key {
                (end_key, prefix_next(&start_key))
            } else {
                (start_key, prefix_next(&end_key))
            },
            (Some(start_key), None) => {
                let end_key = prefix_next(&start_key);
                (start_key, end_key)
            }
            (None, None) => return Ok((resp, remain)),
            _ => unreachable!(),
        };
        let mut range = KeyRange::new();
        range.set_start(start);
        range.set_end(end);
        resp.set_range(range);
        Ok((resp, remain))
    }

    pub fn collect_statistics_into(&mut self, statistics: &mut Statistics) {
        self.exec.collect_statistics_into(statistics);
    }
}

#[inline]
fn inflate_cols(row: &Row, cols: &[ColumnInfo], output_offsets: &[u32]) -> Result<Vec<u8>> {
    let data = &row.data;
    // TODO capacity is not enough
    let mut values = Vec::with_capacity(data.value.len());
    for offset in output_offsets {
        let col = &cols[*offset as usize];
        let col_id = col.get_column_id();
        match data.get(col_id) {
            Some(value) => values.extend_from_slice(value),
            None if col.get_pk_handle() => {
                let pk = get_pk(col, row.handle);
                box_try!(values.encode(&[pk], false));
            }
            None if col.has_default_val() => {
                values.extend_from_slice(col.get_default_val());
            }
            None if mysql::has_not_null_flag(col.get_flag() as u64) => {
                return Err(box_err!("column {} of {} is missing", col_id, row.handle));
            }
            None => {
                box_try!(values.encode(&[Datum::Null], false));
            }
        }
    }
    Ok(values)
}
