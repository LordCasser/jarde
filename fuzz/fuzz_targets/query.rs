#![no_main]

//! Bounded query fuzz entry (design 3.3).
//!
//! The whole input is the artifact. `Engine::open` may reject arbitrary bytes through the
//! public error path — that is the documented outcome for a damaged container, not a
//! fuzzing failure — and the targets only assert the contract once a snapshot really
//! opened. An opened snapshot is then queried with one of a small set of *fixed* requests
//! (selected from the input's first byte) under the hard limits in `jarde_fuzz::limits`,
//! so a finding is reproducible from the input bytes alone. No reader, mutation engine or
//! request generator is implemented here.

use jarde_fuzz::{assert_query_contract, limits, open, query_request, run_query};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let limits = limits();
    let Some(snapshot) = open(data, &limits) else {
        return;
    };
    let request = query_request(data.first().copied().unwrap_or(0), &snapshot);
    let Some(report) = run_query(&snapshot, &request, &limits) else {
        return;
    };
    assert_query_contract(&report, &limits);
});
