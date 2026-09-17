#![no_main]

//! Bounded query fuzz entry (design 3.3).
//!
//! The whole input is the artifact. `Engine::open` may reject arbitrary bytes through the
//! public error path — that is the documented outcome for a damaged container, not a
//! fuzzing failure — and the targets only assert the contract once a snapshot really
//! opened. Every opened snapshot gets all five fixed requests under the hard limits in
//! `jarde_fuzz::limits`, independent of the file magic. The shared driver checks and drops
//! each result before the next request; findings remain reproducible from the input alone.

use jarde_fuzz::exercise_query;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    exercise_query(data, |_, _| {});
});
