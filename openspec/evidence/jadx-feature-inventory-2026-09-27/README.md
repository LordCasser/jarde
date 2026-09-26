# 以 JADX 测试和实现为起点的 Java 语法清单

基线是本地 `/Users/lordcasser/workspace/testzone/jadx` 的提交 `2fb1b16386941660fda07e9017285aec40fcb37f`。`jadx-core/src/test/java/jadx/tests/integration/` 共 612 个 `.java` 文件，分布在 27 个一级目录；609 个文件名以 `Test` 开头，608 个含 JUnit 测试注解，26 个含 `@NotYetImplemented`。这些是**文件/标记数量**，不是语法特性数、通过数或 Java 8 覆盖率；同一测试可能检验多个语法点，目录 `names`、`deobf`、`fallback` 等还包含非语法基础能力。

本次审计先按控制流、声明/类型、表达式/杂项三个互斥**文件归属**分工；每份报告再将测试断言归纳成可独立验收的**特性单元**。一个单元至少有准确测试路径和生产算法入口，并标明正例、拒绝边界及 JADX 测试自身的 `NotYetImplemented`/弱断言。跨目录的同一个特性只计一次，引用另一组 ID，不凭文件名推断已实现。Jarde 状态须以当前测试、源码输出及 Java 8 重编/运行证据重新核对，不能由 OpenSpec 勾选或 JADX 输出代替。

目录文件数可在该 checkout 用以下命令重算：

```sh
rg --files jadx-core/src/test/java/jadx/tests/integration -g '*.java' | sed 's#^jadx-core/src/test/java/jadx/tests/integration/##' | cut -d/ -f1 | sort | uniq -c | sort -nr
```

| 文件组 | 目录 | 文件数 |
| --- | --- | ---: |
| 控制流 | `conditions`, `loops`, `trycatch`, `switches`, `synchronize` | 215 |
| 声明/类型 | `types`, `inner`, `enums`, `names`, `generics`, `java8`, `annotations` | 173 |
| 表达式/杂项 | 其余 15 个目录 | 224 |
| 合计 | 27 个目录 | 612 |

三组测试和生产代码入口的去重结果、验收单元数与实施次序见 [summary.md](summary.md)。实现顺序由 Jarde 实测差距、语义风险和架构依赖决定，再为每个闭合语法点写 OpenSpec。先对齐可证明的 JADX 已有工作，之后才探索双方均未覆盖的情况。
