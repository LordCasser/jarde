# Implementation map: preserving static type qualifier bindings

This map is based on the current implementation in `crates/jarde-java/src/{names,report,field,build,decode,facts,stop}.rs` and on the three baseline audits in `openspec/evidence/java-syntax-2026-09-22/type-name-shadowing/`. It is a code-fact map for the later implementation; it does not change the production design or add a recovery stage.

## Evidence boundary

The defect is lexical binding in the emitted Java text. The AST already distinguishes `ExprKind::Path` from a local expression, but both are written as Java names and Java reclassifies the first segment according to the names in scope. The default-package `arg0` baseline (`359` bytes, SHA-256 `6df3f4ccd7ce4fcbaca22c4a38f38cb0f637ce01d5d7b0df0c4ececf98846363`) compiles while reading or writing `ShadowOther.value` instead of `arg0.value`. The external-owner baseline (`340` bytes, SHA-256 `ab42b65ce6bdec23cf980037232d946203c2459f07858fb1804cfb6935992445`) has the same wrong binding for `arg0.pick`, reads, and writes. The package-prefix baseline (`468` bytes, SHA-256 `d775ee2998d9e6d18cb0a2225ce186c341fb4fa6e04d784af10a53439bc22a0c`) keeps debug parameter `java`, so `java.lang.Math.abs` is rejected by `javac`. All three were audited with CLI SHA-256 `7527b03abc1e487121d204672b52043ca5f3aa1148c6d65b136e36f2bbe1f4aa`.

The collection boundary is one decoded method body. It must not inspect every class constant-pool entry, class declaration, unused method, method reference, construction site, or real field declaration merely because such a name exists. Existing unknown or unspellable owners keep their current refusal/fallback behavior; this change must not invent a type spelling.

## Where the owner facts come from

`decode::Operations::of` resolves each decoded instruction once into a BCI-keyed `BTreeMap` (`decode.rs:78-110`). `facts::Operation::Invoke(CallTarget)` carries the invocation kind and the owner in the pool's internal form (`facts.rs:328-391`, `493-495`). `facts::Operation::Field` carries the access, static bit, owner, member name, and descriptor (`facts.rs:508-523`). A production collector should walk the existing SSA body instructions and call `operations.get(instruction.bci())`; it should not add a constant-pool walk or depend on the test-only `Operations::iter`.

For an ordinary static invocation, the current builder reads `CallTarget::owner()` in `build.rs:3963-4023`. With no discarded static-call qualifier, it emits a type path only when `declaring_class` is present and differs from the target owner (`build.rs:4007-4022`). A same-class static call is deliberately bare. When bytecode evaluated an explicit qualifier and then popped it, the builder renders that expression instead (`build.rs:3972-4006`); the target owner is not written in that path. The qualifier change must keep this evaluation rule and must not re-analyse pop effects. The bounded owner constraint can therefore conservatively collect a spellable foreign owner from a static `CallTarget`, while skipping a target equal to the declaring class and a run with no declaring class. That retains the required same-class/no-qualifier behavior without introducing a new call proof. A foreign owner may be conservatively reserved even when an existing discarded-qualifier path eventually supplies the expression; this is a naming constraint only and does not authorize a new call recovery.

Fields have a stronger existing proof. `field::plan` scans the body and records each claimed access as `Plan::claimed: BTreeMap<u32, (Evidence, Shape)>` (`field.rs:67-80`, `242-297`). `Evidence` is the already-decoded owner/member fact and `Shape::receiver` is `None` for a claimed static access (`field.rs:214-240`, `386-415`). `build::render_value` uses `fields.claim(bci)` and writes `spell_reference(&evidence.owner)` for a static read (`build.rs:3772-3813`). `field_write` does the same for static writes except a proven simple same-class blank `static final` write, where `simple_static_final_write` intentionally removes the receiver (`build.rs:4736-4769`). Instance-field owners are not type qualifiers in the emitted expression and must not enter this set.

The field plan is the place to reuse the proof. The smallest integration is a read-only plan helper which walks the already claimed map and returns the claimed static owner facts (and, if practical, the existing simple-final field-name set) to `report`; it must not decode or reparse the pool. If that helper is not added, a report-side walk over existing SSA BCIs plus `fields.claim(bci)` still reuses the proof and avoids field parsing, but it pays for a second bounded scan. Combining the owner and simple-final-name extraction in one plan traversal avoids that duplicate scan and keeps the existing final-field reservation intact. `field::Plan` should return internal owner strings and the BCI needed for charging; `report`/`build` remains the place that calls the canonical type spelling helper.

