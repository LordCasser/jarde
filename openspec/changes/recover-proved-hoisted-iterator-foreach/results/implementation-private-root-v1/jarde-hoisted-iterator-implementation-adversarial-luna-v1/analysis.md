# Hoisted iterator foreach candidate: adversarial static review

This review is read-only and static. I inspected the current four OpenSpec artifacts, the current production `build.rs`, `reuse.rs`, `ssa.rs`, the accepted same-IR v3 diagnostic, and the final private candidate pinned below. No shared file was changed. No Git, Cargo, Rust parser/compiler/formatter, Java, JADX, CLI, or network command was run. The candidate has not been compiled or executed; this review does not establish that the recovered foreach passes the remaining Builder proof.

## Finding

The final candidate is structurally consistent with the narrow task 2.1 contract on the points reviewed. I found and reported one earlier-draft issue: an ancestor declaration could be removed when it happened to be in the active `self.stmts` vector. The collector corrected this before the pinned version: a strict-prefix owner returns no declaration-removal index, while still validating the active-vector match; strict-prefix `Local` placement is refused. The final code therefore preserves the outer declaration even when nested regions share one vector.

## Owner identity, visibility, and deletion

`iterable_for_each_candidate` now receives the loop `path` from its sole production caller at `build.rs` around lines 19228 and 21464. The owner helper at candidate lines 11854–11978 resolves the iterator store, test load, and next load through `reuse::Plan::variable_at` and requires one identical `LocalVariable`. It also checks the source name, the Builder's current `declared` set, a non-`Incomplete` placement, and `path.starts_with(owner)`. This is materially stronger than matching a slot or spelling alone.

For `Elevated`, it requires exactly one matching variable in `Declarations::at_region[owner]` and exactly one anchor at the initializer Store BCI. The existing `declare_at` path (base `build.rs` around line 22663) inserts the variable in `declared` and pushes that owner declaration through the normal billed `push`; the loop is reached only after the enclosing traversal has emitted that declaration. The candidate also searches only the active vector before `init_index`, and requires at most one matching blank `Iterator` declaration whose slot resolves to the same `LocalVariable`.

The critical strict-ancestor case is now handled correctly: when owner is a strict prefix of loop path, only `Elevated` is accepted, the helper returns `(None, true)` even if a declaration row is found in the same active vector, and the caller consequently removes only `init_index`. For a same-path declaration, the helper can return its current-vector index; the caller sorts those current-vector indexes descending and removes them only after the candidate returned successfully. No owner-path index is synthesized or applied against a different vector.

The caller first builds the loop body with the existing `walked?` propagation, restores the outer statements/declarations, and only then runs the candidate. The returned removal list is consumed after the candidate returns `Some`; an owner/Phi `StopReason` is carried out of the candidate's `Option` closure and returned before this removal and before the final loop `push`. The method-level atomic-stop/publication behavior still needs the existing root budget-stop tests; this static observation only establishes the candidate does not perform its removal on a captured stop.

## Phi occurrences, liveness, and unrelated Phis

The new `iterator_phi_use_ownership` helper at candidate lines 11984 onward:

- Visits every direct use of the exact `ValueId`; a BCI use must be one of the test/next loads, and each `None` use is counted in its canonical block.
- For each such block scans Phi rows, but counts only `PhiInput::Value` occurrences exactly equal to the checked `ValueId`. It requires the aggregate exact-input multiplicity to equal the block's `None`-use multiplicity. Repeated operands count repeatedly; `Itself` contributes no occurrence.
- For each Phi with at least one exact operand, requires the output to have a replacement, no remaining uses, and a bounded replacement chain to the same iterator. It checks every `Value` arm of that matching Phi resolves to the same iterator, so a mixed or unrelated arm rejects. Phis without the exact checked value do not gate this proof, even when they are from another slot or have their own nontrivial replacement/liveness state.
- Bounds replacement following by the existing finite Phi count and charges each hop. A replacement cycle or chain that fails to reach the iterator returns false.

This matches the accepted observed facts: the header V12 has `[V37, Itself]`, zero uses, replacement V37; downstream V22 has three exact V37 inputs, zero uses, replacement V12; the direct V37 `None` multiplicities are one in block 35 and three in block 97. The block-97 Local(6) Phi V23 has no exact V37 operand and is intentionally irrelevant to ownership. These diagnostic observations explain the intended positive shape; they do not prove that next/cast/effect gates pass.

The helper is invoked for both `iterator_value` and the existing header Phi value. The second invocation may repeat bounded proof work where there are uses; that work is also charged. This is conservative and remains within the local existing gate rather than adding SSA mutation or a global liveness service.

## Gap semantics and unchanged later proof

For a strict ancestor owner, the candidate requires every statement after the initializer in the active pre-loop list to be a blank declaration. Thus even literal assignments and other executable statements are refused on this new path, preserving the possibility that an exception from `iterator()` observes prior local state. Same-path behavior retains the old gap predicate, including its existing allowance for literal assignments; the design records that as a separate timing/effect debt rather than silently broadening this task.

The diff leaves the existing exact interface owner/name/descriptor checks, init Store and test/next Loads, iterator header Phi shape, no-other-writes test, next result unique consumer, both inline and separated cast/store branches, handler equality, origin construction, and outer-loop refusal untouched. No new assumption about the later cast/effect proof is made. The accepted diagnostic's immediate old refusal was the iterator value/Phi-use gate; it did not run or establish the downstream next/cast/effect gates.

## Budget and adversarial test obligations

The owner helper polls and charges `AnalysisSteps` for each access identity check, owner/path check, active statement row, and owner declaration row. The Phi helper polls and charges each direct use, each Phi row visited for a None-use block, each input inspected in that block, each matched Phi output check, and each replacement hop. These operations return `Result<_, StopReason>`; the `Option` closure stores the first StopReason and the caller returns it after the closure rather than turning it into an ordinary refusal. Existing earlier bulk charge remains; the added work is explicitly charged rather than assumed covered by it. Actual budget headroom and exact stop usage/location still require runtime verification.

Task 2.2 should exercise: identical and split `LocalVariable` identity; absent, `Incomplete`, non-visible, wrong-anchor, and duplicate owner rows; strict ancestor declaration preserved when it shares the active vector; executable/literal ancestor gap refusal; exact Phi multiplicity shortfall/excess and duplicate operands; multiple matching Phis; `Itself`; mixed/non-resolving arms; absent replacement, replacement cycle/overlong chain, and live output; unrelated nonmatching Phi ignored; an extra ordinary use refused; and real budget/cancellation stops at owner scan, use scan, Phi row/input scan, and replacement hops. Verify default/all/noRuleDetails preserve the same decision and that stop does not publish a partial method. The candidate itself adds no tests, so these remain explicit acceptance work rather than completed coverage.

## Pins and limits

`pins.json` records the exact base source, final private candidate/patch, all four planning artifacts, related source APIs, and the accepted diagnostic/raw/class inputs. The candidate hash in this review is the final collector version `2e72552b…4850bb9`; earlier draft hashes are not the reviewed artifact. Static inspection only: no compile, formatter, or execution result is asserted.
