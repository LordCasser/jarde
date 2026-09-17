# Bounded P1 fuzz workspace

Standalone, test-only [`cargo-fuzz`](https://rust-fuzz.github.io/book/cargo-fuzz.html)
workspace for the P1 query and artifact-tree / multi-release surfaces (design 3.3). It is
deliberately **not** a member of the main workspace: `Cargo.toml` declares its own
`[workspace]` table, so the fuzzing toolchain never enters the production dependency tree
and never touches the library's MSRV 1.88.0 surface. `cargo metadata` at the repository
root still reports exactly the members `jarde` and `jarde-cli`, and
`cargo tree --workspace -e normal` contains none of `libfuzzer-sys`, `cc`, `arbitrary`,
`jobserver` or `shlex`.

## Layout

| path | what it is |
| --- | --- |
| `Cargo.toml` | own workspace root; pins `libfuzzer-sys = "=0.4.13"` and `jarde = { path = "..", version = "=0.1.0" }` |
| `Cargo.lock` | committed, so the tools and the runtime library stay pinned |
| `rust-toolchain.toml` | pins the fuzz workspace to `nightly-2026-07-20` (sanitizer and coverage instrumentation are nightly-only) |
| `src/lib.rs` | fixed requests, bounded entry points, the public-contract assertions, the corpus checks |
| `fuzz_targets/query.rs` | target 1: input → artifact → all five fixed query requests |
| `fuzz_targets/artifact_tree.rs` | target 2: input → artifact → nested tree enumeration + multi-release selection |
| `corpus/generate_seeds.py` | deterministic generator for the committed seeds |
| `corpus/<target>/` | the committed minimal seeds (digests below) |

## What the targets do

Both targets treat the whole fuzz input as the artifact and use only public entry points
(`Engine::open`, `Engine::query`, `Engine::enumerate_artifact_tree`,
`Engine::select_multi_release`) under the hard limits in `jarde_fuzz::limits()`. There is
no hand-written reader, no hand-written mutation engine and no class generator in the
targets.

| target | input → handling | assertions |
| --- | --- | --- |
| `query` | input bytes are the artifact; a rejected input is an accepted outcome; an opened snapshot gets all five fixed requests sequentially, independent of the file magic (`mentions_symbol` on `com/example/Fuzz.target:()V`, on the class, `literal_value` on `fuzz-literal`, the raw `constant_pool_contains` probe, and a request naming every consumer category) | `page.returned_items == items.len()`; published items ≤ `limits.result_items`; a cursor implies `has_more`; `CompleteWithinSchema` ⇒ `ExecutionReport::Complete` + no unsupported category + no skipped range; any other execution ⇒ `Partial`; `unsupported_categories` is exactly the declared-but-unimplemented kinds; the two other coverage dimensions stay `NotRequested`; `unknown_candidates ≤ scanned_items`; every item answers the requested relation and claims no resolution; every `usage` field stays inside the limit that produced it |
| `artifact_tree` | input bytes are the artifact; an opened snapshot is enumerated as a nested tree and then run through standard multi-release selection for a fixed Java 17 class-path runtime view, each with its own budget | reported containers + layout nodes ≤ `limits.result_items` (2 reservations + 1 per returned entry evidence + 1 per selection in the selection call); the same completeness/execution coupling on the artifact-structural and runtime-resolution dimensions; `verification == NotPerformed`; every `usage` field stays inside its limit |

The shared `exercise_query` driver opens once, then checks and drops each report before
starting the next request. A public query error does not skip the remaining shapes. Each
operation has its own `limits()` budget: at most one open plus five queries, with at most
five times each query limit in cumulative query work (including five cooperative deadlines).
Only one query report is retained at a time. This is a bounded multiplier, not a single
shared-budget request or a hard wall-clock timeout.

The corpus tests run this same driver and require the actual visited shapes to be
`[0, 1, 2, 3, 4]` for both valid CLASS and JAR seeds and for opened damaged seeds. Valid
seeds must produce nonempty answers; damaged reports must retain their failure/Partial
state, and the all-category request must report Verification/Debug as unsupported. The old
first-byte selector was faulty: CLASS magic `0xCA` always selected shape 2, and ordinary
PK ZIP magic `0x50` selected shape 0. Restoring it makes the new routing regression fail.

The assertion set intentionally only states documented behaviour: a libFuzzer finding must
be a real contract violation, not a legitimate outcome (an empty page, a damaged artifact,
the documented `has_more`-without-cursor continuation seam) tripping a check. The checks
are not vacuous: `cargo test` in this workspace injects a page that overspends its result
budget and a report that claims completeness over a stopped scan and requires the check to
fail on both (`the_query_check_rejects_*`).

## Seeds

`corpus/generate_seeds.py` writes every seed deterministically (fixed ZIP timestamps, no
randomness and no network); running it again reproduces the same bytes and prints the
digests below. The class it writes is `com/example/Seed`, whose `probe()V` body mentions
all three fixed `query` targets, so a valid seed answers a request without any mutation.

| seed | bytes | SHA-256 |
| --- | --- | --- |
| `query/minimal-class` | 185 | `7f4c40a79be7805f554253b4abe33509e9751bfb8ef3b4e1acdbfa5f3d994d83` |
| `query/minimal-jar` | 659 | `e40e8212e4bb0d80f7f69b55ed9ba7797aa806309e54509a5de4ebd2cefce456` |
| `query/damaged-candidate.jar` | 498 | `1c2eb1f019396688fb7dc1d8db0461cc5729514368cebad814b61ea889e8b494` |
| `query/corrupt-payload.jar` | 659 | `f6024656e338ddf57d880bf3f37daa4702f83abc4209243dfc88ac1d9ec5fe6c` |
| `query/truncated-class` | 20 | `6fe24a9d34ae7f04938f1f36fd54216f69964393367b3f52f87a5ff547f61dfa` |
| `artifact_tree/minimal-jar` | 659 | `e40e8212e4bb0d80f7f69b55ed9ba7797aa806309e54509a5de4ebd2cefce456` |
| `artifact_tree/multi-release.jar` | 866 | `e62f90dd135b10a59b3816626841aa1f92079db967f3881e0e749248e45e633d` |
| `artifact_tree/nested.jar` | 783 | `7f2120bde1e7bf305528ce12626dbe5d119598aca5f14c5906abe6f543e3e6b4` |
| `artifact_tree/damaged-entry.jar` | 498 | `1c2eb1f019396688fb7dc1d8db0461cc5729514368cebad814b61ea889e8b494` |
| `artifact_tree/truncated-jar` | 80 | `8e677c19609a57aae4acca413b669f04af926a48c281f2db0434d7fdc1bca833` |

What each damaged seed is meant to reach (`cargo test` in this workspace asserts it): the
ZIP of `damaged-candidate.jar`/`damaged-entry.jar` is intact and its CRC is consistent, so
the *class candidate* rule rejects it (`query_class_candidate_malformed`);
`corrupt-payload.jar` keeps the ZIP intact but breaks the stored entry's CRC, so the
materialization path rejects it (`entry_integrity`); `truncated-class` is a standalone
CLASS cut inside its header; `truncated-jar` loses the ZIP end record and is rejected when
it is opened.

New inputs the fuzzer generates are **not** committed and are not ignored: start a run
against a scratch copy of `corpus/<target>/` (the commands below do), so the tracked seeds
stay byte-identical.

## Pinned tools

| tool | version |
| --- | --- |
| cargo-fuzz | 0.13.2 (`cargo install cargo-fuzz --version 0.13.2 --locked`) |
| libfuzzer-sys | 0.4.13 (`=0.4.13` in `Cargo.toml`, committed in `Cargo.lock`) |
| libFuzzer runtime | the compiler-rt sources vendored by libfuzzer-sys 0.4.13, compiled by the `cc` crate with the local C++ compiler (Apple clang 21.0.0.0 locally, the runner's clang in CI); the crate publishes no LLVM revision string |
| toolchain | `nightly-2026-07-20` = rustc 1.99.0-nightly (9f36de775 2026-07-19), cargo 1.99.0-nightly (3efb1f477 2026-07-17) |

Local prerequisite: `cargo-fuzz` spawns a plain `cargo` for the actual build, so that
`cargo` must be the rustup shim (the one that honours `rust-toolchain.toml`). If `cargo`
resolves to a non-rustup installation, prepend the rustup bin directory:

```sh
cd fuzz
PATH="$HOME/.cargo/bin:$PATH" cargo fuzz run query /tmp/jarde-fuzz/query -- \
  -max_total_time=60 -max_len=65536 -rss_limit_mb=512 -workers=1 -print_final_stats=1
```

## Historical P1 smoke runs (before the routing fix, macOS aarch64, AddressSanitizer)

These historical runs used the old one-request selector. They do not validate the revised
five-request driver; see the harden-p1-validation verification record for the new runs.
Each run started from a scratch copy of the five committed seeds and ran one worker:

```sh
cd fuzz
PATH="$HOME/.cargo/bin:$PATH" cargo fuzz run query          /tmp/jarde-fuzz/query --          -max_total_time=60 -max_len=65536 -rss_limit_mb=512 -workers=1 -print_final_stats=1
PATH="$HOME/.cargo/bin:$PATH" cargo fuzz run artifact_tree  /tmp/jarde-fuzz/artifact_tree --  -max_total_time=60 -max_len=65536 -rss_limit_mb=512 -workers=1 -print_final_stats=1
```

| target | runs | exec/s | coverage | features | peak RSS | result |
| --- | --- | --- | --- | --- | --- | --- |
| `query` (seed 323430476) | 1 646 123 | 26 985 | 4 425 | 10 123 | 173 MB | exit 0, no crash, no contract violation |
| `query` (seed 584977148, repeat) | 1 650 577 | 27 058 | 4 187 | 9 443 | 181 MB | exit 0, reproducible gate outcome |
| `artifact_tree` (seed 384695688) | 1 278 312 | 20 955 | 2 763 | 5 889 | 456 MB | exit 0, no crash, no contract violation |

Stable replay: `cargo fuzz run <target> corpus/<target>/minimal-jar -- -runs=1` executed the
same input twice per target with the same outcome (exit 0, "Executed … in 5 ms", one
executed unit, no new corpus unit). No crash, hang or contract violation was found, so
there is no minimized counterexample to record.

The historical no-AddressSanitizer comparison executed 4 719 875 units in 61 seconds
with 30 MB peak RSS. It shows that instrumentation materially affects this measurement;
it does not prove that every engine allocation is bounded or exclude leaks. Keep the
512 MB gate and record platform, sanitizer, corpus and duration before comparing peaks.

These runs are a **bounded smoke**, not a security or coverage proof: one worker, 60
seconds, two targets, fixed requests and small budgets.

## License and advisory check

`deny.toml` lives at the repository root and is the only policy source; the fuzz workspace
does not add a second one.

Run both dependency graphs explicitly from the repository root:

```sh
cargo deny --manifest-path Cargo.toml --workspace --locked --config deny.toml check
cargo deny --manifest-path fuzz/Cargo.toml --workspace --locked --config deny.toml check
```

The root graph contains the library and CLI; the independent fuzz graph contains
`libfuzzer-sys`. Both must pass advisories, licenses, bans and sources. The single root
policy grants NCSA only to the exact `libfuzzer-sys 0.4.13` exception, not to arbitrary
future dependencies. This crate declares `(MIT OR Apache-2.0) AND NCSA`: the NCSA term
covers its bundled libFuzzer runtime. Upgrading it requires reviewing the exception.


## CI

`.github/workflows/ci.yml` adds the `fuzz-smoke` job: install pinned nightly-2026-07-20 and
cargo-fuzz 0.13.2, replay the committed corpus once through `cargo test` (the seed checks
above), then run both targets for 20 seconds each with
`-max_len=65536 -rss_limit_mb=512 -timeout=10 -workers=1` against scratch corpus copies,
and finally require `git diff --exit-code` plus an empty `git status --porcelain` so the
run cannot leave a tracked file changed. It proves a bounded smoke passes; it is not a
security proof and no corpus accumulates between runs.

The `supply-chain` job checks both manifests explicitly against the root policy. A green
root audit alone cannot cover this independent fuzz workspace. Local command results and
remote CI run results are recorded separately in
[`harden-p1-validation`](../openspec/changes/harden-p1-validation/verification.md).
