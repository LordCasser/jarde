# NumericComparison private input review

These are standalone source inputs for the existing, OpenSpec-approved `NumericComparison` condition producer path. `countLong`, `countFloat`, and `countDouble` each use the requested counted-loop shape and compare `values[i] < bound`; the induction local `i` is also consumed as the array index, so its condition-side read is exercised without adding a primitive conversion fixture or another dependency class.

The control set covers fixed finite `{1, 4}` arrays with bound `3` for all three primitive types. By source inspection, each normal loop performs one body iteration and then exits on the second element, so these normal cases are bounded. Float and double controls put NaN first and later: Java's `<` comparison with NaN is false, which predicts zero and one iterations respectively. Null controls predict `NullPointerException`; one-element arrays with bound `3` predict `ArrayIndexOutOfBoundsException` on the next condition check. Long MIN/MAX controls compare equal at the stopping element and predict zero/one iterations respectively. These are language-level predictions, not measured output.

The set deliberately does not add `Store`, `Increment`, or `Copy` producer coverage and does not alter any production source. Existing OpenSpec constraints keep role recognition, CFG, init/update/phi ownership, invariant and external-access checks, and Builder condition placement unchanged. The control baseline is pinned in `pins.json`; its accepted archive describes original-class dual-JDK runs and isolated generated-class comparisons, while this new oracle still awaits the parent's compile/run. No result here asserts a CLI outcome.

Normalized shape under test (all three methods differ only by primitive array/bound type):

```text
int count = 0;
for (int i = 0; values[i] < bound; i++) {
    count++;
}
return count;
```

Review status: source-only inspection; not compiled, executed, or compared with oldCLI/newCLI. The parent must first compile and run this exact input against the original class to record the raw oracle, then use the frozen-control collector for oldCLI/newCLI comparison. Pins record SHA-256 over the current source/spec/archive files and these inputs; they do not attest to runtime behavior.
