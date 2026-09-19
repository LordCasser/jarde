#![no_main]

//! Bounded P2 method-analysis fuzz entry (design 5.3).
//!
//! Same input contract as the P1 targets: the whole input is the artifact and a rejected input
//! is an accepted outcome. What is new is that the request cannot be a fixed one — the method
//! to analyze has to be derived from the input — so the shared driver reads a class header the
//! input itself provides, takes its first member that declares a body, and drives every fixed
//! request shape against it. Each shape gets its own budget (an ample one, or one that must
//! stop the pipeline after the read) and asserts the public contracts of the report: the
//! scheduled phase prefix and its stop rule, the execution that explains the stop, the single
//! phase whose completion raises `semantic_validation`, the one artifact that makes the report
//! `Conservative`, and the budget bound of every counted dimension. An artifact with no such
//! member ends the run without a request, and a public error never skips a shape.

use jarde_fuzz::exercise_method_analysis;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    exercise_method_analysis(data, |_, _| {});
});
