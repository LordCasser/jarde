# Bounded P1 fuzz workspace

Standalone, test-only [`cargo-fuzz`](https://rust-fuzz.github.io/book/cargo-fuzz.html)
workspace for the P1 query and artifact-tree / multi-release surfaces (design 3.3) and the
P2 bounded method-analysis entry (design 5.3). It is deliberately **not** a member of the
main workspace: `Cargo.toml` declares its own
`[workspace]` table, so the fuzzing toolchain never enters the production dependency tree
and never touches the library's MSRV 1.88.0 surface. `cargo metadata` at the repository
root reports `jarde`, `jarde-cli`, `jarde-reader`, `jarde-query`, and `jarde-jvm`; the
fuzz workspace remains separate, and
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
| `fuzz_targets/method_analysis.rs` | target 3: input → artifact → method derived from the input → all nine fixed stage requests through `Engine::analyze_method` |
| `corpus/generate_seeds.py` | deterministic generator for the committed seeds |
| `corpus/generate_method_analysis_seeds.py` | deterministic generator for the committed P2 method-analysis seeds |
| `corpus/<target>/` | the committed minimal seeds (digests below) |

## What the targets do

All three targets treat the whole fuzz input as the artifact and use only public entry
points (`Engine::open`, `Engine::query`, `Engine::enumerate_artifact_tree`,
`Engine::select_multi_release`, plus `Engine::inspect_header`, `Engine::enumerate` and
`Engine::analyze_method` for the analysis target) under the hard limits in
`jarde_fuzz::limits()`. There is
no hand-written reader, no hand-written mutation engine and no class generator in the
targets.

| target | input → handling | assertions |
| --- | --- | --- |
| `query` | input bytes are the artifact; a rejected input is an accepted outcome; an opened snapshot gets all five fixed requests sequentially, independent of the file magic (`mentions_symbol` on `com/example/Fuzz.target:()V`, on the class, `literal_value` on `fuzz-literal`, the raw `constant_pool_contains` probe, and a request naming every consumer category) | `page.returned_items == items.len()`; published items ≤ `limits.result_items`; a cursor implies `has_more`; `CompleteWithinSchema` ⇒ `ExecutionReport::Complete` + no unsupported category + no skipped range; any other execution ⇒ `Partial`; `unsupported_categories` is exactly the declared-but-unimplemented kinds; the two other coverage dimensions stay `NotRequested`; `unknown_candidates ≤ scanned_items`; every item answers the requested relation and claims no resolution; every counted/depth `usage` field stays inside its corresponding limit (`elapsed_millis` is observed cooperatively and is excluded from this assertion) |
| `artifact_tree` | input bytes are the artifact; an opened snapshot is enumerated as a nested tree and then run through standard multi-release selection for a fixed Java 17 class-path runtime view, each with its own budget | reported containers + layout nodes ≤ `limits.result_items` (2 reservations + 1 per returned entry evidence + 1 per selection in the selection call); the same completeness/execution coupling on the artifact-structural and runtime-resolution dimensions; `verification == NotPerformed`; every counted/depth `usage` field stays inside its limit (`elapsed_millis` excluded) |
| `method_analysis` | input bytes are the artifact and a rejected input is an accepted outcome; the method to analyze is derived from the input (`Engine::inspect_header` on a standalone root, or on the first few `Base` `.class` entries of a ZIP, at most 8 header probes, taking the first member that declares a `Code` attribute) because the request cannot be a fixed one; an artifact with no such member ends the run without a request; the derived method is then driven through nine fixed shapes — stage prefixes from `RawFacts` to the whole pipeline, one request whose stages are named out of order and twice (normalized to the fixed phase order), the starved budgets that must stop after the read, and the clone-starved budget whose `Partial` canonical phase and `Fallback` quality the public planes cannot tell apart from a published graph of a decoded prefix | the report answers about the method the request named and echoes the requested phases in fixed phase order without duplicates; the scheduled phases are the phase prefix that ends at the last requested phase and no phase behind a stop is reported as reached; `execution` explains the stop and every stop is named by a diagnostic of its own code; a `Complete` execution carries no error diagnostic; `quality == Conservative` requires a published canonical graph and `Fallback` never claims one; only a `Completed` `Ssa` phase raises `semantic_validation` to `LocalInvariants`; `representation == Bytecode`, `syntax_status == NotJava`, `compile_status == NotAttempted` and `verification == NotPerformed` always; the caller domain's own loader is echoed; the reads are the request's own class definition only; every counted/depth `usage` field stays inside its limit (`elapsed_millis` excluded) |

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
the documented `has_more`-without-cursor continuation seam, a derived method whose body the
budget stops) tripping a check. The checks
are not vacuous: `cargo test` in this workspace injects a page that overspends its result
budget and a report that claims completeness over a stopped scan and requires the check to
fail on both (`the_query_check_rejects_*`), and for the analysis contract it injects a
report with a phase behind a stop, `Conservative` quality without a published canonical
graph, a stop without a diagnostic, and a schedule that ends before the phase the request
asked for, requiring the check to fail on each (`the_analysis_check_rejects_*`).

