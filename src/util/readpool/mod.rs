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

mod config;
mod context;
mod priority;

use std::time::Duration;
use futures::Future;
use futures_cpupool as cpupool;

use util;
use util::futurepool::FuturePool;

pub use self::config::Config;
pub use self::context::Context;
pub use self::priority::Priority;

const TICK_INTERVAL_SEC: u64 = 1;

#[derive(Clone)]
pub struct ReadPool {
    pool_high: FuturePool<Context>,
    pool_normal: FuturePool<Context>,
    pool_low: FuturePool<Context>,
}

impl util::AssertSend for ReadPool {}
impl util::AssertSync for ReadPool {}

impl ReadPool {
    pub fn new(config: &Config) -> ReadPool {
        let tick_interval = Duration::from_secs(TICK_INTERVAL_SEC);
        let build_context_factory = || || Context {};
        ReadPool {
            pool_high: FuturePool::new(
                config.high_concurrency,
                config.stack_size.0 as usize,
                "readpool-high",
                tick_interval,
                build_context_factory(),
            ),
            pool_normal: FuturePool::new(
                config.normal_concurrency,
                config.stack_size.0 as usize,
                "readpool-normal",
                tick_interval,
                build_context_factory(),
            ),
            pool_low: FuturePool::new(
                config.low_concurrency,
                config.stack_size.0 as usize,
                "readpool-low",
                tick_interval,
                build_context_factory(),
            ),
        }
    }

    #[inline]
    fn get_pool_by_priority(&self, priority: Priority) -> &FuturePool<Context> {
        match priority {
            Priority::High => &self.pool_high,
            Priority::Normal => &self.pool_normal,
            Priority::Low => &self.pool_low,
        }
    }

    pub fn future_execute<F>(
        &self,
        priority: Priority,
        future: F,
    ) -> cpupool::CpuFuture<F::Item, F::Error>
    where
        F: Future + Send + 'static,
        F::Item: Send + 'static,
        F::Error: Send + 'static,
    {
        // TODO: handle busy?
        let pool = self.get_pool_by_priority(priority);
        pool.spawn(future)
    }
}

#[cfg(test)]
mod tests {
    use std::error;
    use std::result;
    use std::fmt;
    use futures::{future, Future};

    pub use super::*;

    type BoxError = Box<error::Error + Send + Sync>;

    pub fn expect_val<T>(v: T, x: result::Result<T, BoxError>)
    where
        T: PartialEq + fmt::Debug + 'static,
    {
        assert!(x.is_ok());
        assert_eq!(x.unwrap(), v);
    }

    pub fn expect_err<T>(desc: &str, x: result::Result<T, BoxError>) {
        assert!(x.is_err());
        match x {
            Err(e) => assert_eq!(e.description(), desc),
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_future_execute() {
        let read_pool = ReadPool::new(&Config::default());

        expect_val(
            vec![1, 2, 4],
            read_pool
                .future_execute(
                    Priority::High,
                    future::ok::<Vec<u8>, BoxError>(vec![1, 2, 4]),
                )
                .wait(),
        );

        expect_err(
            "foobar",
            read_pool
                .future_execute(
                    Priority::High,
                    future::err::<(), BoxError>(box_err!("foobar")),
                )
                .wait(),
        );
    }
}
