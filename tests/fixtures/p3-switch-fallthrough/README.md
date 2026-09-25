# Ordinary switch fallthrough evidence

These are the frozen `lookupswitch` and `tableswitch` inputs from
`openspec/evidence/java-syntax-2026-09-22/switch-fallthrough-order/`. Their exact class bytes are
reused here so the integration test exercises the same javac 23.0.1, `--release 8 -g:none`
fixtures as the source/JADX/Jarde runtime comparison. SHA-256:

* `v8/SwitchFallthroughOrder.class`: `511fa56a325aeb1c2c7781e3fb0c86b6640cc4c3436df7cbb5c05ce0f5943ad0`
* `v8/TableswitchFallthrough.class`: `f45d78320286e1912b30f4b01524b17695a1cef269a0204b18d3d7191ce16f84`
* `v8/DefaultMiddle.class`: `8a137f3884bf366a8a962c4bb2efb460b882aae2cd6198c459ff8f6de434add8`

The source, compiler logs, `javap` output, original/JADX/Jarde sources and audit runners are kept
with that permanent evidence bundle.

`DefaultMiddle` is an additional Java 8 source/class pair compiled with the same flags. Its
`default` label sits between two case labels, and both preceding arms fall through. The runner
comparison and expected values are recorded in the 2026-09-24 acceptance note.
