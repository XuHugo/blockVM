// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use prometheus::{register_int_counter, IntCounter};
use once_cell::sync::Lazy;

/// Count of times the module publishing fallback was triggered in parallel execution.
pub static MODULE_PUBLISHING_FALLBACK_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "aptos_execution_module_publishing_fallback_count",
        "Count times module was read and written in parallel execution (sequential fallback)"
    )
    .unwrap()
});

/// Count of speculative transaction re-executions due to a failed validation .
pub static SPECULATIVE_ABORT_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "aptos_execution_speculative_abort_count",
        "Number of speculative aborts in parallel execution (leading to re-execution)"
    )
    .unwrap()
});

/// Count of speculative transaction re-executions due to a failed validation .
pub static V_version_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "V_version_COUNT",
        "Number of speculative aborts in parallel execution (leading to re-execution)"
    )
    .unwrap()
});

/// Count of speculative transaction re-executions due to a failed validation .
pub static V_Resolved_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "V_Resolved_COUNT",
        "Number of speculative aborts in parallel execution (leading to re-execution)"
    )
    .unwrap()
});

/// Count of speculative transaction re-executions due to a failed validation .
pub static V_Dependency_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "V_Dependency_COUNT",
        "Number of speculative aborts in parallel execution (leading to re-execution)"
    )
    .unwrap()
});

pub static V_Unresolved_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "V_Unresolved_COUNT",
        "Number of speculative aborts in parallel execution (leading to re-execution)"
    )
    .unwrap()
});

pub static V_NotFound_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "V_NotFound_COUNT",
        "Number of speculative aborts in parallel execution (leading to re-execution)"
    )
    .unwrap()
});

pub static V_DeltaApplicationFailure_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "V_DeltaApplicationFailure_COUNT",
        "Number of speculative aborts in parallel execution (leading to re-execution)"
    )
    .unwrap()
});