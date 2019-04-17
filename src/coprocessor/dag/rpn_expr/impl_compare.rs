// Copyright 2019 TiKV Project Authors. Licensed under Apache-2.0.

use super::types::RpnFnCallPayload;
use crate::coprocessor::dag::expr::EvalContext;
use crate::coprocessor::Result;

#[derive(Debug, Clone, Copy)]
pub struct RpnFnEQReal;

impl_template_fn! { 2 arg @ RpnFnEQReal }

impl RpnFnEQReal {
    #[allow(clippy::float_cmp)]
    #[inline]
    fn call(
        _ctx: &mut EvalContext,
        _payload: RpnFnCallPayload<'_>,
        arg0: &Option<f64>,
        arg1: &Option<f64>,
    ) -> Result<Option<i64>> {
        // FIXME: It really should be a `Result<Option<f64>>`.
        Ok(match (arg0, arg1) {
            (Some(ref arg0), Some(ref arg1)) => Some((*arg0 == *arg1) as i64),
            // TODO: Use `partial_cmp`.
            _ => None,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RpnFnEQInt;

impl_template_fn! { 2 arg @ RpnFnEQInt }

impl RpnFnEQInt {
    #[inline]
    fn call(
        _ctx: &mut EvalContext,
        _payload: RpnFnCallPayload<'_>,
        arg0: &Option<i64>,
        arg1: &Option<i64>,
    ) -> Result<Option<i64>> {
        // FIXME: The algorithm here is incorrect. We should care about unsigned and signed.
        Ok(match (arg0, arg1) {
            (Some(ref arg0), Some(ref arg1)) => Some((*arg0 == *arg1) as i64),
            _ => None,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RpnFnGTInt;

impl_template_fn! { 2 arg @ RpnFnGTInt }

impl RpnFnGTInt {
    #[inline]
    fn call(
        _ctx: &mut EvalContext,
        _payload: RpnFnCallPayload<'_>,
        arg0: &Option<i64>,
        arg1: &Option<i64>,
    ) -> Result<Option<i64>> {
        // FIXME: The algorithm here is incorrect. We should care about unsigned and signed.
        Ok(match (arg0, arg1) {
            (Some(ref arg0), Some(ref arg1)) => Some((*arg0 > *arg1) as i64),
            _ => None,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RpnFnLTInt;

impl_template_fn! { 2 arg @ RpnFnLTInt }

impl RpnFnLTInt {
    #[inline]
    fn call(
        _ctx: &mut EvalContext,
        _payload: RpnFnCallPayload<'_>,
        arg0: &Option<i64>,
        arg1: &Option<i64>,
    ) -> Result<Option<i64>> {
        // FIXME: The algorithm here is incorrect. We should care about unsigned and signed.
        Ok(match (arg0, arg1) {
            (Some(ref arg0), Some(ref arg1)) => Some((*arg0 < *arg1) as i64),
            _ => None,
        })
    }
}
