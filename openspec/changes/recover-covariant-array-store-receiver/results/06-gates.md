# 06 · Gates (task 3.1)

Everything below is run in this worktree, on the final file state, at `HEAD` + the three commits of
this change (implementation, test/fixture, docs). Disk was checked before each build round; the
target directory was cleaned once (`cargo clean`, 39.9 GiB) when the free space fell below the
20 GiB floor, before the authoritative round.

## Formatting

```sh
cargo fmt --all -- --check
```

Ran after the last edit of every file (the new test file's long lines were the only formatting the
run found, and the check is re-run clean afterwards). Result: **clean** (exit 0, no diff).

## Clippy (the CI invocation, verbatim)

```sh
sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh
sh /tmp/ci-clippy.sh
```

Result: `cargo clippy --workspace --all-targets --all-features --locked` — see
[06-clippy.txt](06-clippy.txt) for the verbatim log: `EXIT=0`, `Finished` with **zero** `warning:`
lines (the `-D warnings` line of the CI step sits outside the extracted range, so the log is read
for warnings as well as the exit code).

## The workspace suite (authoritative totals)

```sh
cargo test --workspace --all-targets --all-features --locked --no-fail-fast
```

Authoritative reading: **exit code**, `grep -c "^test result: ok"`, and **zero** `test result:
FAILED` lines. See [08-workspace-tests.out](08-workspace-tests.out) for the full log and
[07-suite-tails.txt](07-suite-tails.txt) for the tail of each target.

| run | exit | `test result: ok` | `test result: FAILED` |
| --- | ---: | ---: | ---: |
| 1 (first full sweep, before the census/fingerprint bookkeeping) | 101 | 337 | 2 (both intended corpus bookkeeping: the new fixtures' census and fingerprint) |
| 2 (after the bookkeeping) | 101 | 338 | 1 — `d3_artifact_binding::the_evidence_is_rebuilt_after_every_temporary_of_the_first_request_is_dropped`, a **timing flake** (`elapsed_millis: 1` vs `0` in an otherwise identical `UsageSnapshot`) |
| 2-flake rerun | 0 | 1/1 | 0 — the same test alone, `--all-features`, twice: 2/2 green |
| **3 (authoritative)** | **0** | **339** | **0** |

The flake is the known `elapsed_millis` family: the assertion compares two `UsageSnapshot`s of two
requests of the same shape, and the machine was under heavy load from other worktrees during that
run. It touched no file of this change (the failing shape is a `d3` binding-cost comparison), and
both single-test reruns pass.

### The change's own suites

| suite | result |
| --- | --- |
| `recover_covariant_array_store_receiver` (not ignored) | 4 passed, 0 failed, 4 ignored |
| `recover_covariant_array_store_receiver -- --ignored` | 4 passed, 0 failed |
| `recover_array_element_field_receiver` (the read-side suite the change is symmetric with) | 10 passed, 0 failed, 3 ignored |
| `p5_corpus_fingerprint` (manifest, after regeneration) | 5 passed, 0 failed, 1 ignored |
| `jarde-reader --lib` (the fixture census) | 178 passed, 0 failed |

## The oracle leg (the corpus comparison)

```sh
cargo test --test p3_execution_comparison --all-features --locked -- --ignored
```

Result: see [05-oracle-leg.txt](05-oracle-leg.txt) — every corpus row replayed, no expectation
needed a change (the change moves no member the p3 corpus pins).

## Spec validation

```sh
openspec validate --all --strict
```

Result: **311 passed, 0 failed**.

## Corpus sweep, census and fingerprint

[05-corpus-sweep.md](05-corpus-sweep.md): three two-leg scans over the fixture corpus (907 captures
per leg, twice: single-class and family-jar) and the evidence corpus (2,725 captures per leg), every
delta classified — **zero** render deltas outside this change's own fixtures and the patrol's own
`as.jar`. Reader census `(899, 3867, 334, 2445, 8)` → `(907, 3911, 344, 2453, 8)`; fingerprint
manifest 1,666 → 1,678 files (+12, 0 removed, 0 changed).

## Commits (not pushed)

| commit | subject |
| --- | --- |
| `87b0a349` | `feat(java): widen a covariant array store's access receiver to Object[]` |
| `0f0db83a` | `test(covariant-store): freeze the patrol anchor, the driver and the controls on both legs` |
| _docs commit_ | the evidence of this directory plus the `tasks.md` record |

## Disk and `/tmp`

`df -h /` was read before every build round; the one moment the free space fell under the 20 GiB
floor it was restored with `cargo clean` (39.9 GiB) *before* the next full round, never during one.
The scratch the evidence was produced from was measured and then removed before this report:

| directory | occupancy before removal |
| --- | ---: |
| `/tmp/asis` (baseline/change binaries, both legs' renders) | 127 MiB |
| `/tmp/fx` (fixture sources, jars, generated constants, manifest diff input) | 66 MiB |
| `/tmp/gate2` (the gating scratch family) | 3.1 MiB |
| `/tmp/jarde-scan-single` (single-class capture trees) | 206 MiB |
| `/tmp/jarde-scan-family` (family-jar capture trees) | 276 MiB |
| `/tmp/jarde-scan-evidence` (evidence capture trees) | 560 MiB |
| **total removed** | **~1.24 GiB** |

Final free space: 30 GiB; the workspace `target/` holds the built test artifacts of the
authoritative run.
