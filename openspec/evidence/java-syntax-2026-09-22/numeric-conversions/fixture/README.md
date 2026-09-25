# Primitive conversion fixture evidence

`PrimitiveConversions.class` is the only frozen class. `run_audit.py` compiles the probe, support,
effect, and runner sources with `javac --release 8 -g:none`, verifies the generated probe bytes
against `v8/PrimitiveConversions.class`, installs those exact bytes on the original side, and runs
the runner with `java -Xverify:all`.

The audit then feeds the complete class to the current CLI, compiles the full Jarde text with all
source-only helpers, and runs it when compilation succeeds. It also compiles and runs the complete
JADX output with the same helpers. `summary.json` records the 15 opcode counts, 31 methods and
Code attributes, 41 runtime lines, 8 effect-order cases, all stage exit codes, and the CLI hash
before and after. `jadx-differences.txt` preserves the raw output differences caused by JADX
dropping intermediate conversions; it is evidence, not an oracle.
