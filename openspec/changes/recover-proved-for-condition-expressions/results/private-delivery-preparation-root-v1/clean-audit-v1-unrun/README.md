# For-condition clean-main audit preparation v1

This is a private, unexecuted audit candidate derived from the CF11 delivery-prep v4 clean-main audit. It keeps the frozen source-closure checks, live SHA pins, Git tree blob and live blob OID comparison, real `origin` branch proof, and per-worktree cleanliness/ancestry/target checks. The acceptance requires exactly 15 listed worktrees: the main checkout plus 14 detached checkouts, each checked individually.

Invocation after the product and docs work is complete:

```text
python3 clean-main-audit.py OUTPUT_DIR EXPECTED_HEAD EXPECTED_BUILD_RECEIPT_SHA256
```

Both values are runtime inputs. `EXPECTED_HEAD` is the actual full commit ID to audit. `EXPECTED_BUILD_RECEIPT_SHA256` must be read from the final build receipt after root-v2 completes; no receipt SHA is frozen here because that receipt does not yet exist. The audit binds the supplied SHA to `/private/tmp/jarde-for-condition-final-lint-build-root-v2/execution.json`, checks the runner and guard/scanner pins, the exact final CLI identity, `source_pins_before == source_pins_after` (the runner has no `source_pins_after_format`), the 4718-source closure, and the five actual command argv/exit/guard outcomes: fmt, workflow-derived clippy, OpenSpec strict validation, CLI build, and clean.

The frozen candidate CLI identity is `/private/tmp/jarde-for-condition-final-cli-root-v2`, 61,213,840 bytes, mode `0555`, SHA-256 `51c9a1326031fd10444f7e18a055c6daecef233423c90478257becf24d1e0bd4`. At preparation time the CLI and build receipt were not present, so this candidate cannot establish acceptance. The script was checked with `ast.parse` and `symtable.symtable`; it was not imported or executed. No Git, Cargo, JDK, JADX, or CLI command was run.