Only claimed static reads and writes whose builder will write an owner path need a prefix. A static read always writes `Owner.field`; a static write writes it unless `simple_static_final_write` applies. A refused field access has no structured owner expression and is not collected. The simple-final field name itself remains reserved exactly as today. Do not collect `Allocate`, `CheckCast`, dynamic-site, method-reference, or class-declaration names: they are separate spelling/effect decisions and are outside this change.

## Canonical spelling and prefix extraction

`build::spell_reference` is the only internal-name-to-Java spelling function (`build.rs:7088-7132`). It converts `java/lang/Math` to `java.lang.Math`, removes object-descriptor wrapping, and reuses the array parser for descriptors; malformed forms return `None`. The qualifier collector must call this helper and insert only the first Java path segment: `arg0` becomes `arg0`, `java.lang.Math` becomes `java`, and `$` in a binary/nested class segment remains part of that segment. It must split the already-spelled path at `.` rather than attempting to parse internal names itself. Do not reserve the full dotted path or only `Math`. An unspellable owner produces no invented prefix and leaves the builder's existing refusal/fallback path unchanged.

The resulting type-prefix set is a local `BTreeSet<String>`. It is merged with the current `fields.simple_static_final_names(budget)` result before the `NameTable` is built. Keeping it as a local set until collection succeeds prevents a stopped run from publishing a partially reserved naming decision. Owner constraints have no local-variable evidence record of their own; the original BCI, owner, member, and pool identity remain in the existing operation/field evidence and AST origins.

## NameTable contracts that must remain unchanged

`NameTable` stores rendered names, the reserved set, and alias/invention counts (`names.rs:332-345`). `report::recover` prepares the table after recovery, reuse, and `field::plan` (`report.rs:470-519`). `build_with_reserved` and `build_with_receiver_and_reserved` clone the supplied `BTreeSet` before the deterministic slot/variable walk (`names.rs:379-465`). This is the right existing input for type prefixes and simple-final field names; no AST rewrite or post-emission string replacement is needed.

The candidate rules are precise:

* An unnamed parameter or local gets `argN`/`localN` (`names.rs:534-540`). A reserved `arg0` therefore makes the generated candidate collide before it can be emitted.
* A legal debug spelling is retained until a reserved or previously taken name forces the numeric collision suffix. The suffix loop starts at `_2` and records `AliasReason::Collision`; an illegal raw spelling still follows the existing `AliasReason::Unspellable` alias path (`names.rs:426-461`). The raw debug name remains in `RenderedName::raw`, and `report` later materializes the same name evidence (`report.rs:1010-1023`).
* All split variables of a reused slot are candidates in the existing slot/variable order. Reserving both `arg0` and `arg0_2` before the walk is essential: a generated/debug `arg0` must skip the already reserved `arg0_2`, rather than taking a suffix that is another type prefix.
* An instance slot 0 is always `this` from the member's receiver fact (`names.rs:365-376`), with its raw debug evidence preserved. The qualifier set must not turn receiver identity into a pseudo-local or rename a real class/field.
* `free_name` searches the union of the reserved set and rendered local names, appending `_` until the candidate is legal and free (`names.rs:482-503`). The builder's lambda parameter helper calls it and then separately avoids earlier lambda parameters (`build.rs:5614-5626`). Passing the type prefixes into the table consequently makes an existing synthetic `free_name` candidate obey the same lexical constraints without a second owner analysis. The helper has no raw local evidence, so it must not be presented as a collision alias.

Same-class static invocation owners are not reserved when the builder emits the bare call. A same-class static field read, and a same-class static write that is not the proven simple-final case, still emits `Type.field` and therefore does reserve the owner's first segment. A method owner is never deleted to avoid a collision, and a static access is never changed to an instance expression or a fabricated null receiver.

## Budget and cancellation: current gap and narrow closure

The current facilities do not yet satisfy the complete budget wording in the change spec:

