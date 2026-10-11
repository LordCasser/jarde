# Private v3 verifier correction

The actual v2 candidate run was retained unchanged at `/private/tmp/jarde-compound-negative-render-final-cli-v2/execution.json` (SHA-256 in `pins.txt`). It has status `observations_collected`, all 12 javap/class-source commands succeeded, all four fixtures are present with both `default` and `all` profiles, and the top-level frozen `expected_class_owners` table contains `cycle`, `irreducible`, `no-repeat`, and `clone`. Every per-fixture row omits `expected_owner`, as does the v2 collector source. The recorded v2 verifier failure is preserved in `verifier-v2.stderr.raw`: it stops at `verify.py:355` on the redundant per-fixture field assertion.

The only v3 change removes that assertion. It retains `expected_owner = EXPECTED_OWNER[name]` for downstream verification against actual class/method identities and source-map owners. It also retains the top-level `record['expected_class_owners'] == EXPECTED_OWNER` assertion and the independent `verify_owner_oracle(record)` check against the frozen BLAKE3 oracle/raw/tool pins and SHA-256 owner table. The actual execution record and collector are not changed; no CLI, javap, Java, Git, or toolchain execution was repeated.

The candidate AST was parsed successfully with Python's `ast.parse` (syntax check only; the verifier was not executed). Source, patch, retained execution, and retained failure hashes are listed in `pins.txt`.

Files in this private v3 directory:

- `verify.py`: candidate verifier source.
- `verify.py.patch`: exact one-assertion deletion against the v2 verifier.
- `verify.py.base`: byte-for-byte copy of the v2 verifier.
- `pins.txt`: source/patch and retained actual evidence hashes.
