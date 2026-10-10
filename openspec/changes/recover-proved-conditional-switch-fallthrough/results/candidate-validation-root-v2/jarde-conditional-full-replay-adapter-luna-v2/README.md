# CF12 conditional replay adapter draft

This private draft adapts the type-slice complete-class collector and independent verifier for `recover-proved-conditional-switch-fallthrough`. It is not run and is not copied into the repository; root owns toolchain execution and final review.

Files:

- `prepare-conditional-replay.py`: collector based on `prepare-local-source-types-replay-root-v2.py`.
- `verify-conditional-replay.py`: independent verifier based on `verify-local-source-types-replay-root-v2.py`.

The accepted type-slice comparator is the repository artifact `openspec/changes/recover-proved-local-source-types/results/complete-source-root-v4/{execution,acceptance}.json`. Actual hashes are execution `27afad91dc4e5c426d90f6ff4253b4244541490161752b759d1f5ada19847ed5` and acceptance `d9e336b8b6eb33b76c0d866eb83040540586eba9df5ba629311f2371f577812f`. Its execution has 99 commands, six cases, and eight class instances; its acceptance status is `verified-typed-replay-observations-only` with `cf12_complete:false`. No `acceptance-root-v2.json` exists at that location. The adapter intentionally does not point to the post-pop artifacts.

The only new conditional anchor is `TestSwitchWithFallThroughCase.test`, with exact method key `test(IZZ)Ljava/lang/String;`. It is replayed on JDK 8 and JDK 23. The two previously typed anchors (`TestSwitch.test`, `TestSwitchNoDefault.test`) remain byte/body/map checked against the accepted typed baseline and retain their existing dual-JDK replay. The other three upstream rows remain JDK-23 controls. The six upstream case rows keep all eight original class files, including `$Inner`; default and all profiles render and compile the full class set.

The type-specific standalone `LocalSourceTypesBoundaries` replay and its four extra JDK/profile legs were removed. The original upstream 429-input/451-evidence closure, historical root-v6 observations, JDK manifest and tool pins, product runtime jars, official six test-SDK jars, helper leak checks, guard provenance, raw stdout/stderr hashes, original/JADX/default/all runtime comparisons, full generated class inventories, source-map owner identity, and exact physical BCI-set check are retained.

Candidate CLI, metadata, validation build, source-base, JDKs, and JADX remain explicit pinned arguments; no candidate CLI/meta/build values are guessed. The source-base supplied at invocation must be `2d70da515896c25ce022b8c28f4935ff2e105026`. The failure path keeps captured raw streams and the independent verifier hashes every recorded stream before classifying outcomes, including the historical compile-failure controls.

Before root executes these drafts, review should resolve two items: (1) adapt the collector's current freeze metadata schema checks if the conditional candidate runner uses a new but equivalent schema; (2) confirm the conditional anchor's reported method identity is exactly `test(IZZ)Ljava/lang/String;` from the frozen input class. The descriptor `(IZZ)Ljava/lang/String;` is already confirmed by the source and reader analysis recorded in this task; no `javap` run is needed here. No toolchain or draft script was executed.


The v2 validation runner is `run-validation-build-root-v2.py`. It keeps the 17 product-source pins, adds the conditional positive/boundary/scope integration sources plus seven existing switch/string/loop targets, and extends the `include_bytes!`/`include_str!` closure across all pinned product and test Rust sources so the `region.rs`/`build.rs` unit fixture literals are pinned. It statically selects the two CF12 switch unit tests, all 9 typed tests, 3 positive/recovery tests, 2 refusal tests, and 2 scope tests. `--expected-lib-count` is caller-supplied and is checked against the actual lib test summary; it is not hard-coded to the prior 337 count. The runner emits the conditional schema and the requested CLI/meta paths, and uses the pinned v9 guard.

The collector pins the requested source base exactly, verifies the JDK homes against the manifest before setting `JAVA_HOME`, prepended `PATH`, `LC_ALL=C`, and `TZ=UTC`, and requires JADX version stdout `1.5.6`. The independent verifier expects the conditional validation/candidate schemas and checks exact runtime argv, `-Xverify:all`, classpath layout, and raw stdout/stderr for each anchor and control. The Reader-verified conditional descriptor is `test(IZZ)Ljava/lang/String;`.
