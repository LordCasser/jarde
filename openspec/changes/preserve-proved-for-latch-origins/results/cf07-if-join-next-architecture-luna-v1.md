# CF07 `counted(II)I` If-join origin audit

## Finding

The smallest safe fix can reuse the existing `OriginSet` and the current `Region::If` shape; it does not need a new public IR or a new `Region::If` field. Extend the existing If-origin construction in `Builder::region` to recognize a narrowly defined, nonempty arm whose final owned straight block ends in an unconditional transfer to that If's already-elected join. Add that transfer BCI as a derived origin of the `StmtKind::If`. The existing empty-arm case remains unchanged.

Do not reuse `Region::Loop.gateway_origins` for this edge. That field is carried by `Region::Loop` and is folded into the loop statement's origin. BCI 20 is the edge that closes the inner If arm, so its owner is the If span; assigning it to the enclosing while would give the wrong source-map owner. `OriginSet` is reusable at the Builder boundary, but the loop-only `gateway_origins` storage is not.

## Frozen facts

The frozen javap at `openspec/changes/preserve-proved-loop-latch-origins/results/cf07-candidate-root-v1/cases/javac23-original/javap.txt:60-87` shows `counted(II)I`. BCI 14 branches to 23; the then arm executes `iinc 2, 2` at 17 and `goto 27` at 20; the else arm's `istore_2` at 26 falls through to 27; BCI 27 increments the loop counter and BCI 30 returns to header 6. This bytecode establishes the physical then-arm edge and the join. It does not by itself establish what `Region::If.join` held in a particular runtime replay.

The frozen candidate observation at `candidate-observation.json:2145-2162` records `counted@20` under out-of-scope anchors and explicitly says it does not claim that anchor is covered. The Jarde rendering at `cases/javac8-jarde-all/LoopCases.java:34-40` presents a nonempty `if/else` inside `while`; the local JADX none-profile rendering at `jadx-output/none/sources/cf07/LoopCases.java:13-20` instead renders the conditional as a ternary. These are artifact observations; this audit did not run a CLI or independently replay the class.

## Existing seams

`Region::If` currently contains `prefix`, `branch`, `branch_bci`, both arms, and `join` (`crates/jarde-java/src/region.rs:438-445`). Ordinary If construction puts the recovered arms and join into that shape (`region.rs:3928-3936`). The recursive arm walk runs under a shared frame and tracks visited ownership (`region.rs:3522-3528`); the ordinary join check uses reachability and the arm continuations (`region.rs:3820-3920`). This establishes the structured If and its join, but it does not carry an origin for a transfer hidden at the end of a nonempty arm.

The Builder has the data needed for a bounded exact check: `canonical`, `ssa`, `code`, and `operations` (`crates/jarde-java/src/build.rs:8772-8776`), plus its mutable `budget` (`build.rs:8875-8879`). `CanonicalCfg::edges()` exposes every canonical edge (`crates/jarde-jvm/src/canonical.rs:356-363`); edge kinds distinguish normal, exception, call, and return edges (`canonical.rs:106-119`). The existing If builder creates a direct origin from `branch_bci`; it only adds transfer origins for an empty arm (`build.rs:17489-17509`). That is why the current presentation can be structurally correct while BCI 20 is absent.

`Stmt::If` is emitted as one source-map node. The emitter records the node's full byte span with its `OriginSet` (`crates/jarde-java/src/emit.rs:607-639`), and statement emission routes through that node (`emit.rs:654-659`). Thus `OriginSet::new(direct(branch_bci)).plus_derived(derived(20))` places BCI 20 on the complete If span, while the loop's separate origin continues to own BCI 30 (`build.rs:18060-18069`).

JADX's `IfRegionMaker.process` chooses/restructures an If, creates an `IfRegion`, and recursively creates then/else regions (`/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/regions/maker/IfRegionMaker.java:64-117`). Its `findOutBlock` tries a dominator-frontier intersection, then candidates from the union, then path-cross fallback (`IfRegionMaker.java:230-279`). This is a useful comparison for how another decompiler elects a join. It neither creates Jarde's `OriginSet` nor proves Jarde's physical source owner; the local ternary rendering is not a certificate for BCI 20.

## Exact acceptance boundary for the source addition

For a nonempty arm, accept only when all these facts hold:

1. The If has `join: Some(join)`, and the arm's final owned region is an existing `Straight` run (or the last direct child of its `Sequence` is that run). Its final canonical block is the candidate source block. Do not search backward past a later nested region or transfer.
2. The source block's final physical instruction is an unconditional `goto`/`goto_w` at the candidate BCI and decodes as `Operation::Transfer`. The decoded target and the canonical edge both identify this exact `join`.
3. The source block has exactly one outgoing canonical edge total, and it is `Normal` to `join`. Any exception, call, return, second normal edge, or unknown/missing edge refuses attribution.
4. The candidate block is held by this arm exactly once, not by the other arm or an outer owner. Do not infer ownership from instruction BCIs as though every instruction were a block leader.
5. Charge and poll each examined instruction/edge with the existing Builder budget path before using it; propagate cancellation or budget stop rather than publishing a partial origin.

The other arm may reach the same join by fallthrough. If it has a matching explicit terminal goto too, it may independently contribute its own derived BCI. Keep the existing empty-arm behavior, and avoid adding the same BCI twice. If any exact fact is unavailable, leave the edge unmapped and let the existing coverage/acceptance gate decide the result; do not infer it from text, the branch target alone, or `Operation::Transfer` alone.

A generalized scan of every block in every arm is unnecessary for this case. Restricting the new recognition to a terminal straight tail covers the frozen shape and leaves early returns, loop break/continue, nested-region exits, and arbitrary internal gotos under their existing ownership rules.

## Regression checks

- Positive: for both javac class versions and the accepted default/all source-map paths, `counted@20` has exactly the expected If-statement source span in physical method `counted(II)I`; `counted@14` remains the If condition/statement's existing provenance and `counted@30` remains the enclosing loop origin. Confirm the segment includes the complete emitted If, not just the then-arm assignment.
- Negative: refuse an edge whose terminal is conditional, return/throw, a non-goto transfer, or a goto to a target other than `join`; refuse any source block with multiple or exceptional outgoing edges; refuse a candidate that is not the arm's final owned straight block or is already claimed elsewhere.
- Budget: a zero/insufficient analysis budget and pre-cancellation during edge inspection must stop before publishing the derived origin. Existing empty-arm transfer behavior must still pass unchanged.
- Replay boundary: keep the full physical method/class and default/all equality checks. This audit does not authorize relaxing the all-BCI gate or expanding support to `lastIndexOf@25` or any other missing anchor.

## Unverified assumptions

I did not inspect a runtime dump of the recovered `Region::If` for this candidate, so `join == canonical block 27`, the exact nested `Sequence` shape of its then arm, and the edge inventory seen by Builder remain runtime facts to verify. The frozen bytecode and source output make them the expected shape, but they are not proof that the proposed terminal-tail recognizer will match. No source was changed and no Cargo, Git, JDK, CLI, or candidate replay was run for this audit.
