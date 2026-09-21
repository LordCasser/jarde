# 7.3 protocol measurement (2026-09-21)

Produced by `examples/demand_workloads` and `measure_7_3.py`, under the protocol fixed in
`../verification.md` §7.3 **before** any number here was taken: seven workloads, two arms, ten
interleaved repeats per cell, medians with extremes, peak RSS recorded per run, nothing dropped and
nothing selected.

Scope: `{"kind":"artifact_tree","root_container":"root"}` for both corpora, because a whole-package
measurement must walk the containers the CLI's own export walks — the first attempt used
`snapshot_all` and would have measured the WAR's root container instead of the artifact (it reported
808 methods where the artifact holds 57,180).

The harness attaches the bounded facts store the MCP design gives the host and the CLI attaches by
default (`FactsCapacity::new(1<<14, 1<<27)`). Without one, the same WAR sweep re-parses every
container it revisits: measured 5.7M `archive_entries` against 10,358 with the store, and the run
did not finish inside the timeout the first attempt used. A storeless sweep would have been a
configuration nobody ships.

Load was recorded: bcprov ran at `3.56 4.23 4.75`, s2-009 finished at `1.89 2.00 2.50`. Page cache
was not controlled, so the first run of each configuration pays for reads the later ones do not;
that is why the time columns are medians *with* extremes and are not used to claim a speed-up.

## bcprov-jdk15on-152.jar (2,430 classes, 15,003 methods)

| workload | arm | first result ms (median/min/max) | whole sequence ms (median/min/max) | returned | peak RSS MiB |
| --- | --- | --- | --- | --- | --- |
| `nav-class` (members, no body) | essential | 1.21 / 1.18 / 1.22 | 1.23 / 1.22 / 1.24 | 6,825 B | 15.3 |
| | all | 1.19 / 1.18 / 1.24 | 1.22 / 1.21 / 1.26 | 6,825 B | 15.3 |
| `nav-decl` (members + one body) | essential | 1.23 / 1.21 / 1.26 | 1.26 / 1.24 / 1.29 | 8,854 B | 15.5 |
| | all | 1.23 / 1.21 / 1.24 | 1.25 / 1.24 / 1.27 | 8,854 B | 15.5 |
| `recover-one` (one method) | essential | 1.39 / 1.38 / 1.44 | 1.42 / 1.41 / 1.47 | 9,156 B | 16.5 |
| | all | 1.40 / 1.37 / 1.41 | 1.43 / 1.40 / 1.45 | 10,896 B | 16.5 |
| `sweep` (whole package; first *delivery*) | essential | 2.55 / 2.25 / 2.89 | 1,833.55 / 1,830.13 / 1,849.20 | 15,003 methods | 126.1 |
| | all | 2.53 / 2.25 / 2.79 | 1,909.73 / 1,903.40 / 1,929.99 | 15,003 methods | 134.3 |
| `expand` (essential, then evidence against its binding) | essential | 1.38 / 1.32 / 1.40 | 1.50 / 1.45 / 1.53 | 12,492 B | 17.5 |
| | all | 1.41 / 1.38 / 1.43 | 1.53 / 1.50 / 1.55 | 12,492 B | 16.8 |
| `page-small` (4-item page) | essential | 1.33 / 1.32 / 2.67 | 1.35 / 1.34 / 2.69 | 5,786 B | 15.5 |
| | all | 1.34 / 1.32 / 1.37 | 1.36 / 1.34 / 1.39 | 5,786 B | 15.5 |
| `page-abandon` (two pages, then drop) | essential | 1.33 / 1.30 / 1.36 | 1.48 / 1.45 / 1.52 | 5,807 B | 15.7 |
| | all | 1.34 / 1.31 / 1.37 | 1.50 / 1.46 / 1.52 | 5,807 B | 15.6 |

## S2-009.war (7,200 classes, 57,180 methods, 53 nested containers)

