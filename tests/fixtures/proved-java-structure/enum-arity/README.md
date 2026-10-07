# 顶级普通枚举的 0、1、4 常量

`Empty.java`/`One.java`/`Four.java` 是 `package probe` 下 0、1、4 个常量的普通枚举，`EnumArityRunner.java`
打印三者各自的元数、名字、序号与常量表。四份源码以 `javac --release 8` 编译，冻结的 class 唯一副本在
本目录 `v8/probe/` 下（`v8/` 是生成标记，不是包名），字节身份由 `SHA256SUMS` 固定；major version 52。

完整重编/运行与 JADX 1.5.6 对照由 [replay.py](../../../../openspec/evidence/java-syntax-2026-09-27/enum-arity/replay.py)
重放；证据报告见 [report.md](../../../../openspec/evidence/java-syntax-2026-09-27/enum-arity/report.md)。

## 行为基线（CI 守卫实测，2026-10-07）

`java -Xverify:all -cp v8 probe.EnumArityRunner` 运行冻结 class（JDK 23.0.1）的输出：

```text
empty=0
one=ONLY:0/1
four=[NORTH, SOUTH, EAST, WEST]
```

CI 守卫见 [`tests/fixture_behavior_guards.rs`](../../../fixture_behavior_guards.rs)：呈现腿在默认套件
（三个常量头 + runner 的常量读取），行为腿标 `#[ignore]`（`cargo test --test fixture_behavior_guards --locked -- --ignored`）
并重编四份渲染文本后对照运行输出。取证与分类见
[`openspec/changes/recover-fixture-behavior-guard-coverage/results/`](../../../../openspec/changes/recover-fixture-behavior-guard-coverage/results/README.md)。