# Latch-origin adversarial test delta (Luna v1)

`adversarial-latch-tests-luna-v1.patch` is a private, test-only two-file delta. It strengthens existing assertions without adding fixtures, test functions, counts, or a general source parser.

The loop-arm join assertion uses the existing `LoopIfJoin.run` fixture: BCI 23 is its physical `goto 15`. It checks that the source map includes BCI 23 and that a derived segment for it begins at the actual `while` span, while retaining the existing shape and outer-tail assertions.

The double-jump assertion checks every existing transfer anchor has source-map coverage and that none of its source segments starts at a `while` or `for` header. It examines only the first line of each segment, including `label: while` / `label: for`, so an enclosing `if` segment containing a nested loop is not mistaken for a loop-header span.

The assertion design uses the recorded old CLI observations in `transfer-map-old-cli-root-v1/anchor-spans.json`: the listed `break`/`continue` anchors map to their transfer or enclosing `if` spans, while `LoopIfJoin.run` BCI 23 has no old source-map span. Those observations came from the old frozen CLI before the candidate was applied. They are test-design evidence only; they do not establish that this candidate passes or that the test delta was run.

No test source was edited, and no Git, Cargo, JDK, or CLI command was run for this delta.
