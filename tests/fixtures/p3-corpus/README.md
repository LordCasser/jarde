# P3 3.3 corpus: the flags, the missing dependency, and what the replayable comparison answered

This directory is the P3 3.3 corpus: the compiled samples the **replayable** Java 8 compile-and-execute
comparison reads, the exact commands that produced them, and the table that comparison printed. The
comparison itself is `tests/p3_execution_comparison.rs`; it is `#[ignore]`d because it needs a JDK
(see "Running it" below), and every sample here is a **committed class file**: the compiler is a
generation-only input, exactly like the other P3 fixtures.

## Provenance

All samples except the ECJ one were compiled by **javac 23.0.1** (Oracle,
`/usr/bin/javac`, `javac 23.0.1`) on macOS. Every command prints its usual
`源值 8 已过时` / `source value 8 is obsolete` warnings (3 of them, 4 for `-source/-target`) and exits
**0**; the classes are committed as bytes.

```text
cd tests/fixtures/p3-corpus
javac --release 8 -g:none          -d v8-gnone          Flags.java
javac --release 8 -g               -d v8-g               Flags.java
javac --release 8 -g:lines,source  -d v8-glines         Flags.java
javac --release 8 -parameters -g:none -d v8-parameters  Flags.java
javac -source 8 -target 8 -g:none  -d v8-source-target  Flags.java
javac --release 8 -g:none -cp absent-Library-stub -d v8-missing-dep MissingDependency.java
rm -rf v8-missing-dep/absent          # the stub is provenance, never shipped: see below
```

| directory | sample | class-file version | bytes | SHA-256 |
| --- | --- | --- | --- | --- |
| `v8-gnone/` | `Flags.class` | 52.0 | 325 | `212fde26bf85b958b85cbe2b3add9fdde5a68c9ecb81e3f310dd2650c18934b6` |
| `v8-g/` | `Flags.class` | 52.0 | 597 | `1f088d15c5aac281f3c30fab187553f10b56af6b598ae5b919e7696c7a4cccd3` |
| `v8-glines/` | `Flags.class` | 52.0 | 441 | `b8d28d2a821497b64523919d2e8ab2ae793b5da70d2cc51682b9285aa96f13ff` |
| `v8-parameters/` | `Flags.class` | 52.0 | 395 | `d841ad231c91d41e861546e5f22f350b22b7e25dd30c3f92acbbbf2582fc1d69` |
| `v8-source-target/` | `Flags.class` | 52.0 | 325 | `212fde26bf85b958b85cbe2b3add9fdde5a68c9ecb81e3f310dd2650c18934b6` |
| `v8-missing-dep/` | `MissingDependency.class` | 52.0 | 264 | `3da493a28864cb1af56f9b5ae76fdfac27f1d5efe601558f2c772fabeaf6c6fb` |

**`--release 8` and `-source 8 -target 8` produce the same bytes here**: `v8-gnone/Flags.class` and
`v8-source-target/Flags.class` are byte-identical (one digest above, and the comparison asserts it).
The same cross-check on the already-committed `p3-local-rewrite` sample gave the same answer: compiling
`tests/fixtures/p3-local-rewrite/LocalRewrite.java` with `-source 8 -target 8 -g:none` produced
`f755f062bc9779d941e93bf1ef4889b3ce6efb0dbd670e5ccbd07152126158a6`, byte-identical to the committed
`--release 8` build — which is why no second copy of it is committed.

**The missing dependency is real**: `MissingDependency.java` was compiled against
`absent-Library-stub/absent/Library.java`, and only `MissingDependency.class` is shipped. `javac` also
wrote `v8-missing-dep/absent/Library.class` on that command (it compiles what it resolves on the source
path); that copy was deleted, as the command above says, so `absent.Library` is unresolvable for every
reader of this fixture — which is precisely the input "a class whose dependency is missing".

**ECJ**: `tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class` (52.0, 303 bytes,
SHA-256 `f9b6566fc4533e3be181ef7bf1455d478fb016f0d9a5e1f685a34901ddb0b6ad`) is the **second compiler**
of this comparison. Its provenance (ECJ 4.6.1, the jar's URL and digest, the Temurin 8 `rt.jar`
bootclasspath, and the `-source 1.3 … -target 1.8` command) is recorded in
`tests/fixtures/historical/README.md`, and it was compiled there, not here. **ECJ is not runnable on
this machine**: the jar is not in the repository and there is no network access, so this corpus reads
the ECJ *artifact* and does not re-run the ECJ *compiler*. The v45–v51 ECJ samples are not part of
this comparison: it is about the Java 8 (52.0) release gate.

