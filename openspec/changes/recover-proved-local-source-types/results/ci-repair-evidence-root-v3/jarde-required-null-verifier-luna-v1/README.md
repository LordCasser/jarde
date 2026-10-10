# RequiredConversions and NullThenBuilder replay verifier draft

This is a read-only verifier draft for the two existing 19-command executions. It writes no acceptance file and launches no process. It checks the actual `execution.json` records, all 38 raw streams per execution, both CLI report documents, class/member identities, the exact 8-leg JDK8/JDK23 × original/default/all/JADX matrix, whole source bytes, compile/runtime argv, compiled class presence, and runtime stdout/stderr.

`verify.py` preserves the `meet-char-int-full-class-comparison-root-v1` execution schema. The candidate CLI is pinned through the actual typed metadata at `openspec/changes/recover-proved-local-source-types/results/candidate-cli-typed-root-v1.json` and the frozen binary path `/private/tmp/jarde-proved-local-source-types-cli-v1`. The expected CLI SHA-256 is `e6978d74d0935621e73db7d2640fc5a545e4ddfd4c090027ebb84930b5419403`; the JDK manifest SHA-256 is `ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec`.

The frozen class inputs and independent execution records are:

| Case | Fixture SHA-256 | Reported owner BLAKE3 / byte length | Execution SHA-256 |
| --- | --- | --- | --- |
| `RequiredConversions` | `d71eeaa5dc3eb66eafffda9e0c509eae43e3dde566b67df0006348352778d1e4` | `96ed4aee49026f64179075ee3843709aa7e7a5cd02ed2d05f7a11e1b9c385d3d` / 1031 | `a2b069ba2101052f73d9adf69d380587086a93f66793ca22adeb01d9ad0b9c28` |
| `NullThenBuilder` | `93acdfe27b312e2c98666a00dbfcb5f61299b03f62ac0dbd916b385232f260b3` | `f24b5b45eb94733793e5ee6d7f52f9cfe710d89bdbd65140930822d38f84a77b` / 629 | `9d4cbb606405c0928cc79b0820ad36db9bd100ad860e38a6ba9f06475e44052e` |

The typed CLI metadata file SHA-256 is `6295d4cc0c65bd5b5c476f71e44cf3abf513f77d854854bfdea777cbaf2be943`. The verifier source SHA-256 is `f879f242ccdb6be9d855f794f995b4a3524ccb7ced2a6245161820f9f5174441`.

The verifier recomputes each fixture SHA-256 from its class bytes and checks the raw CLI reports' class and physical member/field BLAKE3 identities against the frozen values. It does not carry a separate Python BLAKE3 implementation; the raw report SHA-256 closure and direct fixture SHA-256 check bind those report values to the captured CLI observations.

`RequiredConversions` has the original Java source, so the verifier compares its complete bytes, the complete JADX source, and complete default/all Jarde class text before checking all 13 methods and the `field:I` field. Its runner checks the entire UTF-16 `char` domain, 0 through 65535, through `declared`, `assigned`, `written`, `castPart`, and `castPartLast`; the expected output is `chars=65536,sum=2147450880`. It verifies all eight runtime legs. The runner does not directly call every remaining method; those methods are included in the full-class source/member checks and compilation.

`NullThenBuilder` has no original `.java` source. The original legs therefore use the canonical fixture class bytes as the oracle and compile only the runner against that class. The verifier checks all three methods, the exact original class bytes copied into each original leg, all four whole-class sources where available, and both boolean arguments with expected output `true=7` / `false=6` in all eight runtime legs. This is a bounded fixture replay; it does not claim original Java source equivalence.

The draft has only been parsed as Python syntax. It has not been executed. Root should review it before running; only root should create any acceptance record.
