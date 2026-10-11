# Handoff note: hoisted iterator full-workspace validation

Read-only synthesis for root. This note describes evidence already present in the shared checkout and proposes document updates only after root's third fresh full-workspace execution is available. It does not edit project files or change product scope/counts.

## Keep the two fresh full-workspace failures separate

Both receipts declare `no_carried_execution: true`, candidate CLI SHA256 `8bdb868ab58e4c9196970438c71de529996c66ba826c84fe216794d328fdfc9a`, initial source closure 4732, expected 362 targets / 96 batches / 99 commands including final clean, and status `failed`.

- `results/workspace-failure-root-v1/actual/execution.json` started 2026-10-11 20:11:08.972 UTC. It reached batch 75 (command index 75; 3 passed, 1 failed, 0 ignored in `recover_unicode_identifiers`), then ran guarded `cargo clean` as command 76 successfully; observed 77 commands including cleanup. Target was absent afterward. The failure was `the_ascii_presentations_only_change_in_the_two_proved_f1_for_headers` at `tests/recover_unicode_identifiers.rs:442`: generated `SB.cond(Ljava/util/List;)Ljava/lang/String;` had the new Object foreach, while the frozen expected golden still showed the prior explicit Iterator/while form. Raw: `actual/75.stdout.raw` has the full left/right text and assertion; `actual/75.stderr.raw` records Cargo's failed target. This was not a complete workspace pass.
- `results/workspace-failure-root-v2/actual/execution.json` started 2026-10-11 20:26:58.178 UTC, independently fresh. It got past the corrected Unicode batch 75 (11 passed, 0 failed, 2 ignored), then command 85 failed in `p3_loop_exit_gateways::one_arm_straight_prefix_loop_and_optional_tail_are_closed`; observed 87 commands including successful guarded `cargo clean` command 86, with target absent. The legacy assertion expected `countEmpty`'s explicit Iterator/while/hasNext/next presentation. Actual text at `crates/jarde-java/tests/p3_loop_exit_gateways.rs:1196` was a complete `for (java.lang.Object iteratorElement23 : arg0)` retaining the String cast, nested `isEmpty`, increment, and return. `actual/85.stdout.raw` contains the full failing method and test name; `actual/85.stderr.raw` records Cargo's target failure. The runner's `JSONDecodeError` is secondary: it tried parsing the panic's `{`-prefixed text as a Cargo JSON artifact. Do not describe this parse error as the primary failure, and do not treat this run as a pass.

The v1 and v2 receipts are independent failures. Do not combine a successful prefix from one with a suffix from the other, and do not describe either as complete workspace validation.

## Full-class evidence for the two expectation corrections

These are separate bounded comparison receipts, not workspace results.

- `results/unicode-golden-root-v1/actual-runtime/execution.json` is `passed_fixed_sb_cases_only`, 15 guarded commands / 30 raw streams. Commands 3–8 are javac8 original, old CLI, and candidate CLI compile/run; 9–14 repeat on javac23. The six runtime stdout streams at command indexes 4, 6, 8, 10, 12, 14 have the same SHA256 `f4eca06d83fe380a3e4323494e110892d5a249fde094269c82d877a3a43efcf6` and exact six-line oracle: main `a,b,/yy/[1, 2]/[1, 0]/p-q/9/6`, empty `empty=`, null `NullPointerException`, strings `bbccc`, null element `NullPointerException`, wrong element `java.lang.ClassCastException`. Both javac legs therefore agree across original class, frozen old CLI source, and candidate source. The saved `source.diff` changes only `SB.cond`: removes its Iterator local and explicit while, introduces the Object foreach and retains the String cast and body. `new-default.json` and `new-all.json` are captured. Scope is these six fixed cases; this is not a general semantic-equivalence claim or a dedicated SSA trace.
- `results/gateway-expectation-root-v1/actual-runtime/execution.json` is likewise `passed_fixed_sb_cases_only` (the fixture is `VariablePostfixLoop`, despite the generic status label), 15 guarded commands / 30 raw streams. The six runtime stdout streams at indexes 4, 6, 8, 10, 12, 14 share SHA256 `3366b89f76c3d7dd1296f18f7b933bdf7ff1755bb03230d87db1a39a018e98c4` and exactly match `null=0`, `empty=0`, `mixed=2`, `repeat=1`, `null-element=NullPointerException`. The javac8 and javac23 original/old CLI/candidate runs agree. `source.diff` changes only `countEmpty`: explicit Iterator/while becomes the Object foreach, retaining the String cast, inner `isEmpty`, increment, null guard, and return. README records that only stale positive structural expectations were corrected; preserve default/all equality, map equality, `[6,7]` header anchor, `[6,13,22,39,42]` common structured owner and independent negative controls.

For both fixes, call them updates to test expectations after direct whole-class cross-JDK runtime comparison. Do not frame the runtime receipts as proof that all workspace targets pass.

## Suggested handoff / verification-document edits after the third run

1. In `handoff.md`, update the current front-most hoisted-iterator status to record both fresh failures and the separate full-class comparisons. Keep the previous partial-run records as historical evidence; do not rewrite them into a successful validation.
2. In `openspec/changes/recover-proved-hoisted-iterator-foreach/verification-root.md`, add a concise chronological paragraph after the existing workspace-failure paragraph: v1 failed at the stale Unicode golden; the exact single-method golden correction was supported by the 15-command/30-stream SB whole-class comparison on javac8/javac23; v2 passed that target and failed at the stale gateway positive assertion; the exact structural expectation correction was supported by the corresponding 15-command/30-stream VariablePostfixLoop whole-class comparison. Link the two failure READMEs and two comparison READMEs. Append the real third-run result only from its actual receipt and root acceptance, if present; if it failed, name its exact target and retain failed status.
3. Keep the existing four result READMEs and all raw files. They are accurate records of partial failures or bounded comparison scope. Do not edit task checkboxes, product scope, CF11 progress or the fixed 71-unit / 612-file counts based on these observations.
4. Avoid any wording that says “full workspace passed” until root has the actual third fresh receipt and independently accepts its complete expected target/batch/command inventory, final clean, source pins, and status. The two bounded runtime receipts cannot fill that gap.

## Raw files checked

- `results/workspace-failure-root-v1/actual/{execution.json,75.stdout.raw,75.stderr.raw}`
- `results/workspace-failure-root-v2/actual/{execution.json,75.stdout.raw,85.stdout.raw,85.stderr.raw}`
- `results/unicode-golden-root-v1/actual-runtime/{execution.json,source.diff,4.stdout.raw,6.stdout.raw,8.stdout.raw,10.stdout.raw,12.stdout.raw,14.stdout.raw}`
- `results/gateway-expectation-root-v1/actual-runtime/{execution.json,source.diff,4.stdout.raw,6.stdout.raw,8.stdout.raw,10.stdout.raw,12.stdout.raw,14.stdout.raw}`

The first v1 stdout is verbose Cargo JSON plus the full assertion left/right strings; v2's `85.stdout.raw` contains the actual emitted method. README summaries were cross-checked against these raw records and receipt metadata.
