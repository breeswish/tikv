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

extern crate kvproto;

use kvproto::kvrpcpb::Context;

pub fn new_no_cache_context() -> Context {
    let mut ctx = Context::new();
    ctx.set_not_fill_cache(true);
    ctx
}

/// Whether or not env variable TIKV_BENCH_FULL_PAYLOAD = 1, indicating using full payload to
/// run benchmarks.
pub fn use_full_payload() -> bool {
    if let Ok(s) = ::std::env::var("TIKV_BENCH_FULL_PAYLOAD") {
        s == "1"
    } else {
        false
    }
}
