# Where a whole-scope run's time goes (2026-09-21, one machine, release build)

Produced by `examples/bulk_scope_sweep`, whose three sink modes pay exactly one more layer
each: `discard` counts records only, `encode` serializes every record with the same
`serde_json` the CLI uses and drops it, `write` appends the encoded line to a scratch file.
The counted work is identical in all three (the counters printed beside each row), so the
columns differ only in what the sink is asked to do.

| corpus / mode | jobs=1 | jobs=2 | jobs=4 |
| --- | --- | --- | --- |
| bcprov discard (2,430 classes, 14,495 bodies, `ir_items` 11,025,007) | 3.26 s | 3.50 s | 3.84 s |
| bcprov encode | 3.55 s | 3.60 s | 3.90 s |
| bcprov write | 4.78 s | 4.23 s | 4.95 s |
| s2-009 discard (53,247 bodies, `ir_items` 32,966,465) | 11.16 s | 11.21 s | 12.23 s |
| s2-009 encode | 12.93 s | 11.63 s | 12.75 s |
| CLI `export`, median of 10 interleaved runs (writes 357 MB / 2.37 GB) | 4.41 / 19.31 s | 4.44 / 15.71 s | 4.65 / 16.90 s |

Read alongside `raw-timing-interleaved.jsonl` (CLI, ten interleaved samples per
configuration) and `raw-timing-first-pass.jsonl` (the earlier three-sample pass).

What the table supports:

* **The parallel loss is inside the operation.** With a sink that only counts, four workers are
  slower than one on both corpora, and the counters are identical — so encoding, writing and the
  stream's backpressure are not what removes the gain. The earlier attribution to "single-threaded
  delivery" is refuted by these rows.
* **Encoding is cheap; writing is not.** Serializing the whole stream costs 0.3 s (bcprov) and
  1.5 s (s2-009); writing 357 MB / 2.37 GB to a file costs about another 1.5 s / 6 s and is the
  noisiest column.
* **A caller without a store pays for the walk.** The same s2-009 scope run through the example
  without a facts store did not finish inside a 30 s wall clock (its counters show 5.7-6.6M
  `archive_entries` instead of 10,358): a container's directory is re-parsed for classes read
  after the walk moved on. The CLI attaches one by default and the example now does too.

Not supported by this table: any statement about other machines, about page-cache state (never
controlled here), or about jadx. Single samples per cell.
