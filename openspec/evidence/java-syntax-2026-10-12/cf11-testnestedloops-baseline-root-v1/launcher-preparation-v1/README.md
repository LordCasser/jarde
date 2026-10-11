# TestNestedLoops root baseline launcher prep

This package prepares a one-shot root launcher for the frozen `TestNestedLoops` input. It has not run the collector, checker, Java/JADX tools, or the CLI. The shared repository is unchanged by this preparation.

The input bundle is frozen at `/private/tmp/jarde-cf11-testnestedloops-inputs-root-v1`. Its two compilation inputs, `UpstreamNestedLoops.java` and `Runner.java`, and `manifest.json` are mode `0444`; provenance also retains the original source, pins, README, and exact upstream method slice, all mode `0444`. The upstream `TestCls.test(List<String>, List<String>)` slice at byte offsets `[279, 573)` is 294 bytes, matches its pin, and appears byte-for-byte in the wrapper. The wrapper and Runner hashes match the v3 input pins.

The launcher reuses `source_paths()` and `pin_sources()` from `/private/tmp/jarde-for-condition-next-round-luna-v1/jarde-for-condition-ci-verifier-luna-v4.py`, pinned by SHA-256 in the launcher. That helper covers Cargo manifests, Rust source/test files, fixtures, frozen evidence, and in-repository `include_bytes!`/`include_str!` dependencies. Its current file set matches the existing 4718-entry full-workspace pin inventory. The launcher requires exactly 4718 entries before and after the run, compares the full SHA maps, and records both maps. It also requires `target/` absent before starting and records whether it remains absent after both commands.

The collector is `/private/tmp/jarde-for-condition-collector-root-v3/collect_controls.py` (SHA-256 `f944a892f4920feaff9631822bddf4dd9e49a0e8349ec4e09e4a16046257a7ac`). The checker is `/private/tmp/jarde-for-condition-checker-root-v5/check_controls.py` (SHA-256 `d338d6ae9b4bec67f9b2fa76db0f71dc04ea7d75c82f7155805bb8ba6975c51b`). Collector environment/tool pins come from `/private/tmp/jarde-cf11-testnestedloops2-baseline-root-v1/execution.json`; only input, output, CLI, and build-receipt bindings are changed. The CLI and build receipt are pinned to the final v3 artifacts and validated against each other before the collector starts.

To run the prepared one-shot sequence, root can invoke:

```sh
python3 /private/tmp/jarde-cf11-testnestedloops-baseline-launch-prep-luna-v1/launch_baseline.py
```

The launcher runs the collector first and then invokes checker root-v5 if the collector leaves `execution.json`, even when the collector exits nonzero, so a partial-run checker failure remains visible alongside the collector failure. It writes a launcher receipt to `/private/tmp/jarde-cf11-testnestedloops-baseline-root-v1-launcher-receipt.json`; collector output goes to `/private/tmp/jarde-cf11-testnestedloops-baseline-root-v1`, with checker output at `checker-root-v5.json`. Existing outputs cause refusal to overwrite. This preparation makes no claim about the eventual collector or checker result.

`pins.json` records the source, tool, config, and frozen-input hashes reviewed here. No compilation, runtime oracle, JADX decompilation, collector invocation, checker invocation, or Git command was performed during preparation.
