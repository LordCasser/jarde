# Compound lvalue permanent-fixture replay

The permanent inputs, class hashes, bytecode boundaries, and expected seven-line execution are
documented in `../../../../../tests/fixtures/p3-compound-lvalue-updates/README.md`. From the repository
root, rerun the complete original/JADX/Jarde comparison with:

```sh
python3 openspec/evidence/java-syntax-2026-09-22/compound-assignments/permanent-fixture-replay/replay_fixtures.py
```

The script checks the frozen CLI hash and exact source-to-class bytes, verifies and executes each
original or patched input with `java -Xverify:all`, and writes generated complete class sources and
logs under `evidence/`. `evidence/summary.json` records the full comparison. The root agent reran
the script independently: exit 0, main class SHA-256
`889f36d3a5c06951d32762c8914829c059d8b1d6692001c08df73404c92f673f`, 12 Code methods,
seven original/JADX-matching lines, six Jarde differences, 13 boundary executions (all original/JADX
equal), and an unchanged frozen CLI hash. Two ordinary `=` boundaries already match Jarde; the
remaining boundary outputs are not accepted as restored `+=` behavior.
