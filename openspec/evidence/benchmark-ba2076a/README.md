# `ba2076a` round: assets and how to read them

Archived from `/tmp/jarde-bench/` so the round survives that directory (the protocol's own rule).

| file | what it is |
| --- | --- |
| `report.md` | the round's report (`jadx-vs-jarde-ba2076a.md`) |
| `handoff.md` | the reviewer's handoff, including the decision it asked for |
| `harness-diff.md` | the experiment that located the 1.125× gap between the two harnesses |
| `bulk.jsonl` | C/D arm samples: three sink modes × 1/2/4/8 workers, ten interleaved repeats per cell |
| `cli_export.jsonl` | the CLI's own `export` runs (`auto` and `jobs=1`) — a different path from C/D |
| `jadx.jsonl` | the same round's jadx runs |
| `degenerate.jsonl` | the 1 MiB output-allowance shapes |
| `classification.json` | how the sampled methods were classified for the correctness pass |
| `sample.json` | the sampled probe results behind 421/424 |
| `corpus5.json` | the 12 artifacts with full SHA-256 |
| `load.log` | machine load during the round |

Read the numbers with the protocol's own caveats: page cache was not controlled (per-configuration
warm-up only), so the time columns are shape evidence rather than steady state; the C/D arms are the
example driver and not the CLI; and the cross-tool rows measure different products (whole classes
against per-method records).
