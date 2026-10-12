# CF11 structured-list controls observation checker

This is a private v6 copy of the v5 read-only checker with one fixture-scoped exception-table adaptation. It accepts an exception handler only when the captured target source SHA-256 is `f0114ff1752aede3a2b19df1887fedb289c6d08b63a84b1913a86eb1d2cbb236`, the physical owner is `cf11/UpstreamNestedLoops`, and the method is `handled(Ljava/util/List;Ljava/util/List;)V`. In that case it requires exactly one row: `[0, 47) -> 50`, `java/lang/RuntimeException`.

For every method, the checker decodes the original classfile Code attribute's full exception table and compares every row to the saved original `javap -p -c -s -v` output. It also requires the pinned classfile parser's handler count to equal the decoded table length. All other methods retain the zero-handler requirement. The previous v5 failure invocation and its raw streams remain preserved at their original paths.

The remaining v5 checks are retained, including input and tool pins, all command receipts and raw hashes, method maps and BCI coverage, JDK/profile checks, compile output inventory, and conditional runtime comparisons. Run from this directory with `python3 -B check_controls.py --execution /private/tmp/jarde-cf11-structured-list-controls-baseline-root-v1/execution.json --out /private/tmp/jarde-cf11-structured-list-controls-observation-acceptance-root-v1.json`.

This checker verifies the captured observation only. It does not claim behavioral acceptance or accept a product fix. The user-facing acceptance output should be created by the root's independent invocation.
