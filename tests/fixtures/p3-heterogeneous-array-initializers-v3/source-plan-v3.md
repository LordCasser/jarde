# Heterogeneous reference-array initializer fixture plan v3

The fixture preserves the complete v2 `factory` and `direct` class families and changes only the observer. Each family contains its own six top-level classes in one archive per compiler leg. Factory calls remain the positive type-proof inputs; direct constructor expressions remain an independent, verifier-valid constructor/array-store composition control. Refusal in the direct family is not an illegal-assignment claim.

The observer reads the one known non-empty array element into an `Object` local and uses a simple `if`/`else` to print either `null` or the element class name, followed by the side-effect trace. This removes the v2 loop/ternary/array-read observer from the recovery surface while preserving the original deterministic output. Every initializer, helper, hierarchy class, and control remains present.

Each compiler leg compiles all six classes together using Corretto javac 8 (`-source 8 -target 8 -g:none`) or OpenJDK javac 23 (`--release 8 -g:none`), with explicit empty classpath and sourcepath. The earlier v2 sources, class files, raw logs, runner and manifest are historical evidence under `openspec/changes/recover-heterogeneous-array-init/evidence/heterogeneous-array-initializers-v2/`; the v3 fixture does not read or depend on them.
