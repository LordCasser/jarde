# CF-10 数组长度循环条件验证

## 实现范围

`Region::test_is_pure` 仅在 header-tested loop 的终端分支同块 SSA 消费链中允许 `ArrayLength`，并要求该 SSA 输出在块内只有一个读取位置。其它测试操作、非消费读取和无法一次呈现的栈操作仍触发 `loop@1` 的 `StatementFree` 回退。Latch-tested loop 沿用原准入规则。

正例恢复了 `StepIndex.everyOther([I)I` 的完整索引循环、`i += 2` 更新和循环后返回；数组长度在每次条件求值中按原位置呈现。定向负例分别覆盖未消费的长度读取、复制后作为两个分支操作数复用的长度值，以及同一测试块内额外的局部写入。

## 三方隔离重放

环境为 `javac 23.0.1` / OpenJDK `23.0.1`，所有完整源文件均使用 `javac --release 8 -g -Xlint:-options` 编译，所有运行使用 `java -Xverify:all`。

### StepIndex

| 来源 | SHA-256 | Java 8 编译 | 运行输出 |
|---|---|---|---|
| 原始源码 `isolation/step-index/original/StepIndex.java` | `874313396926ffdc2593212edc1215e1ab7580707f277f6382c4712cc69ea592` | 通过 | `4` |
| 原始 class `isolation/step-index/original/StepIndex.class` | `64b5baa6270ce4604c0a70fa0a6d3438902bdeaa9f3bc3f7cb5105b40c512344` | 输入 | — |
| 固定 JADX 源码 `isolation/step-index/jadx/StepIndex.java` | `13c2d06f4731c0aa452822aa1a11b775d06050883f012fb8c764a5ed02d1349b` | 通过 | `4` |
| [Jarde 修后完整源码](evidence/StepIndex-jarde.java) | `61d6d63bafdb9ccfb1b8ad750df7d46df5df7af719df0be23b73aa3b98de63a3` | 通过 | `4` |

Jarde 正例测试断言 `values.length`、索引循环（无 foreach 冒号）、步长 `+ 2`、返回语句、无回退，并确认 BCI 6、7、16、22 都映射到恢复文本。`RecoveryEvidenceRequest::essential()` 与 `all()` 的源文本一致。

对 `everyOther(null)` 的 Java 8 运行探针在三方均输出 `NullPointerException:everyOther:2`：异常类型、首个方法帧和栈深一致，表明异常仍发生在每次循环条件求值的 `everyOther` 中。

### 组合 ForeachCases

| 来源 | SHA-256 | Java 8 编译 | `-Xverify:all` 输出 |
|---|---|---|---|
| 原始源码 `input/ForeachCases.java` | `2e082a247e01798f1f8172d0e8ebcbc6d3fb24224366ae9d77d3887d2d4b7eb5` | 通过 | `10` / `abc` / `4` |
| 原始 class `original/ForeachCases.class` | `6b65ab7efe39ce153eb008f626ed2f0cd382b2260c3c10a241fef8031b0c6864` | 输入 | — |
| 固定 JADX 源码 `jadx/sources/defpackage/ForeachCases.java` | `c6dd6d32896f63c3b2a5cebccb6416798753e1cf1b33b5bd6d0e24f135784c0c` | 通过 | `10` / `abc` / `4` |
| [本分支 Jarde 完整源码](evidence/ForeachCases-jarde.java) | `8caec4920fda6cc1bd38198814189fdb455f201b62e41c2cf89a2a9bc1f8a319` | 通过 | `10` / `4` |

本分支 Jarde 源码中的 `everyOther` 已完整恢复；`main` 在 BCI 53 的 `Arrays.asList` 调用参数仍因 `List→Iterable` 缺少安全引用转换证据而回退，所以组合运行没有 `abc`。这与数组循环无关，已有独立 [CF-10 失败隔离记录](../../evidence/java-syntax-2026-09-27/cf10-foreach/isolation/README.md) 描述该边界。这里不扩大当前变更；组合行为的合入后复核由主线任务处理。

## 自动化验证

- `cargo fmt --all -- --check`：通过。
- `cargo test -p jarde-java`：通过，包含 232 个 crate 单元测试及全部 crate 集成测试；新增未消费、复用、额外效果负例均通过。
- `cargo test --test p3_arraylength_loop_condition --test p3_array_foreach --test p3_iterable_foreach --test p3_loop_boolean_exit --test p3_loop_transfers`：通过，分别为 2、6、5、3、5 个测试。
- `cargo check --workspace`：通过。
- `openspec validate recover-arraylength-loop-condition --strict`：通过。
- StepIndex、ForeachCases 的原始/JADX/Jarde Java 8 完整源码编译及 `-Xverify:all` 重放：均成功；输出差异按上表记录。

CF-10 当前保持**部分已测**：隔离步长循环和局部结构门已验证；组合类的 Jarde 输出还依赖其独立调用参数转换在目标主线上的验证结果。