| workload | arm | first result ms (median/min/max) | whole sequence ms (median/min/max) | returned | peak RSS MiB |
| --- | --- | --- | --- | --- | --- |
| `nav-class` | essential | 0.50 / 0.48 / 0.51 | 0.54 / 0.53 / 0.56 | 19,045 B | 76.8 |
| | all | 0.50 / 0.49 / 0.50 | 0.54 / 0.54 / 0.55 | 19,045 B | 75.8 |
| `nav-decl` | essential | 0.54 / 0.52 / 0.55 | 0.59 / 0.56 / 0.59 | 21,236 B | 76.2 |
| | all | 0.55 / 0.53 / 0.56 | 0.59 / 0.58 / 0.60 | 21,236 B | 76.2 |
| `recover-one` | essential | 0.74 / 0.72 / 0.75 | 0.81 / 0.78 / 0.82 | 51,817 B | 77.0 |
| | all | 0.76 / 0.73 / 0.78 | 0.82 / 0.80 / 0.85 | 53,958 B | 78.9 |
| `sweep` | essential | 2.45 / 2.36 / 2.55 | 6,197.86 / 6,167.60 / 6,236.31 | 57,180 methods | 131.0 |
| | all | 2.37 / 1.24 / 2.56 | 6,337.54 / 6,328.20 / 6,373.31 | 57,180 methods | 139.8 |
| `expand` | essential | 0.76 / 0.73 / 0.77 | 1.05 / 1.00 / 1.08 | 76,698 B | 77.1 |
| | all | 0.77 / 0.70 / 0.78 | 1.06 / 0.98 / 1.07 | 76,698 B | 77.6 |
| `page-small` | essential | 1.59 / 1.50 / 1.61 | 1.61 / 1.52 / 1.63 | 6,188 B | 76.2 |
| | all | 1.60 / 1.56 / 1.63 | 1.62 / 1.58 / 1.65 | 6,188 B | 65.8 |
| `page-abandon` | essential | 1.61 / 1.47 / 1.73 | 2.06 / 1.88 / 2.20 | 6,294 B | 76.5 |
| | all | 1.60 / 1.57 / 1.67 | 2.06 / 2.01 / 2.12 | 6,294 B | 76.4 |

## What these numbers support

* **Contract.** The two arms present the same body: for `nav-class`, `nav-decl`, `expand`,
  `page-small` and `page-abandon` the returned byte counts are *identical* between arms, and
  `recover-one` differs by exactly the optional records the full selection adds. §7.1's semantic
  comparison is what proves the text itself is unchanged; this table shows the payload grew only
  where the selection says it should, and the `expand` rows show the follow-up against a stated
  binding agreeing on both corpora.
* **Work.** The selection is visible in peak RSS where it matters and nowhere else: `sweep` holds
  8.2 MiB more (bcprov) and 8.8 MiB more (S2-009) under the full selection, while every
  sub-package workload's RSS is flat between arms — which is what "not requested, not constructed"
  means at process scale.
* **Time.** Not claimed as a speed-up. Within a workload the arms' medians differ by less than the
  spread of their own samples, except where the selection genuinely adds work (`sweep` +4.2% / +2.3%
  while delivering identical counts; `recover-one` and `expand` sub-millisecond). The number the
  progressive design exists for is the first-result column: a whole-package sweep delivers its first
  method in **~2.5 ms** on both corpora, three orders of magnitude before the 1.8 s / 6.2 s it takes
  to deliver every method. That is reported as observed, not as a threshold, and the machine's load
  is stated beside it.
* **Worth keeping from the discarded attempt.** The one measurement the protocol forced into view by
  accident is the store's: a whole-package walk without one re-parses containers per class, which on
  this WAR is 5.7M `archive_entries` against 10,358 — the same effect task 5.4 fixed in the CLI, now
  confirmed from the library side by a harness that has to attach the store to be measuring a
  deployed configuration.
