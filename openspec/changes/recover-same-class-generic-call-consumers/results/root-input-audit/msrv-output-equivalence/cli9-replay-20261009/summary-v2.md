# CLI9 saved-output equivalence review (append-only v2)

This review compares each saved CLI8 `class-source` invocation against a direct CLI9 replay. It does not rebuild the project, regenerate JADX output, or alter the first replay manifest or runner.

The saved CLI8 records contain 342 invocations: matrix 144, nested 4, field 46, constructor 80, and raw 68. The field run could replay 28 calls; 18 `source-rebuilt-temp-jar` inputs had been removed and are itemized with their original jar/class/source hashes and CLI8 output metadata in `field18-recovery-linkage.json`. Those 18 require the separately assigned GC09 full recovery/comparison and are not claimed complete here. The replay ran 324 calls: matrix 144, nested 4, field 28, constructor 80, and raw 68.

All 324 CLI9 exit codes match their recorded CLI8 exit codes. This includes the four `RawNewHold` constructor calls, whose CLI8 and CLI9 exit codes are both 4 and whose JSON reports compare semantically equal. Across the text-format calls, all 216 source outputs (matrix 144, nested 4, raw 68) match byte-for-byte. The 108 JSON-format field/constructor reports match after removing only the exact `elapsed_millis` key when its immediate parent is `usage`.

The report written to stderr for text-format runs is a flattened TOML-like representation, not valid TOML because it emits `null`. For semantic comparison, the analysis maps only an exact scalar ` = null` to a sentinel string before parsing with Python `tomllib`, then removes only `usage.elapsed_millis` entries structurally. Every one of 216 such reports matches after that comparison. The raw stderr byte comparisons and hashes remain in the first manifest; no timing fields were removed from saved output files. JSON reports are parsed structurally and use the same exact-parent rule. No other fields are normalized.

The original JSON formatting can differ when elapsed values differ: raw report bytes match for matrix 10/144, nested 0/4, field 6/28, constructor 21/80, and raw 2/68. After the single timing-field mask, all 324 reports match semantically and all 324 exit codes match. The first manifest also retains each actual stdout/stderr file and SHA-256.

CLI identities: CLI8 `/private/tmp/jarde-generic-calls-candidate-v8-cli`, SHA-256 `8770d823c70e7f61749cac836e468c0a991093822d1926822d4769adc1cf7339`; CLI9 `/private/tmp/jarde-generic-calls-candidate-v9-cli`, SHA-256 `5bb2fcfdcf958839f0b1e060a2e55114510c6115cb225ec3279d7d7adf95e006` (v42 source archive `fc4220ac5eb6906a786340a6ef5f8dd840ccbc14790df7bcee115f7c9f78fce8`).

Evidence hashes:

- Runner `compare-cli-output.py`: `5710ccae52fe6dd32332a99a5a1fb13d25febae85ea4c5eac61a2b5cec1ea305`
- First replay manifest (unchanged): `3e01e441312fb0f0f557031d62b0f0463dfd0a186fc63ccd43ececfa3148c411`
- This v2 analysis: `982097210403328b980b30609cbe16cfcb6596642dcea9bbf2a9faae68d2798c`
- Field18 linkage: `e844416805d608e9e5203824553d56841108bd00e167bb914698349f2a6aef50`
- CLI9 metadata `results/local-gates/candidate-cli-v9.json`: `12808cf06dd4f63ca9f6fbee6f4f0a0585be0431e19d8fd1557f21c9e59ca4de`
