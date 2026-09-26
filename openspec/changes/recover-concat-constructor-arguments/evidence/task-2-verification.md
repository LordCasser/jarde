# Tasks 2.1–2.3 verification

The construction proof now receives the complete read-only `concat::Plan`. An inner allocation is stepped over only when the physical constructor argument is the chain's exact `toString` result, the chain owns its complete contiguous interval, every member is in that argument's SSA dependency, the result has one use at the constructor, and the outer expression plus consumer share handler coverage. The outer site's `owned` set remains disjoint from concat ownership.

| Case | Test | Expected result |
| --- | --- | --- |
| `thrown` and `constructed` compose, with no duplicate ownership | `init::tests::a_verified_concat_is_composed_only_as_the_unique_constructor_argument` | Site accepted; chain tail is argument; outer and concat ownership disjoint |
| `reserved` BCIs without a concat proof | same test | `jre_new_interleaved_effect` |
| a non-tail String value inside the chain is offered as argument evidence | same test | no chain selected; only the exact tail can qualify |
| one tail is supplied as two argument roots | same test | `jre_new_concat_argument` |
| chain starts at the outer `dup` boundary | same test | `jre_new_concat_argument` |
| a handler row covers the concat but not the outer allocation | same test | `jre_new_concat_exception_boundary` |
| independent void call between allocation and constructor | `p3_ordinary_new_invokes::an_independent_void_call_refuses_the_construction_and_keeps_all_origins` | `jre_new_interleaved_effect`; bytecode BCIs and source origins retained |
| construction result has two physical readers | `p3_new_value::a_leftover_two_instructions_read_keeps_its_refusal` | `new@1` refuses the multi-consumer object |

Focused Rust checks passed:

```text
cargo test -p jarde-java --lib init::tests::a_verified_concat_is_composed_only_as_the_unique_constructor_argument
cargo test --test p3_ordinary_new_invokes --test p3_new_value --test p3_concat_conversion
```

`cargo fmt --all -- --check` passed. `cargo clippy --locked -p jarde-java --all-targets -- -D warnings` did not pass: after removing warnings in this change, Clippy reported 26 library and 31 all-target findings in `enumswitch.rs`, `region.rs`, `report.rs`, `build.rs`, and `reuse.rs`. The `region.rs` and `build.rs` findings are in the concurrent intermediate-join work; the other findings are outside this change. No Clippy finding remained in the concat-constructor additions.

The complete CLI replay, Java 8 compilation, three five-line runtime outputs, identical class hashes, and default/all source comparison are recorded in [`task-3-1-replay`](task-3-1-replay/README.md).
