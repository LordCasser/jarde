# Multi-resource TWR fixture evidence

This directory freezes a self-contained two-resource TWR class and a deterministic runner. It does not edit production code or OpenSpec planning files. Replay from the repository root with:

```sh
openspec/evidence/java-syntax-2026-09-26/multi-resource-twr/replay.sh
python3 openspec/evidence/java-syntax-2026-09-26/multi-resource-twr/patch_negative.py
```

`replay.sh` compiles the handwritten [`MultiResourceTwr.java`](MultiResourceTwr.java) with `javac --release 8 -g:none` and with the current `javac -g:none`, writes the `run()` method's `javap -v -c -p` listing and exception rows, freezes class-family SHA-256 values, then runs five modes under `java -Xverify:all`. The compiler is OpenJDK javac 23.0.1. The release-8 class is major 52, 2,592 bytes; the current-JDK class is major 67, 2,908 bytes.

The handwritten source SHA-256 is `1f9b306d0681eee204367bdab6689969b4c80601acc6eb3a8c2bbeef0b97b36a`. The frozen main-class SHA-256 values are:

- `--release 8`: `c3beb622636f592d83b17cb67c66341bd6823fa482064b7ac23a800c87bdf19c`
- current JDK: `0d222d2e322db7a23342a07f0a0a24d2a139c6e2b7075481f3ee62ef9c77d43e`

Both variants have the same `run()` exception-table geometry (half-open ranges):

```text
[20,33) -> 43  Throwable  inner body/read protection
[44,48) -> 51  Throwable  inner close/addSuppressed protection
[10,37) -> 59  Throwable  outer main row; ends at outer normal close start
[43,59) -> 59  Throwable  exact companion protection for the inner handler span
[60,64) -> 67  Throwable  outer close/addSuppressed protection
```

Normal close calls are at BCI 33 (inner) and 37 (outer). The return is evaluated at BCI 29, saved at 32, then the post-close tail is `iload_2` at 41 and `ireturn` at 42. This gives the fixture the close-after-evaluation return shape called out in the design. The companion row exactly matches the inner handler span `[43,59)` and uses the outer row's type and target.

For the older single-row layout, [`Guarded-two-javap.txt`](Guarded-two-javap.txt) records the repository's existing Java 8 `Guarded.two`: inner main row `[12,15) -> 26`, inner handler span `[26,46)`, and outer main row `[6,46) -> 57` directly covers that span and ends at the outer normal close start. Root also checked `Guarded.three` in the same frozen class: its main rows are `[18,21) -> 32`, `[12,54) -> 65`, and `[6,85) -> 96`; each outer row directly covers the preceding handler span and ends at its own normal close start, without a companion row. The surrounding test sources are `tests/fixtures/p3-handlers/Guarded.java` and `Res.java`; the frozen class used here is `tests/fixtures/p3-handlers/v8/Guarded.class` (SHA-256 `db52b41d27a78dcf5bc88fca2879c0f69947949afcc40112b58799245fe9edcf`).

The original class produces these five result lines in [`release8/runtime.txt`](release8/runtime.txt): normal return `42`; body exception; inner close exception; outer close exception; and body exception with suppressed close exceptions ordered `inner, outer`. The same lines are produced by both frozen javac variants; `release8-vs-current.diff` records the identical comparison.

## Negative control

`patch_negative.py` copies only the release-8 main class and changes the outer row end from BCI 37 to BCI 33, an instruction boundary that puts the endpoint at the inner normal close start instead of the outer normal close start. It leaves code and `StackMapTable` untouched. The patched class passes all five `java -Xverify:all` launches; its `inner-close` output omits `close-outer`, showing why the malformed range cannot be accepted as a faithful TWR proof. Its patched class SHA-256 and exception table are in `patched-negative/`.

The available Jarde executable was `/tmp/jarde-cli-audit-baseline` (SHA-256 `60c045b8f94d363950525692f5ce9e5f5cde6f78134806d75801cd0cae89a573`). The replay command was:

```sh
/tmp/jarde-cli-audit-baseline class-source --input <class> --class <ClassName> --policy single-class --release 8 --format text
```

