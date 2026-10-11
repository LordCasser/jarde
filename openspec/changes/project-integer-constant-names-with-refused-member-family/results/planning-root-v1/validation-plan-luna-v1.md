# CF12 same-class integer-constant validation plan

## Scope and pins

This is a root-run plan for the v2 private candidate only: one existing class-source gate widens `member_family` from exact `Absent` to `Absent | Refused { child: None }`, plus one reader/API test. The candidate diff is `/private/tmp/jarde-cf12-same-class-constant-impl-luna-v2/facade.rs.patch` (SHA-256 `691cb92af554788bda04dd4a6b0236c4db325ef9a158f26c8ca5abe1f499981c`); v2 remains private until root applies/reviews it. The pushed base is `6476b56c357443ef17318891b12142f509977234`; the independent CI run for that commit is still pending. After applying and formatting the candidate, record the *actual* `src/facade.rs` SHA as the candidate pin; do not use v2's private-copy SHA as a post-format product pin.

The test input is the already frozen root class `/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/harness-v3-java/capture/TestSwitchLabels.test/input/TestSwitchLabels$TestCls.class` (753 bytes, SHA-256 `4997ca33261d0585a1008dd860131c5ed3f2438206209bb7acb82c90eb743d22`). The committed harness-v3 capture manifest SHA-256 is `5ffa56c72b2df664d21cb7e9afa30c819cf4a5071cb009c9839e3ae17724f281`. CI command source is `.github/workflows/ci.yml` SHA-256 `b545890a55cc4fda31d70b332889c111d789afbf0949581e96bf2b94ea6cd18a`. The existing render/full-replay runners are pinned below; use private copies/new output paths if their fixed output locations already exist.

## Guard

Keep the existing v9 guard file unchanged and verify its pinned SHA-256 `51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33`. For command execution, load its existing `run_command` and inject the private adapter from `/private/tmp/jarde-target-size-scan-luna-v1/target_size_scan.py` into the loaded module as `target_bytes = lambda: regular_file_bytes(ROOT / "target")`; adapter SHA-256 is `f3e6d8a596ff1c645921c9bdbd26c604ef3d3553f72edb6adbd9179a3014a325`. The adapter only replaces the racy size scan: retain the 5 GiB free-space floor, 1 GiB target limit, `start_new_session=True`, one-second poll, and process-group TERM/KILL cleanup. Preflight the free-space/target bounds before starting. Catch only `FileNotFoundError` during per-path `stat`; other scan errors should fail the run. Use unique private raw-output/evidence paths. For successful test commands, reuse the v9 test-binary cleanup helper if the target approaches the limit; pin every removal and keep the source tree unchanged.

## Minimal Rust validation

Run these in order from the repository root, under the guard above:

```sh
cargo fmt --all -- --check
```

Run CI's exact workspace Clippy invocation from the pinned workflow, including its existing `-A clippy::<lint>` debt list and final `-D warnings`:

```sh
cargo clippy --workspace --all-targets --all-features --locked -- \
  -A clippy::too_many_arguments \
  -A clippy::cloned_ref_to_slice_refs \
  -A clippy::collapsible_if \
  -A clippy::type_complexity \
  -A clippy::len_zero \
  -A clippy::needless_option_as_deref \
  -A clippy::needless_borrow \
  -A clippy::useless_conversion \
  -A clippy::large_enum_variant \
  -A clippy::question_mark \
  -A clippy::comparison_to_empty \
  -A clippy::op_ref \
  -A clippy::manual_range_patterns \
  -A clippy::if_same_then_else \
  -A clippy::filter_map_bool_then \
  -A clippy::filter_next \
  -A clippy::unneeded_struct_pattern \
  -A clippy::redundant_guards \
  -A clippy::map_identity \
  -A clippy::redundant_slicing \
  -A clippy::unnecessary_get_then_check \
  -A clippy::unnecessary_unwrap \
  -A clippy::redundant_locals \
  -A clippy::replace_box \
  -A clippy::map_clone \
  -A clippy::unnecessary_mut_passed \
  -A clippy::single_element_loop \
  -A clippy::unnecessary_to_owned \
  -A clippy::needless_lifetimes \
  -D warnings
```

Then run the existing integer-constant module, which includes the new frozen-reader test and its duplicate/shadow/invalid-name regressions:

```sh
cargo test -p jarde --lib integer_constant_name_tests --locked
```

Run the real API family boundaries already named by the change design:

```sh
cargo test -p jarde --test member_family_identity --locked
cargo test -p jarde --test class_source --locked
cargo test -p jarde --test p3_nested_annotation_source --locked
cargo test -p jarde --lib nested_enum_ --locked
```

These targets cover RefusedNone/childSome/Prepared/RefusedPair and sibling-family behavior through public reader/API paths. They are separate from the single CF12 integer-candidate test; do not add a copied gate test or claim every sibling fixture also has integer candidates.

## Candidate CLI freeze and frozen Labels replay

Build the just-tested candidate through the guarded command path:

```sh
cargo build -p jarde-cli --locked
```

Copy `target/debug/jarde-cli` to a fresh private path such as `/private/tmp/jarde-cf12-same-class-constant-cli-v1`, set mode `0555`, and record its SHA-256 plus the candidate `src/facade.rs` SHA, base commit, fixture SHA, guard/adapter pins, and build command evidence. Do not overwrite an existing CLI/evidence directory.

Reuse, rather than redesign, the frozen input/replay assets:

- `harness-v3-java/capture/TestSwitchLabels.test/input/` supplies the original `TestSwitchLabels$TestCls.class` and `$Inner.class` bytes; the immutable capture was already recorded by manifest `5ffa56c72b2df664d21cb7e9afa30c819cf4a5071cb009c9839e3ae17724f281`. The upstream source/capture source is held in the same evidence tree.
- `jarde-cf12-render-baseline-root-v1.py` (SHA-256 `21ba6aa495974bc155f794ebb8b5968fa1ff6328ce53a4e1d4f66469bf4e11ea`) shows the existing CLI requests and output paths for `class-source --input ... --class <internal-name> --policy single-class --release 8 --format json`, in both default and `--evidence all` modes. Make a private copy pointed at the newly frozen candidate CLI, a new output directory, and only `TestSwitchLabels.test`; retain its existing process-group/disk guard and redirect its target-size callback through the adapter above.
- `jarde-cf12-full-replay-root-v1.py` (SHA-256 `64a7b25a50800d158f509ce87c8e3cf84fdb34261d6fe233a920084a0e3d195c`) performs the existing original/JADX/Jarde compile and runtime comparison with `-Xverify:all`. Make only the private path/case-filter changes needed to consume the two candidate CLI sources and target `TestSwitchLabels.test`; keep the same frozen probe, JDK pin, helper classpath, and comparison code. Do not rerun the upstream JUnit capture (`jarde-cf12-direct-harness-root-v3.py`, SHA-256 `7b0fdd758dee03b42d696eddf079a8d57ec3edda69a53772ca08b5b988aeacfe`) because its pinned capture already exists.

The observable target is narrow: candidate root source writes `case CONST_ABC:` and `return CONST_CDE;`, while its physical method report remains numeric; its member family remains RefusedNone and does not embed `Inner`. Original, fixed JADX, and candidate Jarde sources must compile and the candidate's runtime stdout/stderr must match the original under the frozen input probe. Record the actual observations and do not infer whole-CF12 completion from this one class.

No workspace, Git, Cargo, JDK, JADX, or CLI actions were performed while preparing this plan.
