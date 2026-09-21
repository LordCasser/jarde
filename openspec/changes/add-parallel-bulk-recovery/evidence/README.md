# Evidence for `add-parallel-bulk-recovery`

Raw samples and the scripts that produced them, kept in the repository so the numbers in
`../verification.md` survive the loss of any temporary directory.

| file | what it is |
| --- | --- |
| `raw-timing-first-pass.jsonl` | the first pass: 3 interleaved CLI repeats per configuration |
| `raw-timing-interleaved.jsonl` | 10 interleaved CLI repeats per configuration (the numbers verification §9 quotes) |
| `campaign.py` | the first-pass campaign driver (absolute paths in `/tmp` documented as prerequisites below) |
| `campaign-interleaved.py` | the A–E arm driver that produced both `raw-*` files; every sample is appended, nothing is dropped |
| `join.py` | the member-set reconciliation against the jadx join documents |
| `comparison-notes.md` | what is and is not comparable across tools |
| `cost-attribution.md` | where a whole-scope run's time goes, before and after the ledger and window work |
| `arm-comparison.md` | A–D arms on the frozen corpus, plus the degenerate E shapes and the protocol's gaps |
| `parallel-diff-whitelist.md` | the fixed differences between 1 and N workers, and what may never differ |

## Running them again

The A/B arms use the **frozen** historical harness at `/tmp/jarde-bench/sweep3/target/release/jarde-sweep3`
(built at `8807fa5`), which was still present at the time of writing; C–E use `jarde-cli export` from
this repository's release build. Both scripts read the vulhub artifacts named in `verification.md`
§11 and the roots documents in `/tmp/jarde-b10/roots/`. A machine without those paths can still run
the C–E arms by pointing the `--input` and `--roots` arguments at its own copies.

No file here is a CI dependency: `tests/p5_bulk_corpus.rs` and the `bulk_recovery_*` suites are what
the gates read, and they build their inputs in-process.
