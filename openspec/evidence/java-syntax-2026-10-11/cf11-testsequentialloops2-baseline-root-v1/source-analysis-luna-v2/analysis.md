# SequentialLoops2 return ternary: source-path review

Review scope: read-only inspection of the shared Jarde source, the local JADX source tree, and frozen SequentialLoops2 captures. No shared files were changed. No Git, Cargo, Java/Javac, JADX/CLI, or Atlas index was run. Source/capture SHA-256 pins are in `pins.json`. Conclusions about what the present pipeline would do on the actual SSA values remain static unless explicitly identified as visible in the frozen report.

## Finding and smallest repair direction

The strongest source-level root-cause candidate is `init::outside_readers` in `crates/jarde-java/src/init.rs:2283-2314`, called by `verify_metered` at `init.rs:1111`. It follows direct uses of values written by the new-site BCIs but skips any SSA use whose `bci` is `None`. `crates/jarde-jvm/src/ssa.rs:196-203` defines exactly that form as a Phi operand use (“no instruction produces” it). This fixture's constructor result is one arm value carried into the join stack Phi at BCI 99, then the Phi is read by `areturn`; the allocation's value therefore reaches the return through a Phi rather than as a direct instruction use. Static control flow predicts the site scan sees no direct external reader and triggers the existing empty-reader refusal at `init.rs:1144-1158`. That is exactly the refusal text visible in the frozen Jarde report: `jre_new_shape`, allocation BCI 91, “nothing in this method reads the instance ...”. The capture confirms the external symptom; no same-run IR trace was available to confirm the exact `ValueId` chain.

The existing `build::prove_conditional_value` already implements the narrow authorization needed for a two-arm stack Phi: it checks the branch and complete canonical edges, exactly two normal predecessors in opposite straight arms, one varying stack Phi, a single Phi input supplied by each predecessor, each input's sole use being that Phi, and exactly one consumer of the Phi in the join (`build.rs:4905-4909`, `5004-5035`, `5235-5289`, `5291-5352`). `build_conditional_value` validates that each full arm is only its rendered value dependencies plus transfers, then renders both arm values at the actual consumer (`build.rs:17442-17465`, with dependency closure at `17882-17945`). For this method the branch test itself is already representable: `condition` maps `JumpIfNotNull` to `!= null` (`build.rs:33143` onward). This points away from a missing condition expression as the first visible blocker.

The minimum mechanism change should let a verified construction be a *single value leaf* of this already-proved conditional, with the Phi as the sole arm use and the Phi's return/call/store as the actual write position. Reuse the existing conditional-value proof or factor its relevant CFG/SSA checks into a shared certificate; do not count a raw Phi use as a generic instruction reader and do not allow arbitrary Phi traversal. There is a sequencing constraint: `report.rs:11140-11162` builds `Sites` before `region::recover`, while the builder's conditional proofs run after regions exist (`build.rs:16378-16475`). A site-verification repair must therefore provide the same bounded certificate before accepting the site, or deliberately move/factor only that preflight; it cannot simply call the current region-based proof from `init.rs` as-is.

Do not repair this by loosening `duplicate_expression`'s “no proved local assignment” path (`build.rs:25733` and surrounding code). That path is for a copy participating in a local-assignment dance. Here `dup` is a stack alias for an uninitialized object, not a source local assignment. Once a `Site` owns the allocation/copy/constructor, `render_value_definition` checks `sites.site_of(bci)` before interpreting the raw operation (`build.rs:26054-26067`), and `new_expr` writes the verified constructor at the value's use (`build.rs:27641` onward). The copy marker is thus a downstream result of the missing site proof, not sufficient evidence that general Copy rendering is the fix.

## Constructor alias and Phi support already present

The JVM frame layer has a precise token model for this exact constructor idiom. `crates/jarde-jvm/src/frame.rs:74-99` states that `new` creates an uninitialized token, pure stack `dup*` operations may move/copy it, and the applicable `invokespecial <init>` converts every alias of that token on the normal path. `frame.rs:1791-1858` implements the conversion, and `frame.rs:2263-2288` applies it at invokespecial. The unit test `a_constructor_call_initializes_every_alias_of_one_new_site` at `frame.rs:4725-4768` checks a duplicate left on the stack becomes an initialized reference. This is evidence that frame/constructor semantics need not be reinvented; it does not prove that the frozen method's actual SSA writes/uses and Phi are correct.

