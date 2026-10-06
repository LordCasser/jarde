# The first full workspace pass (exit 101) — the four deltas it found, verbatim

Command: cargo test --workspace --all-targets --all-features --locked --no-fail-fast
Totals: exit 101, 319 `test result: ok`, 4 `test result: FAILED`.

test a_store_in_a_loop_test_remains_refused ... FAILED

failures:

---- a_store_in_a_loop_test_remains_refused stdout ----

thread 'a_store_in_a_loop_test_remains_refused' (56662167) panicked at tests/p3_loop_test_values.rs:117:5:
the test assignment is not moved into or out of a loop:
    int storeTest(int arg1) {
        // jarde: not recovered: the recovery run for `storeTest(I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method storeTest(I)I
        // @declaration an instance method of `LoopTestValues`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 3
        // the loop@1 rule did not claim the block at BCI 0: it requires a test block whose every instruction is part of a value expression, and the instruction at BCI 2 is not part of one, so presenting the structure would have moved that effect out of the shape it decides
        // @bytecode 6 9 10
        // 2 live block(s) are reachable only through edges the normal-flow view leaves out: [6, 9]
    }

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    a_store_in_a_loop_test_remains_refused

test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s


test corpus_files_match_the_recorded_fingerprint ... FAILED

failures:

---- corpus_files_match_the_recorded_fingerprint stdout ----
corpus fingerprint: 1510 files
  tests/fixtures: 1494 files
  fuzz/corpus: 16 files
  dimension versions (版本): 13 carriers, pinned
  dimension compiler (编译器): 10 carriers, pinned
  dimension packaging (打包): 11 carriers, partial
  dimension identity (身份): 4 carriers, partial
  dimension bytecode (字节码): 10 carriers, pinned
  dimension recovery (恢复): 37 carriers, pinned
  dimension degradation (退化): 10 carriers, pinned
  dimension adversarial (对抗): 9 carriers, partial

thread 'corpus_files_match_the_recorded_fingerprint' (56666728) panicked at tests/p5_corpus_fingerprint.rs:827:5:
the corpus moved under the fingerprint (9 file(s)):
unlisted: tests/fixtures/recover-dup-store-conditional/DS.java (694, blake3 29cb957d6998c8560b6567aeab2dfdbbf48bb335886d003395f07fdebadb7797) is in the corpus but not in the manifest
unlisted: tests/fixtures/recover-dup-store-conditional/NEG.java (504, blake3 c28445b25aa9bd86edeb21299e0ae3455943e913611896be46fe53aabf0c7ab4) is in the corpus but not in the manifest
unlisted: tests/fixtures/recover-dup-store-conditional/REF.java (380, blake3 d34133ed72a3335fba1cb26cdff491325c040e40713edf2a724c5018395bf469) is in the corpus but not in the manifest
unlisted: tests/fixtures/recover-dup-store-conditional/v8-javac8/DS.class (1421, blake3 500c4175dad4dbd42a2485b97d8c0410d74d17ba73b1c6059e90967fd9ba6f78) is in the corpus but not in the manifest
unlisted: tests/fixtures/recover-dup-store-conditional/v8-javac8/NEG.class (1300, blake3 0263227d54e960870bf656ac790d91e42ba65c7467107bdfbd274fbd32f8c76d) is in the corpus but not in the manifest
unlisted: tests/fixtures/recover-dup-store-conditional/v8-javac8/REF.class (1146, blake3 637ddf1701bbe1c288311c8f0d7bf93892dfff909fd2f2b7966697264c7914ff) is in the corpus but not in the manifest
unlisted: tests/fixtures/recover-dup-store-conditional/v8/DS.class (1421, blake3 e1b60f3c8159181b6de2f3cf53622f806b62ced05d9229b446862d131851a834) is in the corpus but not in the manifest
unlisted: tests/fixtures/recover-dup-store-conditional/v8/NEG.class (1297, blake3 fccde06b3e9491e5516f3bfe383a717d461dc966bb59eab57bb1513c1471c661) is in the corpus but not in the manifest
unlisted: tests/fixtures/recover-dup-store-conditional/v8/REF.class (1143, blake3 2cb9f47936f3bedf8af34cf4b4244d518bd80ee6cdde767f2d2b50906e5dcaf7) is in the corpus but not in the manifest

If the change is intended, rerun `cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint` and review the diff. If it is not, the corpus is the thing to fix — do not re-record to make this pass.
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    corpus_files_match_the_recorded_fingerprint

test result: FAILED. 4 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.29s


test cf18_verifier_valid_near_misses_refuse_the_whole_method ... ok

failures:

---- a_reused_array_length_value_keeps_the_loop_test_quoted stdout ----

thread 'a_reused_array_length_value_keeps_the_loop_test_quoted' (56669837) panicked at crates/jarde-java/tests/p3_java_recovery.rs:1270:5:
the repeated-value operation remains the reason for refusal: [RegionRecord { bci: 0, structured: true, blocks: [0], code: None, message: None, rule: Some(RuleVersion { rule: "straight", version: "1" }) }, RegionRecord { bci: 5, structured: true, blocks: [5, 11], code: None, message: None, rule: Some(RuleVersion { rule: "loop", version: "1" }) }, RegionRecord { bci: 17, structured: true, blocks: [17], code: None, message: None, rule: Some(RuleVersion { rule: "straight", version: "1" }) }]

---- an_array_length_duplicate_carried_out_of_the_test_keeps_the_loop_quoted stdout ----

thread 'an_array_length_duplicate_carried_out_of_the_test_keeps_the_loop_quoted' (56669846) panicked at crates/jarde-java/tests/p3_java_recovery.rs:1312:5:
the stack-copy operation blocks condition projection: [RegionRecord { bci: 0, structured: true, blocks: [0], code: None, message: None, rule: Some(RuleVersion { rule: "straight", version: "1" }) }, RegionRecord { bci: 4, structured: true, blocks: [4, 11], code: None, message: None, rule: Some(RuleVersion { rule: "loop", version: "1" }) }, RegionRecord { bci: 17, structured: true, blocks: [17], code: None, message: None, rule: Some(RuleVersion { rule: "straight", version: "1" }) }]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    a_reused_array_length_value_keeps_the_loop_test_quoted
    an_array_length_duplicate_carried_out_of_the_test_keeps_the_loop_quoted

test result: FAILED. 43 passed; 2 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.05s


test classfile::tests::repository_class_fixtures_validate_without_false_target_rejections ... FAILED

failures:

---- classfile::tests::repository_class_fixtures_validate_without_false_target_rejections stdout ----

thread 'classfile::tests::repository_class_fixtures_validate_without_false_target_rejections' (56670446) panicked at crates/jarde-reader/src/classfile.rs:11863:9:
assertion `left == right` failed: fixture population changed: re-measure these counts
  left: (789, 3311, 286, 2061, 8)
 right: (783, 3275, 286, 2023, 8)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    classfile::tests::repository_class_fixtures_validate_without_false_target_rejections

test result: FAILED. 177 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

