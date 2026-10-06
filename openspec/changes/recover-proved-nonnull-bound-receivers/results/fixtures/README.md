# 门控实验的冻结 fixture（`recover-proved-nonnull-bound-receivers` 任务 1.1/1.2）

正例 `OP.java` 逐字节复制自 Optional 链巡查的 fixture
（[`optional-chain-patrol`](../../../../evidence/java-syntax-2026-10-05/optional-chain-patrol/README.md)
的 `fixture/OP.java`，sha256 `d6aca17f…`）；负例 `BRN.java` 为本片新写，三形：可空参数读、
可空字段读（实例字段）、捕获后重写。双腿 class：

```sh
javac --release 8 -Xlint:-options -d v8 OP.java BRN.java                        # javac 23.0.1
/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -d v8-javac8 OP.java BRN.java
```

**来源核验（实测）**：`v8/OP.class` 与巡查 jar（`optional-chain-patrol/fixture/op.jar`）内的
`OP.class` `cmp` **逐字节相同**——巡查 jar 就是同一条 `javac --release 8` 命令的产物，故锚的字节与巡查
记录同源；`v8-javac8/OP.class` 为同源另一条编译产物。两腿 `OP.sideEffect` 的字节码 BCI 0–27 完全同构，
唯一差别是创建期空检查的拼写（javac 23+：`invokestatic java/util/Objects.requireNonNull`@11；
真 javac 8：`invokevirtual java/lang/Object.getClass`@11，即 `facts::is_discarded_null_check` 的两种拼写）。

| 文件 | sha256 |
| --- | --- |
| `OP.java` | `d6aca17f51addc527103dfdc4c133f6d690bcb2564aa476f29ae7a69f61d0d0b` |
| `BRN.java` | `1992b8e50972239f1b34ece41d41c6cd02d6b789856af9dc3e48592bb43c33e1` |
| `v8/OP.class` | `758e87bd639fdd8cb81fb2e3af36ba836c8aa4e744eae152dbda10ae80ec30dc` |
| `v8/BRN.class` | `c7dad2309bf5417d81826c880ffda6f5f35b4b767fa1f998b3c04ff4e3c76235` |
| `v8-javac8/OP.class` | `24d0054c9a0852dc1a1220989708b14738b17652fd8f8b75c1f799905b99d936` |
| `v8-javac8/BRN.class` | `5aff4a4a9607bcdbb30fff362b30505eb900cf177c188d4b7bfd726da9effcf0` |

未放入 `tests/fixtures/`：本片**未落生产改动**（见
[../01-gating-experiment.md](../01-gating-experiment.md)），故没有引用它们的 CI 测试，而 handoff 纪律
禁止新增"未被 CI 引用"的冻结行为 fixture。待 root 重新设计落点后，这两份源与双腿 class 可直接提升为
`tests/fixtures/recover-proved-nonnull-bound-receivers/` 并配 `tests/recover_proved_nonnull_bound_receivers.rs`
（模式见 `tests/recover_temporal_argument_widening.rs`：`include_bytes!` + `Engine` + `ClassSourceRequest`）。

## 2026-10-06 更新：v2 重设计已落地，本目录转为实验记录

两份源与双腿 class 已**逐字节提升**为 CI 引用的冻结 fixture：
`tests/fixtures/recover-proved-nonnull-bound-receivers/`（`cmp` 实测逐字节相同；sha256 同下表，
`tests/fixtures/corpus-fingerprint.json` 与 reader 普查已随该提升重测）。本目录的副本保留为
门控实验（E 矩阵、落点实验）的原始记录，不再新增引用。
