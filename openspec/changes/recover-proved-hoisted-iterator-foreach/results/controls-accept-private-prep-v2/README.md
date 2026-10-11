# Hoisted-iterator controls acceptance preparation

This is a read-only, independent acceptance candidate for the actual controls capture at `/private/tmp/jarde-hoisted-iterator-controls-baseline-root-v3`. It does not run Jarde, JADX, a JDK, Cargo, or Git. The candidate verifier is `accept-controls.py`; it checks the frozen receipt artifacts, raw stream bytes, current source closure, final build/CLI binding, manifest inputs, source/map parity, behavioral legs, compiler outcomes, and the checker’s reported physical coverage gaps. It prints a JSON summary and does not create an acceptance archive.

The pinned run recorded 43 commands and 86 raw stdout/stderr streams. Its 5 GiB minimum-free / 1 GiB maximum-target resource guard was configured, every command completed without guard stop, and per-command receipt values are checked. The launcher reported unchanged current source pins, unchanged frozen inputs, and target cleanup. The verifier recomputes the source closure from the pinned helper and compares it to both final-build pin maps and the capture’s tracked pins; the observed closure count is read from the receipts (4727 in this run).

The two original Java 8 compilations produced the same 15-line Runner output. The four JADX-derived complete-class runtime legs are recorded as differing from that original oracle. The four generated Positive isolated Jarde profiles compiled, produced exact class sets, and matched the original output. In those runs the frozen original Refusal class and original Runner were supplied beside the generated Positive class; this is evidence for Positive only, not for a generated Refusal runtime.

The eight generated combined/Refusal compile failures remain observations. No runtime is claimed for those legs. In each JDK and evidence profile, default and all produced equal source and maps for both classes. The checker validated 20 method maps and identified four physical-coverage gaps: `sumWithWrittenGap(Ljava/util/List;)I` for Refusal, once for each JDK/profile. Their exact missing BCI sets are emitted from the pinned checker receipt. These quality gaps are intentionally retained in the acceptance output.

The source-shape summary comes from the four generated Positive and four generated Refusal presentation files; the original `.class` files and pinned `javap` output are used to independently count physical method BCIs. Counts and compiler/runtime outcomes are derived from the current receipts at verifier time. The reported method presentation counts do not claim that a Jarde source artifact compiled unless its explicit compile observation passed.

## Inputs and pins

- Controls source and manifest: `/private/tmp/jarde-hoisted-iterator-controls-baseline-prep-luna-v2/inputs`, manifest SHA-256 `bf6edfcdcd33e192a02b225e81b4ea1045b9bd9ba4bb700980d81f837ffe3cbb`; frozen input-pin file SHA-256 `d47c6bb37722458709e524bdb3b03c332fea5e4b7297546790fb66bf22287334`.
- Capture receipt SHA-256 `e17a697a4790c7bb3b54128e241997e13a3d4f36a36ea0af057a3f4f16a0f37c`; checker receipt SHA-256 `11077bc4ed1b68bc841f0ca6a39b4201060ce34a6981912a750554e022207a4b`.
- Launcher receipt SHA-256 `35afb6103f8d772d6d296215b37c1eae5ed34ffe8d4881133d8b3d43d4f64dc8`.
- Final CLI SHA-256 `a71c31e80da7c6550aa572e5368f0ecad12ac862066c5d40535f5818c2263241`; final build receipt SHA-256 `90ecba1b52177d7dfae590679603e4624c4e56a42ae2b58d0616d7078c1f46e2`.
- Current source closure helper SHA-256 `ac511e1560c5da57fca5fcf4d37e837b726b4e20f2327014b9f29394870f0d1e`; checker helper SHA-256 `a05acce687bf0e22bfab9eb8ae00520d53a006b740732f4b0041182ce72a464a`.

The verifier has only been Python-AST-parsed during preparation. It has not been run against the capture; root should review and run it independently if desired. This is a controls-baseline acceptance, not a feature claim that all iterator-hoisting controls are recovered.

## v2 corrections and focused review

This v2 is a new candidate; v1 remains untouched. It corrects three bindings confirmed from the actual JSON schema: JADX runtime command labels end in `-run`; generated compile-observation labels omit the command's `-compile` suffix; and original compilation rows are named `javac8-original` / `javac23-original`, so the JDK name must be the prefix before the first hyphen. The physical class helper returns `(owner, methods)`, with each method carrying a zero-based `index`; class rows identify bytes via `classes[].path`.

Focused read-only assertions against the actual receipts confirmed exactly four JADX `-run` commands, eight failed Jarde observations each mapping to an existing `label + '-compile'` command, and four original target class sets with 3 Positive / 2 Refusal physical methods per JDK. For each physical method the candidate records its class-order method index and instruction BCI count, cross-checking the pinned Javap output.

The runtime mismatch is explicitly demonstrated by the same `written-gap-iterator-throws-before-write` line: original output is `result=0|exception=none|trace=values.iterator();`, while each of the four JADX legs has `result=1|exception=none|trace=values.iterator();`. This records the observed seen-field value change when `iterator()` throws before the protected write; it is not treated as an expected equivalence.

Review performed for v2: Python AST parse plus focused receipt/schema assertions only. The whole acceptor, toolchain, and CLI were not run. Once root reviews this frozen candidate and its final pins, do not modify it in place; any further change should use a new private version directory.
