# Conditional-switch boundary observation verifier (draft)

`verify.py` is a private, not-yet-root-reviewed independent verifier for the future collector v3 output. It does not run JADX, JDK, the candidate CLI, Cargo, or Git. It only reads the `--execution` tree and the already-frozen fixture/baseline inputs. Run it only after the collector v3 record is available:

```text
python3 /private/tmp/jarde-conditional-boundary-verifier-luna-v1/verify.py \
  --execution /private/tmp/jarde-conditional-boundary-preflight-root-vN/execution.json \
  --acceptance /private/tmp/jarde-conditional-boundary-verifier-root-v1/observation-acceptance.json
```

The acceptance output is exclusive and observation-only. The collector may record expected observation failures (for example, a generated full-class compile failing); the verifier recomputes the exact failure list from command exits and raw streams and accepts those only as observations. The script checks the v3 collector/schema/completion marker, original fixture pins, exact typed baseline CLI/metadata/build/runner/guard bindings and live source pins, manifest-pinned tools, command labels and argv, exact command-to-leg references, resource guard telemetry, environment stripping, every stdout/stderr byte count and SHA, source/report and Runner bytes, and the closed file inventory. It recomputes default/all whole-source and per-method full source-map/physical identity equality. It also requires both original JDK runtime triples (exit/stdout/stderr) to match.

Compile failures are preserved as observations. A failed compile must have no runtime row; it is never treated as a runtime pass. Runtime comparisons are computed line by line from raw stdout, with stderr and exit checked separately. The report records observed differences without expecting any particular mismatch; known v1 examples are intentionally not encoded as truth. No runtime command means no runtime acceptance. The verifier always writes `product_acceptance: false` and `cf12_complete: false`.

This draft has not been run against collector v3, whose final schema and output are still being prepared. Static Python AST parsing passed. Script SHA-256: `614ff89d4634a4e52c6fade23a898cdb6396ca9d6527e22575818a843153f8ee`.
