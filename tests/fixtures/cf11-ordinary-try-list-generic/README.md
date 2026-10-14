# Ordinary try List fixtures

This directory keeps the accepted 45-row full-class baseline separate from the new focused
admission fixture. The baseline input is `baseline/UpstreamNestedLoops.java` plus
`baseline/Runner.java`; its manifest pins both files. The baseline source SHA is
`edc6d5ab7258ad79eb13cc0163c042dcdc885dd3d3d160829a098a8b5d98a1ed`, and the Runner SHA is
`d810c2466b31edc6d9df9b7286dedc21ca77428fa329f86d98d8a36d6f7dee7d`.

The parent-level `UpstreamNestedLoops.class` is an accepted, frozen JDK 8 original from
`openspec/changes/recover-ordinary-try-list-generic-bodies/results/baseline-45-supplement-product-597010-root-v1/capture.tar.gz`, member
`captures/debug/legs/javac8-original/classes/cf11/UpstreamNestedLoops.class`. Its SHA-256 is
`07cd9a4a18eba761dc86e8a613880ed4da42597edcc35b09fe161167b3579aba` (6,887 bytes); the archive
SHA-256 is `9a4705d119ee27e1857a5571376bcbf9fe1850bcc6b641548b35203b173a9166`.

Keep that accepted class byte-for-byte. Replay runs compile pinned Java sources into each fresh
output directory and do not write over this frozen fixture. Run `tools/replay_baseline45.sh` with
`BASELINE_DEBUG_INFO=debug` and `none` in separate fresh output directories to collect the full
JDK 8/23 and Jarde default/all matrix. The independent `admission/` source, Runner, and manifest
exercise the renamed try, direct catch uses, exact String/Integer types, and negative controls.