`init::verify_metered` already requires the allocation to be immediately followed by `Duplicate`, finds a same-block `<init>` call, validates that the receiver is the allocation/copy/constructor instance, checks argument producer ordering, then demands one external rendered reader (`init.rs:708-720`, `721-825`, `1097-1158`). Direct construction consumers are covered: `tests/p3_new_value.rs:191-220` tests `new; dup; invokespecial; areturn` and a constructor result used as a call argument. The missing case is a constructor value carried from one conditional arm through the join Phi before that consumer. The conditional proof tests in `build.rs:37600-37635` cover Phi-to-return/call/field-style value positions and source arm identity; the tests around `37682-37720` keep unknown common types and independent effects refused. The closest combined gap is therefore a frozen constructor arm plus a plain other arm and a single returned stack Phi, with assertions for both full mapping and exact recovered Java. The current controls are valuable regression boundaries and should stay unchanged.

## Frozen bytecode/source facts

The Java 8 and 23 Javap captures agree on the tail:

```
83 aload_1
84 ifnonnull 91
87 aload_0
88 goto 99
91 new           java/lang/String
94 dup
95 aload_1
96 invokespecial java/lang/String.<init>([C)V
99 areturn
```

The null branch pushes `s`; the non-null branch leaves the alias duplicated at BCI 94, which becomes initialized at BCI 96. The join needs one String Phi for BCI 99. The condition is `c == null`, so source spelling is `return c == null ? s : new String(c);`.

The captured Jarde report explicitly records `new@1` rejecting BCI 91 because the instance has no discovered reader. It records no `new` presentation, maps BCI 84 with derived BCIs 83/87/91, and leaves physical maps `[88,94,95,96]` missing. It marks BCI 94 as a copy with no proved local assignment and BCI 99 as stack-depth-0 entry state. Those last two messages describe the current fallback text, not proof that Copy or Phi is where the first internal rejection occurs.

JADX's frozen generated source has the complete expression (`.../jadx/default/sources/cf11/UpstreamSequentialLoops2.java`, line 35), and the baseline report says all four JADX compile/run legs match the original. In JADX source, `ConstructorVisitor.processInvoke` rewrites a constructor invoke to `ConstructorInsn`, removes the assignment chain back to `NEW_INSTANCE`, and sets the constructor result from the instance alias (`ConstructorVisitor.java:64-112`). `TernaryMod.makeTernaryInsn` requires the two arm results' single-use Phis to be the same, wraps each arm instruction as a ternary operand, and replaces the branch (`TernaryMod.java:72-145`); for direct branch returns it also has a separate two-return fold (`148-176`). `InsnGen.makeConstructor` emits ordinary `new Type(args)` (`InsnGen.java:727-782`), and `makeTernary` emits the condition and arms (`1174-1194`). Thus JADX normalizes constructor-plus-alias to a value-producing constructor node before Phi/ternary formation; it does not require the duplicated receiver to correspond to a Java local assignment. This describes current source code paths, not a trace of the capture's internals.

JADX has a direct integration fixture at `jadx-core/src/test/java/jadx/tests/integration/loops/TestSequentialLoops2.java:9-51`, using the same two loops and `return c == null ? s : new String(c);`; the test checks two while loops, a break, two `<= 127` tests, and return presence, and repeats with no debug info. The frozen raw decompile is stronger evidence for this instance's exact ternary spelling than the test's broad assertion. `TestTypeResolver4.java` also has a `new String(arr)` return but is not a ternary-Phi test.

## Same-IR evidence to collect before choosing a code change

Add a bounded debug-only observation at the current pipeline stages, not a new general tracing framework:

- From frame output: state before/after BCI 91, 94, 95, 96, and at 99, showing the `Uninitialized { new_site: 91 }` token, both aliases, constructor conversion, and initialized String on the non-null edge.
- From the published SSA/CFG: canonical block IDs and normal edges for branch 84, false path 87–88, non-null path 91–96, join 99; every `ValueId` read/written at 83/84/87/88/91/94/95/96/99; `Definition`, `uses` (including each `bci=None` Phi operand), Phi slot/value/inputs, and predecessor exit value.
- From `init::verify_metered`: selected receiver ID at 96; `produced_by` identities; constructor argument ID and its `produced_at` BCI; direct reader list after `outside_readers`; the exact use it skipped; and the failed reader/writer predicate. This confirms whether the candidate root is precisely the Phi boundary.
- From the region/builder path: recovered `Region::If` shape and branch sense; `ConditionalValueAttempt` refusal/proof enum; `when_true`/`when_false` IDs; Phi consumer BCI and operation; `conditional_dependencies` closure; `sites.site_of` for the allocation/copy/constructor value IDs; arm render results and final source-map ownership. This separates a missing site from a separate conditional arm/type/build refusal.

If the constructor alias does not emerge as the expected instruction value or the predecessor Phi lacks one edge's constructed value, stop at that evidence: the existing frame token proof and the actual SSA/report path would then disagree, and the repair would need to target that conversion boundary. If they do line up, the one-`None`-Phi-use gap in `outside_readers` is the narrowest demonstrated fix surface.
