#![no_main]

//! Bounded artifact-tree and multi-release fuzz entry (design 3.3).
//!
//! Same input contract as the `query` target: the whole input is the artifact, a rejected
//! input is an accepted outcome, and an opened snapshot is driven through the public
//! nested-tree enumeration and the standard multi-release selection with a fixed Java 17
//! runtime view and the hard limits in `jarde_fuzz::limits`. Both calls get their own
//! budget so one stopping early does not hide the other's paths.

use jarde_fuzz::{
    assert_multi_release_contract, assert_tree_contract, limits, open, run_multi_release, run_tree,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let limits = limits();
    let Some(snapshot) = open(data, &limits) else {
        return;
    };
    if let Some(tree) = run_tree(&snapshot, &limits) {
        assert_tree_contract(&tree, &limits);
    }
    if let Some(selection) = run_multi_release(&snapshot, &limits) {
        assert_multi_release_contract(&selection, &limits);
    }
});
