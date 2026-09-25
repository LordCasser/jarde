# Java 8 conditional value audit

This is a source-only audit of conditional (`?:`) values. It does not modify production code,
Cargo artifacts, generated recovery source, or an OpenSpec change. `run_audit.py` is the replay
entry point; it uses `/tmp/jarde-ternary-values-audit` for temporary classes and saves every source,
status log, generated source, runtime output, diff, and summary in this directory.

The fixture was compiled with `javac --release 8 -g:none`. `TernaryRunner.java` is compiled
independently against each candidate `TernaryValues` class and observes both values and effects.
The cases cover a conditional returned directly, assigned to a local, nested in arithmetic, passed
as a call argument, a reference conditional with `null`, a conditional used to select a legal
`String` overload, and both conditional branches with distinct side effects and exceptions.

## Three-way result

The original class and full JADX output both compile, verify, and run successfully. Their 16-line
outputs are byte-for-byte equal; the output SHA-256 is
`2228b678cae9dbf92439c619ef502c42c5579620dcbe713deb465695b8e9aa1a`.

The frozen accepted CLI `/tmp/jarde-cli-deferred-accepted-7747` returns class-source exit `0`,
with 20 `@bytecode` markers in the full class. Its generated source is copied byte-for-byte to a
temporary `.java` path before compilation; it is never edited. `javac --release 8 -g:none` then
returns exit `1`, with seven `missing return statement` diagnostics in the conditional-value
methods (`returned`, `assigned`, `arithmetic`, `callArgument`, `reference`, `overloadChoice`, and
`throwing`). The exact locale-specific compiler text is in `frozen-javac.log`; because compilation
failed, no frozen runtime was attempted. `frozen-diff.txt` records that skipped comparison.

The class input is 1414 bytes with SHA-256
`a2e96a4bb220fc3a61f4635827c628b84ad4571d8e71fd9da42a4c0e94c9a80f`. The frozen CLI SHA-256 is
`7747b60a17dc635f0e8402d867cb9e74f9472a62057eb4865dbd8f4a207dfb34`. Original bytecode and the
exact compiler structure are in `original-javap.txt`; the raw frozen report is in
`frozen-report.txt`.

The smallest independently meaningful failure is preserved in `core/`. `TernaryCore.returned`
contains only `return condition ? a() : b();`, where `a()` and `b()` leave distinct trace values.
The 424-byte class has SHA-256
`9c867c925fb0f42b7fe11af18bb4e90645fe6ac8fcd2718114100161378b6ce4`. Original and JADX each
compile/run with exit `0`, and their two output lines match. Frozen class-source returns `0`, but
the unchanged generated `TernaryCore.java` fails `javac` with exit `1` and one missing-return
diagnostic; no frozen runtime is attempted. This reproduces the gap without arithmetic, overload,
reference typing, or exception helpers.

## Architectural boundary

`Region::If` supplies the control-flow arms and join, while SSA records the values produced by each
instruction and the entry phi at the join. The current renderer explicitly refuses a stack
`Definition::Entry`/`Definition::Phi` because no instruction produced that merged value
([`build.rs:4130`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:4130)).
The minimal core reaches exactly that boundary: `ifeq`, one helper call per arm, `goto`, then
`ireturn` consuming the join stack phi. The frozen text therefore preserves both arm regions and
quotes the join value, but has no Java return statement.

This is a conditional-value/stack-phi recovery gap. The helper methods, `if` effects, `iadd`/`imul`,
reference `aconst_null`, overload `Methodref`, and exception behavior are independently ordinary
Java 8 shapes in the fixture and do not explain the failure. The full-class 20 markers are the
aggregate source evidence from all affected conditional methods; they are not 20 distinct missing
opcodes. `original-javap.txt` shows the failing core uses only `iload`, `ifeq`, `invokestatic`,
`goto`, and `ireturn`; the extended cases add the already modeled `astore`/`iload`, `imul`/`iadd`,
`aconst_null`, `ldc`, `areturn`, and overload `invokestatic`. The existing design explicitly leaves a post-region `TernaryMod` out of the earlier
structure change ([`present-proved-java-structure/design.md:48`](/Users/lordcasser/workspace/projects/jarde/openspec/changes/present-proved-java-structure/design.md:48)),
and the deferred-value map requires Entry/Phi to remain a refusal boundary
([`implementation-map.md:28`](/Users/lordcasser/workspace/projects/jarde/openspec/changes/preserve-deferred-value-order/implementation-map.md:28)).

The relevant nearby contracts remain intact: `Builder` already tracks deferred values and committed
bindings (`build.rs:1173-1190`), region test purity only admits value-producing instructions
(`region.rs:2091-2132`), and `SsaPhi` retains one input per logical predecessor
(`ssa.rs:260-270`). Closing this boundary would therefore require a bounded conditional-value
rewrite that proves the join's arm ownership and renders the merged value once; this audit does not
implement that change.

## Reproduction files

- `TernaryValues.java`, `TernaryRunner.java`: complete fixture and independent runner.
- `core/TernaryCore.java`, `core/TernaryCoreRunner.java`: minimal core isolation.
- `run_audit.py`: deterministic replay script.
- `original-javap.txt`, `jadx.java.txt`, `frozen.java.txt`: raw source/bytecode evidence.
- `original-run.txt`, `jadx-run.txt`, `frozen-run.txt`, and `core/` counterparts: exact runtime
  outputs (the frozen files are empty because compilation failed).
- `summary.json`: exact exit statuses, class hashes, CLI hash, and quote count.
