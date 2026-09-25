# Java 8 `do-while` source-only audit

This audit uses only the frozen CLI `/tmp/jarde-cli-deferred-budget-908c`. Its SHA-256 was checked
before each case and is recorded in `cli-sha256.txt` and `summary.json`:
`908c472560d6146354e1fd679c595e884afadf5ec7051727b51e79a1f95c6570`. No Cargo command, CLI
rebuild, production edit, generated-source edit, or OpenSpec edit was performed.

`run_audit.py` is the replay entry point. It compiles every original source with
`javac --release 8 -g:none`, compiles each independent runner, runs the original class and raw
JADX output with `java -Xverify:all`, then runs the frozen CLI's raw full-class output through the
same compiler and runner. Raw generated sources are copied unchanged to temporary paths ending in
`.java`; the `*.java.txt` files in this directory are the untouched captured output. Compiler
stderr, exit statuses, bytecode, runtime output, unified diffs, hashes, and the complete replay
summary are retained.

## Results

The smallest positive case is `positive/DoWhilePositive.java`. It contains an ordinary body and a
side-effecting latch condition: `touch()` increments `checks` and is called on every condition
evaluation. The original, JADX, and frozen jarde full-class sources all compile with exit `0`,
verify, and run with exit `0`; all four output lines match byte-for-byte. The output SHA-256 is
`28e0e52277783b453f2a615767eef04cc213afe30f1e17a74027293651e4b6bc`. The frozen jarde source has
zero `@bytecode` markers. This closes the ordinary `do-while` and the observed effectful condition
subset as healthy in this source-only boundary.

`core/DoWhileCore.java` is the seven-line control-flow probe: three ordinary body runs, two
`continue` runs, and two `break` runs. Original and JADX both compile, verify, and run with exit
`0`, with matching output SHA-256
`5f3fa46c4a5fd6eebc38027bb8926e5e8500869574c35e4fbef01df2c057bee5`. Frozen jarde class-source
returns `0`, but unchanged generated source compilation returns `1` with two `missing return
statement` diagnostics. No frozen runtime is attempted. The exact locale-specific diagnostics and
warnings are preserved in `core/jarde-javac.log`; the generated body shows the loop refusal and
the quoted blocks in `core/jarde.java.txt`.

The full `DoWhileAudit.java` combines ordinary body, `continue`, `break`, and the effectful latch
condition in one class and has nine runner lines. Original and JADX compile/run successfully and
match byte-for-byte; output SHA-256 is
`8783e96b04133881325205d57b52b6538cd8a14cd13d48ec4c70bcd264a1257a`. Frozen jarde class-source
returns `0`, with four `@bytecode` markers. Its unchanged full-class `javac` returns `1` with two
missing-return diagnostics, both caused by `withContinue` and `withBreak`; `effectfulCondition`
is emitted as a complete `do { … } while (touch() + local1 < arg0)` method and has no marker. The
full generated output is therefore retained as a failed all-method compile, while the positive
case supplies the complete three-way runtime comparison for the effectful condition itself.

Class hashes are in `summary.json`: `DoWhilePositive.class` is 532 bytes with SHA-256
`5c012250591c603adc1524e7721319fb284ed1e3f8480c067925348309e0b6ff`, `DoWhileCore.class` is 498
bytes with SHA-256 `9339878366306e65530aee489e15ca5ead92c4ff637b614141d9a8e552cb7a60`, and
`DoWhileAudit.class` is 702 bytes with SHA-256
`d729768d3f533ee24790e1822dcaca609f84105a0ae6e0cc158bb3a1a2e1bff4`. Each case has its own
`original-javap.txt`, raw JADX source/report, raw jarde source/report, compiler logs, runtime
outputs, and diffs.

## Architectural boundary

The existing region layer already has a dedicated `LoopForm::DoWhile`. Its proof requires one latch,
two latch successors with an outside exit, a decoded comparison, a body walk that covers every
iterating block, and the latch test checks in
[`region.rs:2383`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/region.rs:2383).
The builder emits a `StmtKind::DoWhile` after rendering the latch test and body in source order
([`build.rs:1424`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:1424)).
Its frame model already describes an edge back to the loop test as a `continue`
([`region.rs:1004`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/region.rs:1004)),
and the fallback taxonomy has distinct `LoopShape` and `LoopLeavesEarly` reasons.

The positive case shows that the existing `DoWhile` region, local updates, a side-effecting call in
the latch expression, and the per-iteration effect order are sufficient for this bounded shape. The
control core shows the remaining boundary: an inner `if` whose edge targets the latch (`continue`)
or the loop exit (`break`) causes the loop proof to quote live blocks instead of publishing a
complete method. The generated source is not patched to make it compile. This is a loop edge
ownership/shape proof gap in the current frozen implementation, rather than an opcode gap: the
original bytecode uses ordinary `iinc`, comparisons, `goto`, arithmetic, field stores, and return.

The source contract for `test_is_pure` only admits value-producing test-block instructions
([`region.rs:2091`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/region.rs:2091)).
In this fixture the side-effecting `touch()` call is a separate predecessor immediately before the
latch comparison, and the observed region output keeps it in the condition expression with no quote;
the positive three-way execution confirms its once-per-test effect. That observation is bounded to
this bytecode shape and does not broaden the purity rule.

The full case also keeps the side-effect condition distinct from the control failure. Its raw jarde
text renders `touch()` in the latch and the two compiler errors point only to the control methods.
Thus the full-class failure must not be attributed to the side-effect condition or to unrelated
missing conditional-value recovery.

## Evidence files

- `DoWhileAudit.java` and `DoWhileRunner.java`: complete mixed audit and independent runner.
- `positive/`: independently runnable ordinary/effectful positive class and three-way evidence.
- `core/`: minimal seven-case control probe and three-way evidence.
- `run_audit.py`: deterministic replay script.
- `summary.json`: exact statuses, CLI hash, class hashes, output hashes, quote counts, and skipped
  runtime states after failed compilation.
