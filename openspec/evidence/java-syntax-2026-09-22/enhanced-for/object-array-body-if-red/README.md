# Object-array nested-if RED

This is the preserved first Object[] experiment. Its class bytes have SHA-256 `e0b6e524da02db14fae69bb6924c7dd30c23d8b42c81b2e68c996eeee444c4db`, identical to the initial audit input. The original class and runner compile and run with `-Xverify:all`; the frozen CLI emits its unmodified full class source in `jarde.java.txt` and structured report in `jarde-report.txt`.

The CLI exits successfully, but both loop methods are refused with `jre_region_loop_shape`: the loop headers are BCI 10 and BCI 18, and their nested conditional bodies leave blocks outside the region walk. Whole-class `javac --release 8` fails with two missing-return errors. `jarde-runner-javac.status` and `jarde-runtime.status` are `skipped`, with the reason preserved in stderr; no failing source was edited or executed.

Replay with `python3 openspec/evidence/java-syntax-2026-09-22/enhanced-for/object-array-body-if-red/run_red.py`. The frozen CLI hash is checked before and after the replay.
