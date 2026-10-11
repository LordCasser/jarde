# For-condition clean-main audit preparation v3

Private v3 restores the previously accepted candidate CLI identity as an explicit, exact receipt contract. The CLI must be `/private/tmp/jarde-for-condition-final-cli-root-v3`, exactly 61,213,840 bytes, mode `0555`, SHA-256 `51c9a1326031fd10444f7e18a055c6daecef233423c90478257becf24d1e0bd4`; the actual CLI file and receipt's full `candidate_cli` object must both match those constants. `candidate_byte_identity: true` alone is insufficient.

Invocation after the final build receipt and product/docs commit are ready:

```text
python3 clean-main-audit.py OUTPUT_DIR EXPECTED_HEAD EXPECTED_BUILD_RECEIPT_SHA256
```

The HEAD and receipt SHA are runtime values and are not predicted. The final-build runner is pinned to `/private/tmp/jarde-for-condition-final-lint-build-root-v3.py`, SHA-256 `85e25bb6d87a5993305c0fdeb3c58cd9300886701cf8158267df267ce0b92f1e`; receipt schema remains `for-condition-final-lint-build-root-v1`. The audit retains the 4,718 exact source pins with `source_pins_before == source_pins_after`, five exact command argv/exit/guard checks, Git tree/live blob closure, remote/main proof, and individually checked 15 clean checkout ancestry/target conditions.

This is preparation only. No audit, Git, Cargo, JDK/JADX, or CLI command was run.