For the simplified fixture, the report records `jre_guard_handler_range` at BCI 37; the output quotes `run()` and does not present TWR. For the original Patrol class (SHA-256 `ec15a17a0cab131caccb17d34f4d5f40bb0a62dd7a8e97c4a9b206dfb3649ba7`), [`jarde-Patrol-summary.txt`](jarde-Patrol-summary.txt) records the same refusal at BCI 33, alongside uncovered exceptional blocks; [`jarde-Patrol-multiResource.txt`](jarde-Patrol-multiResource.txt) is the generated method excerpt. The proposal fixture has the post-close return tail at BCI 41 (`iload 5`) and BCI 43 (`ireturn`), as shown in the existing `openspec/evidence/java-syntax-2026-09-26/javap.txt`; this fixture independently exercises the analogous tail at BCI 41/42. The patched negative class is also rejected by this CLI at BCI 33 (`patched-negative/jarde-summary.txt`). This executable is the available pre-change Jarde snapshot, not a rebuild of source during this evidence task.

## JADX comparison

`jadx` on PATH is version 1.5.6. This command decompiles the release-8 class family into [`jadx-all/sources/defpackage/MultiResourceTwr.java`](jadx-all/sources/defpackage/MultiResourceTwr.java):

```sh
jadx --no-res -d <output> release8/MultiResourceTwr.class release8/MultiResourceTwr\$Probe.class release8/MultiResourceTwr\$BodyFailure.class release8/MultiResourceTwr\$CloseFailure.class
```

JADX emits nested `try`/`catch(Throwable)` cleanup with explicit `close()` and `addSuppressed()`, rather than a TWR header. Its generated class compiles with `javac --release 8` and passes `-Xverify:all`, but its `inner-close` and `outer-close` runs close the failing resource twice and suppress the repeated close exception. The original and JADX line-by-line outputs are in `release8/runtime.txt` and `jadx-runtime.txt`; `original-vs-jadx.diff` records the two close-failure differences. `original-vs-patched-negative.diff` records the verifier-valid malformed-row behavior change.

The version command was `jadx --version`, which printed `1.5.6`. The requested local JADX source checkout is `/Users/lordcasser/workspace/testzone/jadx` at revision `2fb1b16386941660fda07e9017285aec40fcb37f`. `./gradlew --offline :jadx-cli:run --args='--version'` could not configure because Gradle could not resolve `org.gradle.toolchains.foojay-resolver-convention:1.0.0`; therefore the actual decompilation used the installed 1.5.6 executable. No repository build outputs from JADX are included here.

## Construction-site ownership negatives

`patch_effectful_header.py` deterministically emits two additional verifier-valid variants from the frozen release-8 class. It also writes each variant's `javap` excerpt, SHA-256, and runtime trace; all five modes are launched with `java -Xverify:all` during generation.

`effectful-header-negative/MultiResourceTwr.class` inserts the existing `invokestatic maybeFailBody:()V` at BCI 9, after the outer constructor at BCI 6 and before its Store, which moves to BCI 12. The two original constructions remain verified `new@1` Sites at BCI 0 and 13. The normal path matches the original trace, while the `body` and `suppressed` modes now throw before the Store and leave the opened outer object unclosed. The class SHA-256 is `5ff72e86a8459f8d946ce5fce3bb5637bf5dd1ba591dce82943d43a332463e24`. The focused regression requires `jre_guard_resource_init` at Store BCI 12 and keeps the complete method quoted.

`constructor-identity-negative/MultiResourceTwr.class` keeps the full `new; dup; <init>` Site at BCI 1, passes its constructed Probe as the receiver of the existing `Probe.close()V` call at BCI 10, then stores a typed null at BCI 13. The following Site at BCI 14 remains independently proved. This is verifier-valid, but it changes runtime behavior because the resource Store does not consume the proved constructor result. Its SHA-256 is `e98342b8401de3eee76901ed83312f820fd880f5749f12d78e680095dc188301`. The regression requires `jre_guard_resource_init` at Store BCI 13 and keeps the whole method quoted.

The Rust regressions are `an_effect_between_a_verified_construction_and_its_store_refuses_the_header` and `a_store_cannot_claim_a_different_value_than_the_proved_constructor_result` in `tests/p3_multi_resource_twr_geometry.rs`. Both pass on the stable tree, alongside the original positive geometry and the Java 8 five-mode execution comparison.

## Root acceptance

On 2026-09-26 root independently reran `replay.sh` and `patch_negative.py`, compared the release-8 and current-JDK five-path runtime files byte for byte, and checked the frozen main-class SHA-256 values and the patched outer row. The replay passed. The wrong-end class remains verifier-valid but changes the `inner-close` effect trace; this is a negative semantic control, not an alternate valid TWR projection. `openspec validate recover-multi-resource-twr --strict` passed. Production recovery and Java output remain tasks 2.x–3.x.
