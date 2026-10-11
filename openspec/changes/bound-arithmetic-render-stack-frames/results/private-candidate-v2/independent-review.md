# Independent read-only review: arithmetic render frame extraction

Reviewed against the current workspace files and frozen `bound-arithmetic-render-stack-frames` design. No repository writes, Git commands, or toolchain/test runs were performed.

## Source identity and scope

The current `crates/jarde-java/src/build.rs` SHA is `2586f9eaa09c06056bc8c565ed6bd00c18c35ad6ddb28970ef3d6e0ee2fa5bef`, matching the candidate's pinned base. The current `tests/p3_immediate_functional_receivers.rs` SHA is `ce60b1a18ac11d109c5f5866c9b8e5eb34eae93f2f3be584c8b155e01fcd4e87`, also matching its pinned base.

The production diff is the described split: `render_value` keeps its shared entry checks and gains an early Arithmetic path; its former `Definition` dispatch is moved to `render_value_definition`, with the old Arithmetic match arm removed. `render_value_definition` has only one call site, immediately after the entry checks in `render_value`, so the helper cannot be called around the depth gate. No public type, proof, cache, or budget code was added.

## Semantic invariants checked

- The existing priority checks remain in their original order: depth bound; conditional value; binding; postfix snapshot; binding refusal; array-initializer alias; `this` alias. Arithmetic is considered only afterward.
- The candidate checks `sites.site_of(bci)` before taking the fast path. An owned construction instruction therefore falls through to the extracted dispatcher, where the existing `new_expr(site, at, depth)` branch remains first. Construction-site priority is preserved.
- The arithmetic body retains the same missing-instruction error, exact operand-count error, two operands, left-then-right evaluation order, shared consumer `at`, and `depth + 1` for each operand. The expression still uses `arithmetic_op(*op)` and `Expr::direct(..., bci)`, so its physical origin remains the arithmetic producer BCI. Returning `Ok` early is equivalent to the old match-arm result.
- The `MAX_VALUE_DEPTH` check and refusal text remain at `render_value` entry. Every arithmetic child recurses through `render_value`; the extracted helper does not recurse into itself. Other definition variants retain their prior refusal/render branches.
- The test candidate only changes `an_overdeep_lambda_helper_renames_instead_of_walking_an_unbounded_body`: javac and class-file reading remain outside the worker; `open` and `class_source` move to a requested 2 MiB standard-library thread; the two existing helper-rename assertions stay. The inaccurate comment claiming the test recompiles/runs the whole class is removed. The test still does not assert recovered-class compilation, and the private review does not treat it as doing so.

## Exact risks / items for root

1. The arithmetic fast-path condition currently calls `self.sites.site_of(*bci)` **before** it checks `self.operations.get(*bci)` for Arithmetic. Consequently every instruction value that passes the earlier return checks gets an extra `site_of` scan; if it is not Arithmetic, `render_value_definition` performs the original scan again. `site_of` is a pure iterator search (`crates/jarde-java/src/init.rs:393`), so this does not change output, priority, or budget counters, but it adds uncharged work and can scan all construction sites twice for ordinary non-arithmetic instruction values. Consider ordering the read-only `Operation::Arithmetic` filter before the site test so non-arithmetic values do not incur this added scan. Preserve the site check before executing the arithmetic body. This is a cost concern, not an observed semantic failure.
2. The candidate and patch contain a formatting splice at the end of the moved dispatcher: `).into()),                }` (candidate `build.rs` line 26775; patch hunk near its end). Rust whitespace still separates the tokens, but this is not formatted and may explain or contribute to patch-generation trouble. Root already plans to regenerate the diff; ensure that line is restored as the original `).into()),` followed by the match-arm closing brace on its own line. I did not run `git apply --check`, per instruction.
3. The requested 2 MiB stack reduction and unchanged output/usage are runtime properties. Source review cannot establish the new frame size, prevent optimizer/debug-build frame merging, or prove the class-source call succeeds on that stack. Validate with the root-owned test/build and frozen-output comparison; do not add `inline(never)` absent frame evidence, consistent with design.

## Conclusion

At the source level, the proposal preserves dispatch semantics, construction-site precedence, depth checks, error strings, evaluation ordering, and producer origins, and matches the frozen extraction design. The only identified semantic-adjacent issue is redundant uncharged construction-site scanning in the precheck order; the patch also needs a formatting correction and runtime verification. These are review findings, not a claim that the candidate compiled or passed.
