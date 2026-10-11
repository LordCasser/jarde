# Expected controls (prediction only; not executed)

The original Java class is the behavioral oracle at runtime; this file is a coverage plan, not measured output.

- `threshold=-1` and `12`, with `bound=54`, `window=4`: the `k` values start at 12, so `returnThreshold > k` is not expected to fire; the counter should show ordinary completion through the nested break path.
- `threshold=13` and `100`, with `bound=54`, `window=4`: the first inner-loop value is `k=12`; the early return should fire on the first K-body visit. `runCounts` increments before the condition, so it exposes that visit.
- `bound=0` and `8`: the `j` loop has no value greater than the outer `i` range, so K is skipped. `bound=13` makes `j=12` available, exercising the branch into K.
- `window=0`: K is entered but its body executes zero times before the `break j`; `window=1` provides one body visit; `window=4` provides the four-value window.

The runner combines these into ten fixed inputs. It reports both result and count encoding (`normal = tmp * 1000 + ticks`, early return = `ticks`) so equal `testFor` values cannot conceal different K-body execution. Exact output is not asserted here; root must capture original and compare each regenerated-source runtime against it.
