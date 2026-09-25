# Minimized narrow B/C/S field-store evidence

This directory is the replayable input-stage subset for `recover-narrow-field-stores/task1.1`.
The source class declares six target fields as `int`; `patch_field_stores.py` appends one UTF8
descriptor for each B/C/S target, changes only the matching `field_info` descriptor and
`Fieldref` `NameAndType` descriptor, and checks every method `Code` byte sequence before writing
the patched class. The three naturally typed ordinary fields remain controls.

Replay from the repository root:

```sh
python3 openspec/evidence/java-syntax-2026-09-22/numeric-conversions/narrow-field-stores/p3-core/run_audit.py
```

`run_audit.py` creates a temporary build tree and replays the complete sequence: source
`javac`, the exact class-file patch, `javap`, original and patched `java -Xverify:all`, the
fixed jarde CLI, whole-class jarde `javac`/runtime, and JADX with a whole-class compile
attempt. It writes the phase commands, statuses, outputs, hashes, patch report, and
`summary.json` here; it does not invoke Cargo.

The source class is 1502 bytes with SHA-256
`94cbe660367772ce2bf2debc860a9c7ad4607e9b940f0176b6d8a24e8471f8cb`; the patched class is
1526 bytes with SHA-256
`d4797140eed55121de091518e7db4d9dcb5052d0c9a451ef6ded07e1589af8ef`. Both are class-file
version 52.0 and have 19 methods/`Code` attributes. `patch-report.json` records all six
field-info/Fieldref pairs and proves the 19 Code hashes are byte-identical.

The original and patched classes both pass `java -Xverify:all` and produce 267 runner rows.
The patched output records the B/C/S truncation, producer call count, previous field value, and
the ordering boundary where `value(..., true)` throws before a null receiver reaches `putfield`.
The source and patched output hashes are respectively
`00e009d8fda381ec5540a0fc745b8a42f04edbb7f98caa70f84d1996caeb7d6f` and
`eb57ef6493d6713fbaaaf26b87e55bf5708ca8acfb671270f6cc9b6c175cc10c`.

当前 CLI 在窄字段写入处输出 0 个 `@bytecode` 引用。完整源码可编译并运行；ignored Rust
测试将恢复运行结果与 patched class 对照。JADX 完整源码单独保存，当前 `javac` 失败。CLI
运行前后的 SHA-256 均为
`48edb9d2e3eec451983aabb4affcbcaf6d723c8a75b0284605f729743088cc76`。

Root independently replayed the same sources and patch with frozen CLI SHA-256
`7747b60a17dc635f0e8402d867cb9e74f9472a62057eb4865dbd8f4a207dfb34` in
`root-7747/`. The class and patched JVM-output hashes are unchanged; jarde's complete
source still compiles, but 254 of its 267 output rows differ from the patched JVM.
For example, patched `B:instance:value:-32769:stored:-1` becomes `stored:0` after
the refused field write. JADX's complete source still fails javac.
