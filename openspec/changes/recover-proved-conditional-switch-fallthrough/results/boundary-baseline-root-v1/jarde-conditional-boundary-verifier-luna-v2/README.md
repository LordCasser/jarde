# Conditional switch boundary observation verifier v2

This private draft verifies collector-v3 execution records only. It has not run the verifier or any JDK, JADX, candidate CLI, Cargo, or Git command. It reads the completed root record at the caller-supplied `--execution` path, and writes a new acceptance file exclusively:

```text
python3 /private/tmp/jarde-conditional-boundary-verifier-luna-v2/verify.py \
  --execution /private/tmp/jarde-conditional-boundary-preflight-root-v2/execution.json \
  --acceptance /private/tmp/jarde-conditional-boundary-verifier-root-v2/observation-acceptance.json
```

The verifier independently checks each command's stdout/stderr SHA and byte count, argv, cwd, resource telemetry and environment fields, then binds every command row to the corresponding leg result. It intentionally does not require `test_summary_check.ok` for JVM/JADX commands: the inspected root record shows that the generic Cargo summary parser reports false for a successful JDK 8 JADX launch (`expected [(7,0,0)]`, no Cargo tests in stdout). Those rows are instead checked by exit status and raw streams.

For each JDK it verifies the copied original source and Runner bytes, original class set and class hashes, and the deterministic JADX input JAR's sole entry bytes against the compiled class. It verifies the emitted JADX `.java` source against the saved source, its package-prefix-only Runner adaptation, each leg's javac/java argv, and the resulting class set. For Jarde output it checks the complete JSON report bytes, generated source bytes, exact original Runner plus optional package line, six unique physical member ordinals exactly `0..5`, consistent class owner identity and length, every source-map origin's full method identity, and the recomputed full source-map SHA against the execution row. It recomputes default/all source and map equality.

Compile failures are preserved as observed outcomes and must have no runtime command. Runtime differences are calculated from raw streams; the output records line-level differences and exit/stderr equality for JADX and candidate programs separately. `runtime_acceptance` can be true only when all four candidate profiles compiled, ran, and matched the original oracle exactly. An empty candidate runtime set remains false. Product acceptance and CF12 completion are always false; this document verifies baseline observations only.

The verifier also checks the current frozen typed baseline pins, JDK manifest and live tools, pinned guard/JADX, execution ending SHA snapshot, and the closed file inventory. Root's completed execution was read to confirm the actual field names and values; no verifier run was performed. Python AST parsing passed. Script SHA-256 is recorded below.

Verifier SHA-256: `17c8c5b5cd498bdb6547b599d06717c0c3bea53e10ddc3121cf018bb3d0465ef`.
