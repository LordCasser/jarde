# Dense `tableswitch` source-order fallthrough audit

This fixture isolates the `tableswitch` form of the source-order fallthrough bug. `javac --release 8 -g:none` stably emits a dense `tableswitch` for keys 1 through 4. In source order, `case 4` comes first and falls through into the shared `case 1` / `case 2` body; `case 3` and `default` follow. Keys 1 and 2 therefore share BCI 39, while key 4 enters at BCI 36.

The checked class is 390 bytes, SHA-256 `f45d78320286e1912b30f4b01524b17695a1cef269a0204b18d3d7191ce16f84`. In `javap.txt`, the table maps keys 1 and 2 to BCI 39, key 3 to BCI 45, key 4 to BCI 36, and default to BCI 51. The actual code order begins at BCI 36 (`+40`), then BCI 39 (`+1`), confirming real fallthrough and a nonnumeric source order.

`original-run.txt` and `jadx-run.txt` match line for line; `jadx-diff.txt` is empty. The frozen jarde CLI was verified before use at SHA-256 `908c472560d6146354e1fd679c595e884afadf5ec7051727b51e79a1f95c6570`. Its whole-class output compiles and runs, but moves `case 4` after `case 1` / `case 2`, inserts a break, and leaves BCI 39 as an `@bytecode` marker. Consequently both input-4 rows return 40 instead of the original 41. The other six rows match, and every call increments the selector counter exactly once. See `jarde.java.txt`, `jarde-report.txt`, and `jarde-diff.txt` for the raw decompilation, report, and output comparison.

Replay from the repository root with:

```sh
python3 openspec/evidence/java-syntax-2026-09-22/switch-fallthrough-order/tableswitch/run_audit.py
```

The script checks the frozen CLI hash, compiles the fixture with Java 8 source/target and no debug metadata, verifies that `javap` contains the expected dense `tableswitch`, decompiles with JADX and the frozen CLI, then recompiles and runs each recovered class with `java -Xverify:all`. It recreates only `/tmp/jarde-tableswitch-fallthrough-audit` and writes evidence beside this README. Required tools are `javac`, `java`, `javap`, `jadx`, Python 3, and the frozen CLI at the path recorded in `run_audit.py`.

Evidence files:

- `TableswitchFallthrough.java`, `TableswitchFallthroughRunner.java`: minimal fixture and eight-input runner.
- `run_audit.py`: replay script.
- `source-javac.log`, `javap.txt`: source compilation and bytecode evidence.
- `original-run.txt`, `jadx-run.txt`, `jarde-run.txt`: raw JVM outputs; matching `*-javac.log` files preserve compilation output.
- `jadx.java.txt`, `jarde.java.txt`: recovered sources; `jadx.log`, `jarde-report.txt`: tool output.
- `jadx-diff.txt`, `jarde-diff.txt`, `summary.json`: comparisons and recorded hashes/status.
