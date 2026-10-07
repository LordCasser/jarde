# Task 1.1 — where the allocation point is presented, and the gating experiment

Date: 2026-10-07 (UTC). Binary: this worktree, `cargo build --locked -p jarde-cli`.

## The located emission points

```
$ grep -n "fn project_class_source_double_brace\|fn project_class_source_anonymous_super" src/facade.rs
4666:    fn project_class_source_double_brace(          # new: the four-criteria admission
4890:    fn prove_double_brace_allocation(             # new: one allocation's proof
5222:    fn project_class_source_anonymous_super(      # path A's companion-body projection (untouched)
$ grep -n "anonymous_override" crates/jarde-java/src/emit.rs | head -3
234:    emitter.anonymous_override = Some(AnonymousOverride {
1219:                if let Some(override_) = emitter.anonymous_override
$ grep -n "fn projected_statements_method_text" src/class_source.rs
7861:    pub(crate) fn projected_statements_method_text(
```

* The **allocation point itself** is the emitter's existing `anonymous_override`: it matches the
  `new X$N(args…)` expression whose origin names the allocation BCI **wherever its statement
  sits** (`ExprKind::New` at 1219), hides one argument by BCI and writes the anonymous body. Path A
  never reached the field-assignment position (`DB.dbl = new DB$1();`) because its *site scan*
  (`crates/jarde-java/src/report.rs::class_source_anonymous_site`) recognizes only a direct return
  and a local-declaration initializer; the emitter needed no change for this slice.
* The **companion-body emission point** (path A) is `project_class_source_anonymous_super`, reached
  from `project_class_source_anonymous_interface` under `_anonymous_return_sites.len() == 1`
  (`src/facade.rs:2118`). This slice adds its own pass **before** that dispatch and claims the text
  by setting `anonymous_interface_projection = Projected`, which is what keeps path A, the two
  folds after it and every later projection from re-presenting a text that already carries the
  source form. Path A's code and criteria are untouched.
* The inline family's **allocation identity** is `AnonymousAllocationScan` (one entry per method,
  `verified` sites with the constructor BCI and the ordered argument BCIs) and its **single-use
  channel** is `prove_anonymous_owner_xrefs` (exactly one `new`, exactly one constructor call, two
  typed self rows, no other use of the child anywhere in the selected input). Both are reused
  unchanged; the `val$` discipline (`prove_anonymous_super_val_capture`, the role partition) is
  reused with one contained widening (below).

## The gating experiment: the four criteria alone

With the admission in place and nothing else changed, the patrol's own `db.jar` flips at both
anchors, and the three negatives keep the text they had. Renders (`--format text`):

```
$ ./target/debug/jarde-cli class-source --input openspec/evidence/.../double-brace-patrol/fixture/db.jar --class DB --format text
    static java.util.List withCapture(java.lang.String arg0) {
        return new java.util.ArrayList() {
            {
                this.add((java.lang.Object) arg0);
            }
        };
    }
    static {
        DB.dbl = new java.util.ArrayList() {
            {
                this.add((java.lang.Object) "a");
                this.add((java.lang.Object) "b");
            }
        };
        return;      # <- see the measured correction (f) below: the initializer's own terminator
    }
[exit 0]            # the baseline exit was 4 (partial: the path-A refusal)
```

