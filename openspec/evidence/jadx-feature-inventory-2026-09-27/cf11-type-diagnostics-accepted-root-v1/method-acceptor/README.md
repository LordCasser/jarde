# CF11 method-signature diagnostic acceptance prep

This is an independent, read-only acceptance script for the one existing `TestNestedLoops` API test run. It accepts evidence of the observation path only. It does not treat `candidate=None`, an SSA rejection, or a successful API test as generic-source proof or as a generic fix.

The script verifies the invocation and receipt SHA-256 values passed by the caller; raw stdout/stderr hashes; exact runner, private patch, fixture, test and environment pins; the 4,744-file before/during/after closure and current live bytes; only the two intended temporary source changes; byte-exact restoration; live HEAD and clean working tree; the exact test and cleanup command rows; 5 GiB free-space and 1 GiB target limits; default Rust stack; the cleanup-only `cargo clean` exception; and target absence. It derives the actual number of complete per-request diagnostic groups from the raw logs, checks each group and request-to-request equality, and maps the observed SSA rejected line to the pinned candidate source. It retains the actual parsed Signature, erasure, refusal and same-run Program/SSA text in the output.

Prepared pins: diagnostic runner `6c4ad2598a898b104560bb4aed07f547584107d1656d2f4d8a9696eb5ee6c6ed`; patch `c736eb2b70475d2a00f10d8eebb5e92c990910a5c4a413ed4cf32f65e9c3ac11`; source-pin manifest `d59db1838d9a0eaa4628457ddd6929dccfdaed455097de5b959a78470fbb9f6d`; closure helper `63e52b55fe5d20d5f2ece98ac80676ccdb2de8dfd33d7199b688e075fb1e9673`.

The parent supplied the completed run receipt SHA `f9e84144b3e16ba5e0356fa726e0ee07f7732e94ffee03cf423e08bae9171e02` and invocation SHA `57725966867c79820c8ff645018ac12401f0dcba905744161918569d4e085d52`. Those values are recorded here for invoking the parent-owned independent review; this preparation does not run the acceptor.

Example for the parent to run after reviewing this script:

```sh
python3 /private/tmp/jarde-cf11-method-signature-accept-prep-luna-v1/accept.py \
  --receipt /private/tmp/jarde-cf11-method-signature-diagnostic-run-root-v1/execution.json \
  --receipt-sha256 f9e84144b3e16ba5e0356fa726e0ee07f7732e94ffee03cf423e08bae9171e02 \
  --invocation /private/tmp/jarde-cf11-method-signature-diagnostic-invocation-root-v1/execution.json \
  --invocation-sha256 57725966867c79820c8ff645018ac12401f0dcba905744161918569d4e085d52 \
  --out /private/tmp/jarde-cf11-method-signature-acceptance-root-v1.json
```

Status: prepared only. No build, test, or acceptance run was performed by this prep.
