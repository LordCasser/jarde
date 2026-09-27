# CF-14 实施验收

2026-09-27，以归档 CF-14 三个原始 class、输入源码、runner 和固定 JADX 源码为输入。Jarde 源码由本分支 `jarde-cli class-source` 重新生成；原始、JADX、Jarde 均按完整类和同一 runner 以 `javac --release 8 -g:none` 编译，可编译者用 `java -Xverify:all` 运行。

| 样本 | 原始 | 固定 JADX | 本分支 Jarde |
|---|---|---|---|
| `NestedStringSwitchAudit` | 编译、验证，五行 | 编译、验证，五行相同 | 编译、验证，五行相同；两个嵌套 String switch，无 `@bytecode` 和不可达 `break` |
| `StringSwitchAudit` | 编译、验证，八行 | 编译、验证，八行相同 | 编译、验证，八行相同 |
| `ExtraHashUse` | 编译、验证，六行 | `javac` 因未定义 `r0` 失败 | 编译、验证，六行相同；保留 `hashCode()` 和两个物理整数 switch |

嵌套样本的 Region owner 唯一包含 block 起点 BCI 62/96/105/111/120/123/152/154/156。BCI 71 和 125 是分别位于 block 62 和 123 内的 switch 指令；十一处要求检查的 BCI 均有源码映射。续接只接纳 child switch 的单个已证 join，要求 case entry 支配它、规范图入边均来自该 child、后续 final switch 的新 owner 与走访集合完全一致，且无外部入口、其它 case entry、循环头或异常/子程序边。不能证明时整段保守拒绝。把固定 class 的 BCI 108 `goto 123` 改为 `goto 60` 后，`-Xverify:all` 仍通过，定向测试确认跨 case 流不会误恢复为嵌套 String switch。

`cargo test -p jarde-java` 全部通过，包括新增的正例、跨 case 负例、独立 hash 用途，以及预算/取消不发表部分源码的断言。`cargo fmt --all --check`、`cargo check -p jarde-java -p jarde-cli`、`git diff --check` 和 `openspec validate recover-nested-string-switch-arm-continuation --strict` 均通过。测试使用独立的 `/tmp/jarde-cf14-target`，完成后清理。