The report's own state moves with it: `anonymous_interface_projection.state` reads `refused`
(the path-A source-type refusal) before and `projected` after, and `execution.status` reads
`partial` before and `complete` after — the presentation the operation now delivers is complete,
and the later folds and the companion-body dispatch are skipped exactly as a claimed text requires.
Both renders are frozen under `gating/` (`patrol-DB-before.txt` is the patrol's own recorded host
render, `patrol-DB-after.txt` this worktree's).

The negatives (fixtures frozen by task 1.2, `tests/fixtures/proved-java-structure/double-brace-allocation-site/`):

```
$ ./target/debug/jarde-cli class-source --input <negatives/methods jar>      --class DBM  --format text | grep "new D"
        return new DBM$1(s);          # the companion declares a method: path-A text, unchanged
$ ./target/debug/jarde-cli class-source --input <negatives/unspellable jar>  --class DBN  --format text | grep "new D"
        return new DBN$1(s);          # the superclass `Carrier$Nested` is not a source name: unchanged
$ ./target/debug/jarde-cli class-source --input <negatives/multi-site jar>   --class DBS2 --format text | grep "new D"
        DBS2$1 local1 = new DBS2$1(arg0);   # constructed twice: both sites unchanged
```

The two-leg render scans of the whole corpus (`02-corpus-scans.txt`) state the same result over
every frozen class: the only renders that move are the two anchors (the patrol's `DB` and the
fixture legs' `DB`), plus the new control `DBS`.

## Measured corrections (filing assumptions the implementation had to replace)

Each of these was found by measurement, not by reading; each is contained to this slice's own
paths, and the last one is a latent defect this slice had to fix:

* **(a) the root's `<clinit>` AST is not retained.** The class-source assembly retains a method's
  AST only when `retain_all_method_asts` or the two-shape site scan matches
  (`crates/jarde-java/src/report.rs:6173`), so the static initializer's field-assignment site has
  no AST at all. The retention gains a **third reason**: one verified allocation of the declaring
  class's own anonymous child (`allocates_an_anonymous_child`, the same cheap `X$N` pre-filter the
  admission reads). The charge is the same `weight = 2` the site-scan retention already pays; the
  corpus scans classify the `ir_items +2` bookkeeping delta it produces.
* **(b) a platform superclass cannot be read.** `resolve_class_source_dependency_read_raw` resolves
  and reads a class **from the provided input**, and `java.util.ArrayList` is not a member of the
  input — which is also why path A's `spellable_source_type` requires the same package. The
  admission therefore does **not** read the superclass: the companion's own bytes state the rest
  (JVMS 4.1 requires a `super_class` to be a class; loading requires it accessible and non-final;
  the constructor's own `invokespecial` names the superclass constructor the form re-states, from
  the companion's own package — the root's).
* **(c) the census's child-self-invocation arm is gated on ring 2.** `DB$1`'s own body calls
  `this.add(…)` with the *child* as the pool owner (`invokevirtual DB$1.add`), which the census
  admits only for `AnonymousOwnerCensusPath::DirectSuperclassDirectReturn`. A third discriminant,
  `DirectSuperclassDoubleBrace`, states the same arm for this path: the companion declares no
  method of its own at all, so every such symbolic owner is an inherited member, and the recompiled
  anonymous class inherits it from the same superclass. The other paths keep the exact refusal.
* **(d) a constructor's capture read has the post-call `this` as its receiver.** The read
  `this.val$s` after `super()` reads the value the constructor call wrote into slot 0 (JVMS 4.9.2's
  conversion, which this layer records as that call's write), not the entry `this`
  `value_from_this_load` proves. `CaptureReadReceiver` states the two readings: every existing
  certificate passes `EntryThis` (its bodies are the companion's *methods*), and this slice passes
  `ConstructorThis`.
* **(e) a block statement's capture read sits under a conversion.** The build writes
  `(java.lang.Object) this.val$s` at the `add` argument, so the field expression is one node inside
  a `Cast` that carries the read's own anchor. `CaptureReadAnchor` states the two readings the same
  way: `Field` for every existing projection, `Converted` for this slice's own entry point
  (`project_class_source_converted_parameter_reads`).
* **(f) the anonymous re-emission did not suppress an initializer's void `return`.** `javac` reads
  a `return` inside an initializer block as an error (JLS §8.7), and `emit_class_source_statements`
  already sets the emitter's `initializer` flag for `<clinit>`; the anonymous re-emission
  (`emit_class_source_anonymous_return`) did not, so the static-initializer anchor wrote an
  uncompilable `return;`. The same flag is now set from the same fact in both entry points — a
  no-op for path A, whose sites are never `<clinit>` (a static initializer's own trailing
  `return;` has no value, so the direct-return scan cannot match it).

## The four criteria, as implemented

| criterion | read from |
| --- | --- |
| anonymous subclass | the root's `InnerClasses` row for the child: exactly one, no outer class, no inner name; the child's own row repeats it; its `EnclosingMethod` names the root class and the allocation's own method |
| pure instance block | the child's method table declares its constructor **alone** (no `<clinit>`, no other method) and its field table declares nothing the compiler did not mint — the proved `val$` capture or nothing; the constructor's own statements are the call, the certified capture store, the block and the closing `return`; the call forwards the constructor's leading parameters in order; the block is non-empty |
| single use | `prove_anonymous_owner_xrefs`: exactly one `new`, one constructor call, two typed self rows, no other use |
| spellable superclass | `spellable_source_name` (no `$`, every `/` segment a Java identifier) and not `java/lang/Object` (an interface child is the interface path's own domain) |

Boundaries stated with the criteria: one qualifying allocation per method (the emitter rewrites one
allocation node per pass, the one-site discipline every companion projection keeps); an
**interface** child (`java/lang/Object` superclass or a non-empty interface table) is not this
shape; a child whose constructor body the block reading does not own (an empty block, a `return`
inside it, a read of one of the constructor's own parameters, a pool-form allocation, a local that
would shadow the re-spelled capture name) keeps the presentation it had.