## Seeds

`corpus/generate_seeds.py` writes every seed deterministically (fixed ZIP timestamps, no
randomness and no network); running it again reproduces the same bytes and prints the
digests below. The class it writes is `com/example/Seed`, whose `probe()V` body mentions
all three fixed `query` targets, so a valid seed answers a request without any mutation.

`corpus/generate_method_analysis_seeds.py` does the same for target 3 and prints the
digests below. Its seeds exist because the analysis target derives the method it analyzes
from the input, so the *first member with a body* has to be the shape the corpus is meant
to cover: `jsr-ret.class` copies the committed ECJ 4.6.1 / 45 dialect fixture byte for
byte (its `finallyPath(I)I` really compiles `finally` into two `jsr` calls of one
subroutine, and the corpus test asks for that method explicitly), `legacy-clone.class`
writes one shared subroutine called from two sites, `wide-switch.class` puts
`lookupswitch`, `tableswitch` and `wide` local access in one body,
`exception-overlap.class` and `exception-overlap-mixed.class` carry three overlapping
exception records (the second has a typed and a catch-all record enter one handler), and
`wide-switch.jar` is the same wide/switch body as a `Base` entry of a ZIP, which is the
derivation path a standalone class root does not reach. Nothing here drives the fuzzer:
cargo-fuzz mutates these seeds itself, and the smoke gate never needs the scripts.

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
| `method_analysis/jsr-ret.class` | 258 | `ecd9c7cca0cd3c145be129e3342d4af6e985a2872091cc2b848e557370f5438d` |
| `method_analysis/legacy-clone.class` | 131 | `1e63109fc6c3ad646549855cd88d7bf8278ec324970250db77c304c07e2b5fa4` |
| `method_analysis/wide-switch.class` | 194 | `5b834a4130548c422992f87cf735335cef12d3a66a4dde65b12a3fa6e757c2bb` |
| `method_analysis/exception-overlap.class` | 214 | `dec4deae4816fa089e9e3e074f2a77bf83ecacd9766e4b50f7a0227519c1ced2` |
| `method_analysis/exception-overlap-mixed.class` | 189 | `ce8e93b5ee2c6df3d1f5e1d5b8f42760e878d3dac4613823032a659324025eb7` |
| `method_analysis/wide-switch.jar` | 334 | `574c4c18db87f057aed95d83ab46905672f15a5201d99fdc4ae55358949af41c` |

What each damaged seed is meant to reach (`cargo test` in this workspace asserts it): the
ZIP of `damaged-candidate.jar`/`damaged-entry.jar` is intact and its CRC is consistent, so
the *class candidate* rule rejects it (`query_class_candidate_malformed`);
`corrupt-payload.jar` keeps the ZIP intact but breaks the stored entry's CRC, so the
materialization path rejects it (`entry_integrity`); `truncated-class` is a standalone
CLASS cut inside its header; `truncated-jar` loses the ZIP end record and is rejected when
it is opened.

The corpus tests also pin what the analysis seeds are worth, so a seed that stops reaching
the pipeline is caught rather than merely surviving a smoke run: every seed's recorded
size, the method the driver derives from it (`<init>()V` for `jsr-ret`, `shared()V` for
`legacy-clone`, `pick(I)I` for both wide/switch seeds and `guarded(I)I` for both
exception seeds), all nine shapes visited in order, at least one shape really stopped by
the minimal budget and at least three really completed under the ample one. The
`jsr-ret`/`legacy-clone` seeds additionally assert `usage.normalization_clones > 0` with
all six phases `Completed`, `Conservative` quality and `LocalInvariants`; the shared
subroutine charges no clone when the request stops before the canonicalization.

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
PATH="$HOME/.cargo/bin:$PATH" cargo fuzz run method_analysis /tmp/jarde-fuzz/method_analysis -- \
  -max_total_time=60 -max_len=65536 -rss_limit_mb=512 -workers=1 -print_final_stats=1
```

Point the run at a scratch copy of the matching `corpus/<target>/` directory, as above:
the tracked seeds must stay byte-identical.

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
above), then run all three targets (`query`, `artifact_tree`, `method_analysis`) for 20
seconds each with
`-max_len=65536 -rss_limit_mb=512 -timeout=10 -workers=1` against scratch corpus copies,
and finally require `git diff --exit-code` plus an empty `git status --porcelain` so the
run cannot leave a tracked file changed. It proves a bounded smoke passes; it is not a
security proof and no corpus accumulates between runs.

The `supply-chain` job checks both manifests explicitly against the root policy. A green
root audit alone cannot cover this independent fuzz workspace. Local command results and
remote CI run results are recorded separately in
[`harden-p1-validation`](../openspec/changes/archive/2026-09-17-harden-p1-validation/verification.md);
the P2 target's local 60-second run and its first CI smoke step are recorded in
[`p2-jvm-ir`](../openspec/changes/p2-jvm-ir/verification.md).