## The flag matrix

| corpus entry | `-g:none` | LVT (source names) | `LineNumberTable` | `MethodParameters` | presented as Java | executed | traces |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `v8-gnone/Flags.class` | yes | no | no | no | 3/3 members | 3/3 | identical |
| `v8-g/Flags.class` | no | yes | yes | no | 3/3 members | 3/3 | identical |
| `v8-glines/Flags.class` | no | **no** | yes | no | 3/3 members | 3/3 | identical |
| `v8-parameters/Flags.class` | yes | no | no | **yes** | 3/3 members | 3/3 | identical |
| `v8-source-target/Flags.class` | yes | no | no | no | *same bytes as `v8-gnone`* | — | — |

The shapes are the same under every legal flag set, and the difference the flags make is visible in
the **wrapper**, which is where this comparison reads the naming facts: with `-g` the wrapper's
parameters are named `seed`, `flag`, `local` (the class file's own `LocalVariableTable`), and with
`-g:none`/`-g:lines,source`/`-parameters` they are named `arg0`, `arg1` (the ordinal the reader
invents when the class states no name). `-parameters` adds a `MethodParameters` attribute and nothing
here reads it (P3 3.1's recorded boundary), so its row is the `-g:none` row with a bigger class file.
P3 2.4's `-g` / `-g:none` pair on `p3-scope` is read the same way by the other test.

## Running it

```text
cd <repository root>
cargo test --test p3_execution_comparison --locked -- --ignored --nocapture
```

`cargo test` (without `--ignored`) never runs it, so a machine with no JDK stays green: the fixtures
are bytes and no other test invokes a compiler. The two ignored tests are also run by CI's `stable`
job, which already installs a JDK and already runs the ignored
`jdk25_instruction_boundaries_match_public_bytecode_inspection` oracle. **No fixture is recompiled at
test time** — `javac` is only ever asked to compile the wrappers *this comparison generates*, so the
JDK that runs the comparison cannot change a sample's bytes.

Set `P3_COMPARISON_TRACES=1` to print the two traces of every sample as well; the traces are the
observable evidence, and the run fails when they differ by even one line. The declaration each member
was wrapped in is printed for every member whether or not that flag is set — that is the artifact the
"the wrapper comes from the run's own facts" requirement is about, and for these samples it reads:

```text
p3-scope/v8        scope(Z)I                     is wrapped as `public static int scope(boolean arg0)`
p3-scope/v8-debug  scope(Z)I                     is wrapped as `public static int scope(boolean b)`
p3-scope/v8-debug  receiver(J)J                  is wrapped as `public long receiver(long a)`
p3-corpus/v8-g     choose(ZI)I                   is wrapped as `public static int choose(boolean flag, int seed)`
p3-local-rewrite   cast()Ljava/lang/String;      is wrapped as `public static java.lang.String cast()`
p3-handlers        open(Ljava/lang/String;)LRes; is wrapped as `static Res open(java.lang.String arg0)`
historical/ecj/v52 add(II)I                      is wrapped as `public int add(int arg1, int arg2)`
```

`static` is there because the class's flags say so, `boolean` because the descriptor says `Z`, `b` /
`seed` because the class file's own table says so, `arg0` / `arg1` because it says nothing, and
`java.lang.String` / `java.lang.Object` because the run's own parameter-type fact erases a reference
to `Object` (which would be a *different* signature) and the descriptor's class is the fact that
names it.

## What the comparison answered (the recorded run of 2026-09-20, re-run after P3-R7 and finding (i) were closed)

`cargo test --test p3_execution_comparison --locked -- --ignored --nocapture`
on javac 23.0.1 = **2 passed; 0 failed; 0 ignored**. Columns: the run's own classification
(`Java/Structured` = it wrote the whole body as Java; `Mixed/Fallback` = it kept quoted bytecode), what
`javac --release 8` did with the wrapper of that member's own declaration, and what the comparison then
did with it (the two traces compared line for line). The two rows that changed with the closure of the
two findings are marked **closed** below; every other row is the earlier run's own answer, unchanged.

### The required members (P3-R1/R2/R3 and R5)

| sample (compiler, flags) | member | run | wrapper | comparison |
| --- | --- | --- | --- | --- |
| `p3-local-rewrite/v8` (javac, `--release 8 -g:none`) | `make()Ljava/lang/Object;` | Java/Structured | compiles | executed: traces identical, count `0->1` |
| | `post(I)I` (R1) | Mixed/Fallback | javac: missing return statement | not executed; count control compiles, count `1->1` |
| | `bump(I)I` (R1 control) | Java/Structured | compiles | executed: traces identical |
| | `doubleIt(I)I` (R1 control) | Java/Structured | compiles | executed: traces identical |
| | `saved(I)I` (R1) | Mixed/Fallback | javac: cannot find symbol | boundary; count control also refuses (`unexpected return value`) |
| | `conditional(I)I` (R1) | Mixed/Fallback | javac: missing return statement | not executed; count control compiles, count `1->1` |
| | `loopAcross(II)I` (R1 control) | Java/Structured | compiles | executed: traces identical |
| | `cast()Ljava/lang/String;` (R2) | Mixed/Fallback | javac: missing return statement | not executed; count control compiles, `calls 1->2` on **both** sides |
| `p3-scope/v8` (javac, `--release 8 -g:none`) | `scope(Z)I` (R3+R5) | Java/Structured | compiles (`if (arg0)` over a `boolean`) | executed: traces identical |
| | `simple`, `armOnly`, `reuse`, `after`, `reassign`, `receiver` | Java/Structured | compiles | executed: traces identical |
| `p3-scope/v8-debug` (javac, `--release 8 -g`) | the same seven | Java/Structured | compiles with the class file's own names (`b`, `seed`, `a`) | executed: traces identical |
| `p3-handlers/v8` (javac, `--release 8 -g:none`) | `body`, `tail`, `one`, `two`, `three`, `suppressed`, `secondInitFails`, `sync`, `syncThrows`, `main` | Java/Structured | compiles | executed: traces identical (78 lines, incl. the reverse close order of `two`/`three` and `boom \| suppressed close-r`) |
| | `open`, `openFailing` | Java/Structured | compiles | executed: traces identical (**closed**: see finding (i)) |
| | `syncBody`, `fail`, `boom` | Mixed/Fallback | compiles (quoted body) | boundary: the run wrote no body to execute |
| | `withCatch` (`jre_guard_unexplained_row`), `branching` (`jre_guard_body`), `fin`, `catchFinally` (`jre_guard_finally_copy`), `syncThrowsCatching`, `secondInitFailsCatching` (`jre_guard_resource_init`), `suppressedCatching` (`jre_region_irreducible`) | Mixed/Fallback | compiles (quoted) | boundary: every BCI of every refused region is quoted, and every quoted BCI is anchored |
| `historical/ecj-4.6.1/v52` (**ECJ 4.6.1**, `-source 1.3`, target 52.0) | `add(II)I` | Java/Structured | compiles | executed: traces identical |
| | `finallyPath(I)I` | Mixed/Fallback | javac: missing return statement | boundary: quoted whole (**closed**: see finding (ii)) — the graph of that body accounts for no instruction of its handler, which the text now quotes |

### The corpus and the missing dependency

| sample | member | run | wrapper | comparison |
| --- | --- | --- | --- | --- |
| `p3-corpus/v8-gnone` | `counted(I)I` | Java/Structured | compiles | executed: traces identical, counts `0->1`, `1->2`, `2->3` |
| | `copied(I)I`, `choose(ZI)I` | Java/Structured | compiles | executed: traces identical |
| `p3-corpus/v8-g` | the same three | Java/Structured | compiles with the class file's own names | executed: traces identical |
| `p3-corpus/v8-glines` | the same three | Java/Structured | compiles with ordinal names | executed: traces identical |
| `p3-corpus/v8-parameters` | the same three | Java/Structured | compiles with ordinal names | executed: traces identical |
| `p3-corpus/v8-missing-dep` | `viaAbsentLibrary(I)I` | Java/Structured | **javac refuses**: cannot find symbol (`absent` is not shipped) | boundary |
| | `plain(I)I` | Java/Structured | compiles | executed: traces identical |

## The content classification of this sample (recorded run of 2026-09-20, `25ed621` + this change)

The comparison's per-member table prints a `content` column beside the run's representation/quality,
from `RecoveryReport::content`: `contains_statements` when the artifact the run committed holds at
least one statement the emitter wrote as Java, `explanation_only` when the artifact is the envelope,
its reasons and the quoted bytecode alone, and `not_produced` when the request was answered with a
stop and no artifact. Each sample's table is followed by its own reconciliation, and those numbers
are these:

| sample (compiler, flags) | members declared | with `Code` | requests | produced | contains_statements | explanation_only | stopped | not requested (initializer) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `p3-corpus/v8-gnone` | 4 | 4 | 3 | 3 | 3 | 0 | 0 | 1 |
| `p3-corpus/v8-g` | 4 | 4 | 3 | 3 | 3 | 0 | 0 | 1 |
| `p3-corpus/v8-glines` | 4 | 4 | 3 | 3 | 3 | 0 | 0 | 1 |
| `p3-corpus/v8-parameters` | 4 | 4 | 3 | 3 | 3 | 0 | 0 | 1 |
| `p3-corpus/v8-missing-dep` | 3 | 3 | 2 | 2 | 2 | 0 | 0 | 1 |
| `p3-local-rewrite/v8` | 9 | 9 | 8 | 8 | 8 | 0 | 0 | 1 |
| `p3-scope/v8` | 8 | 8 | 7 | 7 | 7 | 0 | 0 | 1 |
| `p3-scope/v8-debug` | 8 | 8 | 7 | 7 | 7 | 0 | 0 | 1 |
| `p3-handlers/v8` | 24 | 24 | 22 | 22 | 14 | 8 | 0 | 2 |
| `historical/ecj-4.6.1/v52` | 3 | 3 | 2 | 2 | 1 | 1 | 0 | 1 |
| `p3-nested-eval/v8` | 5 | 5 | 4 | 4 | 4 | 0 | 0 | 1 |
| `p3-refused-cast/v8` | 7 | 7 | 6 | 6 | 3 | 3 | 0 | 1 |
| **total** | **83** | **83** | **70** | **70** | **58** | **12** | **0** | **13** |

The counts reconcile three ways, and the comparison asserts each: 83 members declared = 83 with
`Code` + 0 without; 70 requests = 70 produced + 0 stopped; 70 produced = 58 `contains_statements` +
12 `explanation_only`. Every one of the 13 members not requested is a constructor or class
initializer: a wrapper would have to re-declare the sample's own class name and this layer's AST does
not model `super()`/`this()`, so those members are listed as *not requested* and are in no denominator
above. They are the same shape the benchmark's reference decompiler folding `<init>`/`<clinit>`
represents: an unmatched initializer is an applicability fact, not a failure of either engine, and it
is excluded here rather than counted against either side.

Twelve of the 70 artifacts hold no statement: `p3-handlers`' refused guarded bodies, `finallyPath`
(the ECJ `finally` body finding (ii) closes), and the three refused casts of `p3-refused-cast`. Their
text is not empty — it is the reasons and the quoted bytecode — which is exactly the case a
text-shaped classification gets wrong. The other 58 include the refusals that keep a statement
(`post`, `conditional`, `cast`, `nestedLocal`), which is why `explanation_only` and
`contains_statements` are the only two produced values and neither is derived from the other.

### The historical token heuristic is a heuristic, not this classification

The distribution the architecture review quotes — 25,853 requests over the benchmark corpus, 99.4%
"has text", ≈83.4% "has statements", 16.0% explanation-only — comes from `compare2.py`'s
`statements` heuristic: it strips comment lines from the artifact's text and asks whether any token
is left. It is an audit trail with its own definition, measured on the benchmark corpus (not on these
fixtures), and it is **not** the engine's classification: `RecoveryReport::content` is read from the
committed structure, per request, with no text parsing at all. The two are not the same instrument
and must not be read as one: the old numbers are kept as they were, no historical percentage is
rewritten from these counts, and nothing here is a statement-coverage rate. `contains_statements`
means "the artifact holds a statement", not "the body was recovered"; `explanation_only` is the
share of *produced* artifacts that hold none, and a produced artifact of either value can still be
`Mixed`/`Fallback` with unproven semantics.

## The boundaries: what cannot be a compilation unit, and why

Every row below is a place where the recovered text is **not** a method body Java accepts. The reason
is stated as the cause, and the evidence is `javac --release 8`'s own refusal of the text under the
declaration this comparison derived from the run's facts.

1. **The run quoted a read it could not prove, so the body has no value to return**: `post`, `saved`,
   `conditional`, `cast`. `javac` reports `missing return statement` for `post`, `conditional` and
   `cast` (the quoted read stood where the `return`'s value would be), and `cannot find symbol` for
   `saved` — the local the `return local1;` names is the value of a write the run refused to derive,
   so the name is not declared anywhere either. This is R1's and R2's refusal, and it is the answer
   rather than a gap: the artifact states the bytecode it refused (both BCIs) and the reason.
2. **A quoted body that still compiles is not executed**: `Guarded.syncBody`, `fail`, `boom`,
   `withCatch`, `branching`, `fin`, `catchFinally`, `syncThrowsCatching`, `secondInitFailsCatching`,
   `suppressedCatching`. Their text is the refused region's quote (comments), which `javac` accepts as
   a `void` body; executing it would compare an empty body against a body that opens a resource or
   throws, which is why the comparison asserts the quote instead (every BCI of every refused region is
   quoted, and every quoted BCI is an anchor of the source map).
3. **The declaration of the body itself cannot be compiled under the member's own signature**: no
   member of this corpus is a boundary for this reason any more. It was finding (i) — `Guarded.open`
   and `Guarded.openFailing`, whose `new@1` argument was written as the integer literal `0`/`1` where
   `Res.<init>`'s own descriptor declares `boolean` — and the shared argument path of the build now
   types every call's arguments by the callee's descriptor, so both members compile **and execute**
   (their rows are above). The literal's type is decided where every invocation is written, not in
   `new@1` alone: the same question is asked by `invokevirtual`, `invokestatic`, `invokespecial`, the
   constructor call of a construction site and the `super(…)`/`this(…)` of an instance initializer.
4. **The type the body names is not shipped — finding (ii)-adjacent**: `MissingDependency.viaAbsentLibrary`.
   The layer reads one class file and states the call; `javac` needs the class (`cannot find symbol`),
   which no fixture in this repository has. The *original* member cannot be run either
   (`NoClassDefFoundError`), so this row is a boundary on both sides — which is exactly the question
   "missing dependency" asks: what a reader that has no classpath can and cannot say.
5. **A constructor or class initializer is not wrapped at all**: `<init>` and `<clinit>` of every
   sample. A wrapper would have to re-declare the sample's own class name, and `super()`/`this` are not
   modelled by this layer's AST; nothing in this comparison asks for them, and the skip is printed
   rather than silent.
6. **R2's count control is a declaration this comparison states, not the member's own**: for a member
   whose value the run refused (`post`, `conditional`, `cast`), the same text is compiled once more
   under a `void` declaration, because that is the weakest declaration that can hold a body the run
   gave no value to. It is used for the **call count only** (the value dimension is not compared there,
   and the row says so), which is how `make()`'s count is measured: the original `cast()` moves
   `LocalRewrite.calls` by 1, and the recovered text moves it by 1 as well. `saved`'s control does not
   compile either (`unexpected return value`) and the row states that too.

## Findings this corpus produced (**both closed** by the change that follows them)

**(i) A `boolean` argument is written as an integer literal.** `Guarded.open`/`openFailing`: the text
`new Res(arg0, 0)`/`new Res(arg0, 1)` was not compilable because `Res.<init>`'s descriptor says
`(Ljava/lang/String;Z)V`. The report stated `representation=Java`, `quality=Structured` for these
bodies (it never claims `syntax_status=Checked`), so the boundary was visible only when the text was
actually compiled — which is what this comparison does. **Closed** by typing a call's arguments by the
callee's own descriptor in the shared argument path (`crate::build::typed_arguments`): an argument that
is a *literal* `0`/`1` is written `false`/`true` where the parameter is `Z`, and every other parameter
type keeps the argument it had (`byte`/`char`/`short` legally take an `int` constant, JLS 5.3). The two
members are now `compiles` + `executed: traces identical`, and the comparison's own tables state that
rather than a boundary.

**(ii) An ECJ `finally` body's exceptional copy is not accounted for.** `HistoricalControlFlow.finallyPath`
*was* presented as Java (`{ int local3 = arg1 + 1; arg1 = arg1 + 2; return local3; }`) and its value
comparison passed, while the class file states an exception table row `[0, 4) → 9` and BCIs `9..14`
(`astore_2; iinc 1, 2; aload_2; athrow` — the copy of the `finally` the compiler emitted for the
exceptional path, which also increments `value`):

```text
public int finallyPath(int);
   0: iload_1   1: iconst_1   2: iadd   3: istore_3
   4: iinc 1, 2   7: iload_3   8: ireturn
   9: astore_2  10: iinc 1, 2  13: aload_2  14: athrow
   Exception table: from 0 to 4 target 9 any
```

The run's own record of that member was one structured region at BCI 0 with `blocks=[0]`, and the
comparison printed `note: exception handler entry BCI(s) [9] are named by no anchor`: the handler's
entry — and the code behind it — had no text and no anchor in the artifact, so a reader could not tell
"judged dead" from "never seen". javac's own `finally` copy *is* refused (`fin`/`catchFinally` →
`jre_guard_finally_copy`, listed above), so the two compilers' `finally` shapes were answered
differently. **Closed** by the recovery layer's own coverage check
(`crate::region::unaccounted_instructions`): before any region of the graph is presented as the body,
every instruction the same read decoded has to be covered by a canonical block or named as
unreachable. The graph of this body accounts for none of BCI 9/10/13/14 — nothing in `[0, 4)` can throw
synchronously, so the normalization never made the handler a node and never listed it as dead — so the
body is now refused whole under `jre_region_unaccounted_instruction`, quoting exactly those BCIs; its
row above reads `Mixed/Fallback` and the comparison's ledger assertion is what keeps it that way. When
the walk *already* refuses part of a body, the same check adds a quote naming the unaccounted
instructions instead of replacing the walk's own reason (`p3_guard`'s patched `one` shows both codes at
once).

**The canonical graph's own side of (ii)** — reported, not changed: the handler block is not merely
unreachable, it is **absent**. `MayThrow` (`jarde-jvm/src/cfg.rs`) is a per-opcode table, and exception
edges are created only from instructions that table admits, inside blocks the walk already created; the
declared `Exception table` is not itself a walk root. In the ECJ **v45** twin the same handler survives
as a node because the `jsr` normalization creates one per call context (six nodes, three dead); at v52
there is no call context, so `canonical.blocks()` is a single node `[0, 9)`, `unreachable()` is empty
and `handler_rows()`/`throw_sites()` are both empty. Making the normalization root at every declared
handler would publish a new dead node for every such body, which is a P2 decision (frames, SSA, billing
and the archived golden counts all move with it), and the coverage check above is what makes the P3
artifact honest without it.

**The TWR members' handler entries: an unanchored entry label is not an unaccounted instruction.** For
`one`/`two`/`three`/`suppressed`/`secondInitFails` the old note also printed handler entries (BCIs 32,
38, 69, 44, 77, 108). Their shape is *presented* as `try (Res local0 = open("r")) { … }`, whose
statements (both closes, the `addSuppressed`, the `athrow`, the primary's store) are anchored — the
handler *entry label* is the JVM's jump target, which the `try` statement states in its region's own
block list rather than quoting. That distinction is what the comparison's assertions are written
against: an instruction is accounted for when a canonical block covers it or the graph names it dead,
so a covered-but-unanchored entry passes, while an entry no block covers has to be quoted (P3-R7's
invariant, asserted over **every** decoded instruction and not only over handler entries).

## What this corpus does not cover* **A second javac generation**: only javac 23.0.1 is installed here. The *target* version (52.0) and
  the flag set are covered (`--release 8`, `-source 8 -target 8`, `-g`, `-g:none`, `-g:lines,source`,
  `-parameters`), but "an older javac's own codegen" is not available on this machine and no sample
  claims to be one.
* **A running ECJ**: the ECJ *artifact* is covered (see above); the ECJ *compiler* is not runnable here
  (no jar, no network), so no new ECJ sample was produced for this slice.
* **A real obfuscator**: no ProGuard/R8 is installed and there is no network access, so no obfuscated
  sample is committed. The half of obfuscation that matters to this layer is name loss, and that is
  covered by `-g:none` (and by the LVT-less `-g:lines,source` row); a sample whose names are not Java
  identifiers at all would have to be hand-built, and this slice does not fabricate one.
* **A non-Java-language compiler** (Kotlin/Scala/Groovy): none is installed (`kotlinc`, `scalac`,
  `groovyc` are all absent) and there is no network access. ECJ is the second compiler this corpus
  could have, and does.
* **Execution of the boundary members**: by construction (see "The boundaries" above).
* **A *new* ECJ sample**: the ECJ fixture is read as bytes only (no jar, no network), so the second
  compiler's shapes are covered by the committed class files and not by a re-run of the compiler.
* **`p3-handlers`' `open`/`openFailing`** are no longer a gap at all: they compile and execute, which
  is the closure of finding (i) recorded above.
