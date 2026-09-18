//! The fixture files this crate's unit tests read, addressed from the crate root.
//!
//! The fixtures live at `<repository>/tests/fixtures`, which is where the integration tests
//! expect them too, and only one copy exists. What a source file must not do is spell the path
//! out relative to itself: `include_bytes!("../tests/fixtures/…")` is correct only while the
//! file sits in `src/`, and the layering work moves these files into `crates/<name>/src/`, where
//! the same literal would resolve inside the new crate and fail to compile.
//!
//! The depth therefore appears **once per crate**, in the two items below, and every call site
//! names only the fixture. Moving a file into a crate means editing this module, not hunting
//! literals; and a call site that forgot to be updated cannot compile, because the segment it
//! would need is not there to omit.
//!
//! The gate is `test` plus the same `test-support` feature as the shared class builder: after
//! the layering, a dependent crate's tests read fixtures through this crate, and their
//! `cfg(test)` cannot see anything here.

/// Embeds one fixture at compile time, named relative to the fixtures root.
///
/// Use it as `fixture!("historical/ecj-4.6.1/v52/HistoricalControlFlow.class")`. A missing
/// fixture is a compile error, exactly as a mis-relative path was before.
///
/// The allow is for the build where `test-support` is on but `test` is not — `--all-features`
/// compiles this crate that way — and every caller lives in a `cfg(test)` block.
#[cfg(any(test, feature = "test-support"))]
#[allow(unused_macros, reason = "callers are all in this crate's test builds")]
macro_rules! fixture {
    ($relative:literal) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/",
            $relative
        ))
    };
}

#[cfg(any(test, feature = "test-support"))]
#[allow(unused_imports, reason = "the macro is imported by test modules")]
pub(crate) use fixture;

/// The fixtures root as a path, for the tests that walk the tree instead of naming a file.
#[cfg(any(test, feature = "test-support"))]
#[allow(
    dead_code,
    reason = "used by this crate's tests, or by a dependent crate's test build"
)]
pub(crate) fn fixtures_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}
