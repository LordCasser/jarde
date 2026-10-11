# NumericComparison private input review (v2)

This private source set contains exactly two top-level target classes: `cf11.ForConditionNumeric` and `cf11.Runner`. The runner uses three ordinary static helpers (`runLong`, `runFloat`, and `runDouble`) with direct calls; it has no nested interface, lambda, anonymous class, or additional fixture dependency. The manifest-shaped `sourcefiles` entries in `pins.json` record each class/source filename and the current source byte length and SHA-256. `compiled_or_run` is false.

The three target methods keep the requested counted-loop shape and compare `values[i] < bound`. Induction local `i` is consumed as the array index. The 15 runner cases preserve v1 labels and scenarios: three finite `{1, 4}` / bound `3` normal cases, four first/later NaN cases, three null cases, three one-element bounds cases, and Long MIN/MAX stopping boundaries.

By Java source semantics, predictions are: normal cases return 1; first NaN returns 0 and later NaN returns 1; null inputs throw `NullPointerException`; one-element inputs throw `ArrayIndexOutOfBoundsException` on the next condition check; Long MIN and MAX boundary cases return 0 and 1 respectively. These are predictions only. The original-class runtime oracle remains pending the parent compile/run, and this source has not been compiled or executed or compared with oldCLI/newCLI.

The accepted controls archive is pinned in v1. This v2 changes only the runner shape to match the collector's exact two-class target set; it does not change the three target methods or expand the NumericComparison scope. No shared source or build artifacts were modified.