* `field::plan` and its simple-final proof are already bounded. The proof polls and charges class-field headers, claimed accesses, and attributes (`field.rs:300-384`); `simple_static_final_names` polls once and charges each claimed entry (`field.rs:142-160`).
* `stop::charge` checks cancellation and charges before work, while `stop::poll` observes cancellation/elapsed interruption (`stop.rs:111-150`). The report converts either result into an honest stopped report before building text.
* `NameTable::decide` has no `Budget`, and its slot walk and collision-suffix loop are uncharged (`names.rs:401-466`). `NameTable::free_name` also has no `Budget`; its underscore candidate loop is uncharged (`names.rs:482-503`). The extra `lambda_params` suffix loop in `build::param_name` is uncharged as well. Determinism exists, but a low `IrItems` limit or cancellation cannot currently stop inside these naming loops.
* The report currently calls the non-budgeted name constructors immediately after the field-name reservation (`report.rs:498-519`). Source-map replay uses the already-decided AST and does not rename again, but that alone does not make name selection budgeted.

The minimal bounded closure is local to this change. First, collect static owner candidates into a temporary set, polling and charging one `IrItems` item before each examined static invocation/claimed static field (including duplicate references; deduplicate only after the paid observation). If the charge or poll fails, return the existing stopped report without passing a partial set to `NameTable`. Second, provide narrow budget-aware internal naming entry points used by `report`—for example, `try_build_with_reserved`/receiver variant and a `try_free_name` path—whose candidate suffix/underscore attempts poll and charge before each actual attempt. Keep the existing deterministic order, raw evidence, `Collision` classification, and public/unit-test convenience constructors; do not precharge a guessed bound, because that would not report the actual stopping candidate. Thread the existing `StopReason` through the already-`Result`-returning report/build path. If synthetic lambda names remain outside this change, state that limitation explicitly; if this spec is to cover them, the builder's narrow `param_name` call must use the same budgeted helper rather than the current uncharged `free_name`.

This is a targeted extension of the existing naming input and stop channel, not a general naming refactor. There is no honest implementation of the limited-budget scenario using the current `NameTable` signatures alone. In particular, merely charging the owner set while leaving the suffix and `free_name` loops uncharged would satisfy collection accounting but fail the spec's “constraint collection or name collision processing” stop requirement.

## Report integration order and invariants

The existing order in `report::recover` is the usable insertion point:

1. `Operations::of` is created from the method code, then canonical flow and `region::recover` run (`report.rs:442-469`).
2. `reuse::plan` decides whole/split slot variables (`report.rs:470-482`).
3. `field::plan` proves field claims and simple-final writes (`report.rs:483-497`).
4. In the same name-preparation block, obtain the existing simple-final field names and the bounded static-owner prefix set, union them locally, and call the budget-aware receiver/non-receiver `NameTable` constructor (`report.rs:498-519`).
5. Pass that one immutable table to `build`; do not recollect owners during AST construction, rerun `lambda::plan`, add an AST type tag, or rename after `emit`.

The collector must use the method's decoded operations and field claims, so an unused constant-pool owner cannot alter an unrelated method. It must remain deterministic by observing BCIs in a stable order and inserting prefixes into a `BTreeSet`. Essential and full evidence selections then share the same pre-build naming decision; evidence materialization after the artifact (`report.rs:939-1023`) can expose existing field/call origins and aliases without choosing another name. A stop before the table is built publishes no partial qualifier reservation; a source-map replay reads the same AST and table and never allocates a different name.

## Explicit boundaries

This map does not cover real fields whose names shadow package prefixes, method-reference qualifiers, class renaming, imports, type hierarchy resolution, or all unused class facts. Those remain separate evidence/changes. It also does not claim that current `free_name`, lambda suffixing, or `NameTable::decide` already honor a budget; the narrow budget-aware naming entry points above are required if the spec's limited-run scenarios are to be implemented rather than merely tested at generous limits.

## root 审读决定

上述源码事实与现有设计一致。预算入口的具体函数名只是实现建议，不能据此建立第二套命名算法，也不要求为后向兼容保留重复入口；候选、Collision/raw 证据及停止处理应共享一个决定过程。新增约束会影响普通槽名及共享非槽名，二者的冲突尝试均属于本项预算范围，不能把 lambda/延期值名称另留为无预算例外。前置 deferred-order 实现完成后按实际统一命名入口接入，避免同时重写同一 Builder。

`simple_static_final_names` 的遍历可在同一字段计划查询中合并所需静态 owner，或复用有界 claim 查询；不用为了消除一次小遍历而创建长期名称计划对象。字段 claim、canonical spell_reference、NameTable reserved 仍是唯一事实链。未使用 CP、真实字段遮蔽和方法引用 owner 保持设计声明的边界。
