# Workspace v9 local acceptance archive

This bundle records local full-workspace acceptance only. It makes no product CI or push-clean claim.

- Acceptance: `accepted_local_workspace_only`; SHA-256 `6b119c7fedec7a84c4c9cd0aa1b66d01edec49c6fd022fe8b4de5f8ae2cbf188`.
- Workspace receipt SHA-256: `1b0215f1ae01bf41515cc4fbda3c3e3d7667a7d4e8524c311352956a0980fa56`; records `195` (including format check, metadata, and final cleanup).
- Test batches: `192` total, 96 for each of the two pinned seeds; target inventory: `364`.
- Source pins: `4854` entries; before/after maps match.
- Raw streams: `390` members, each checked against receipt bytes and SHA-256.
- Focused receipt: `887bff1cb4999f13d03e9d3e6b411a032c46ce26151b0f566ea78448a7869401`, summarized in `focused-receipt-summary.json`; the related root summary records 705 passed, 0 failed, 4 ignored.
- Earlier durable archive: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-ordered-field-concat-operands/results/reader-census-fix-and-final-local-root-v1/capture.tar.gz`; its verified 29-member archive is referenced by `prior-localarchive-reference.json`, not copied.

The compressed bundle and the detached manifest list the evidence under the tar member paths; no unpacked payload directory is created.
