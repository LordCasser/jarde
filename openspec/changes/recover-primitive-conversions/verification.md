# Verification

## Implementation

The decoder now maps each opcode from `i2l` through `i2s` to one
`PrimitiveConversion { source, target }` fact. The builder reads the actual single stack
operand at that producer, checks the presented source type against the opcode category, and
emits the existing `ExprKind::Cast` at the conversion BCI. Int accepts byte/char/short/int
presentations; boolean and mismatched Long/Float/Double categories are refused. The narrow
conversions retain byte, char, or short as the cast target while the JVM frame remains Int.

The conversion node participates in producer rendering, terminal-reader discovery, expression
closure, deferred-value saves, quote tracing, and `renders_the_value_it_reads`. Recursive
rendering keeps the original consumer position, so nested conversions stay ordered and their
operands use the shared evaluation/save machinery. No conversion folding or new type/frame model
was added.

## Automated checks

- `cargo test -p jarde-java --lib`: 115 unit tests passed, including the complete 15-opcode
  mapping and source-category/boolean rejection checks.
- `cargo test -p jarde-java`: passed; 115 unit, 32 Java-recovery integration, and 50 pattern
  integration tests passed.
- `cargo test -p jarde --test p3_primitive_conversions`: 2 passed, 1 JDK runtime test ignored by
  default. The ignored test was then run explicitly and passed.
- `cargo build -p jarde-cli`: passed.

The conversion integration tests recover every permanent fixture method as structured Java,
compare essential and all-evidence output byte-for-byte, and require a complete nonempty source
map whose segments carry member and BCI origins. They also verify that the fixture has no quoted
`@bytecode` body.

The focused budget/replay test uses `longFloatRound` (`l2f` at BCI 1, then `f2l` at BCI 2). Its
essential and all-evidence bodies match; repeating the all-evidence method request reproduces the
same complete source map, whose direct cast anchors point to those two BCIs in that method. Output
and IR limits are derived from successful measured usage and reduced by one, avoiding a guessed
fixed budget. Neither boundary publishes structured partial Java, and a precancelled request
states cancellation without publishing a body or source map.

The explicit runtime test compiled the complete recovered class with `javac --release 8` and ran
both it and the exact frozen original class bytes using `java -Xverify:all`. All 41 output lines
matched, including all 8 effect-order cases.

## Full-class audits

The 180-case audit compiled and ran all three complete conversion group classes. Each `javac`
completed successfully; all 180 output lines matched the original, with zero quoted methods and
zero differences. This covers integer results, raw floating-point bits, and conversion-sensitive
overload selection.

The 73-case chain audit also compiled and ran the complete class successfully. All 73 output lines
matched the original, including raw float/double bits, intermediate rounding, evaluation order,
and exception class/trace. No first mismatch was found. JADX retained seven known chain differences;
Jarde matched the original on all cases. Audit scripts were run from temporary copies with output
paths redirected, so the checked-in evidence inputs were not edited.

## Remaining verification boundary

The conversion traversal reuses the builder's existing IR-item charging, cancellation polling,
recursion-depth checks, and source-origin propagation. The focused test covers output and IR limits,
cancellation, and source-map replay; it does not exhaust every budget dimension or depth separately.
Floating-point constant recovery is not implemented in this codebase, so the conditional
NaN/constant-fold guard in task 2.4 has no constant path to exercise. The root agent owns the final
task-2.4 checkbox decision.

Root review leaves 2.4 open: the focused test proves output/IR exhaustion, precancellation,
essential/all equality, and deterministic cast source-map replay, but does not independently
exhaust analysis-work, recursion-depth, or source-record dimensions. The conditional floating
constant guard remains inapplicable until that separate feature is implemented. This is a
verification gap, not an observed conversion mismatch.

## Root acceptance, 2026-09-23

Root independently rebuilt and froze CLI SHA-256
`8b86c7293387e4be385630884109cb0effa4d044762788ac019abb7cbdd756e9`. With that exact
binary, the complete three-class audit reproduced all 180 original output lines and the separate
conversion-chain audit reproduced all 73, each with zero quoted methods and zero differences;
the result hashes are in `/tmp/jarde-root-primitive-replay-20260923/summary.json`. The permanent
fixture's ignored JDK execution test passed all 41 lines under `java -Xverify:all`. A source-only
grouping control avoiding the unsupported `fconst_2` also reproduced 18/18 lines with no quote.

Root ran the conversion and eight adjacent regular integration targets: all nonignored tests
passed after updating the stale unary-negation assertion to accept the now recovered `i2b`.
The conversion and unary-negation ignored JDK runtime tests passed. Compound-lvalue tests passed
4/4. The floating-constants target still has three intentionally red recovery tests because that
separate feature has not been implemented; the corresponding `fconst_2` boundary remains quoted.
There is no implemented shift-expression target to claim as a pass.

The Java package unit/integration suite passed in the implementation pass. Root additionally ran
all 160 reader unit tests and the corpus fingerprint target (5 passed, 1 intentional regeneration
test ignored). `cargo fmt --all -- --check` passed after formatting two new lines. Strict
`jarde-java` Clippy reports only the pre-existing `region.rs:1736` type-complexity debt; rerunning
with that one lint waived passes `--all-targets -D warnings`. OpenSpec strict validation passes.
Implicit narrowing at `ireturn`, narrow local declarations, floating constants, and shift
expressions remain separate work rather than being folded into this change.

The permanent 31-method fixture and the 180/73 audited cases have no quoted/reference bodies.
The separate task-1.2 boundary class still has four references after the root replay reduced them
from 24. Its remaining `grouped(I)I` reference is at BCI 4, `fconst_2`, which belongs to the
independent floating-point-constant change. The legal `i2b; pop` discard case is deliberately
quoted and still compiles and runs (`discarded:ok`); boolean `(Z)Z` remains explicitly refused.
These boundary results are not a claim that the whole edge class is semantically equivalent.

Known adjacent limits are implicit narrowing at `ireturn` without an explicit conversion,
refinement of byte/char/short local declarations, and floating-point constant recovery; these are
outside this change's scope.
