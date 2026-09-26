# CF-01：真实短路 CFG 与否定组合复放

本报告只把下列 javac Java 8 形态记为已测子集，不宣称 CF-01 整体完成。输入由 [ShortCircuitNegation.java](ShortCircuitNegation.java) 和独立观察驱动 [Runner.java](Runner.java) 组成：`&&`、`||`、它们的直接否定、`(a && b) || c`、该组合的否定，以及两个嵌套否定组合。每个叶子都调用会记录次数和先后顺序的 `probe`，Driver 穷举三个布尔入参的八种组合。

JADX 输入是账本指定的 JADX checkout `2fb1b16386941660fda07e9017285aec40fcb37f`；Jarde 输入是此 worktree `origin/main` 的 class-source CLI。原始 `.class` 由 `javac --release 8 -g:none` 构建；[javap 输出](original-javap.txt) 中的条件跳转（例如 `logicalAnd` 的 BCI 6 `ifeq 22`、`logicalOr` 的 BCI 6 `ifne 18`）实际决定后继是否执行，不是 eager `iand`/`ior`。固定 class、Java 源、runner、javap 及工具版本摘要见 [results.json](results.json)。

三方完整类源码均以 Java 8 重新编译，并和同一个 Runner 一起通过 `java -Xverify:all`。全部 64 条方法/输入观察的 stdout SHA-256 完全一致：`1ceed97f4fdd8b0d0cf909f23976583e1e93e0a78c41b5fa642cd878964509bb`。可见输出例如 `conditions(false,false,true)=true;2;ac`（右侧 c 被执行）、`conditions(true,false,false)=false;3;abc`，以及 `negatedConditions(true,true,false)=false;2;ab`（c 被跳过）。原/JADX/Jarde 完整源码分别保存在 [原始输入](ShortCircuitNegation.java)、[JADX 输出](jadx-ShortCircuitNegation.java.txt) 和 [Jarde 输出](jarde-ShortCircuitNegation.java.txt)；该子集 Jarde 输出可重编，运行行为与原 class 一致。

Jarde 的已观察源码包括：

```java
return arg0 && arg1;
return !arg0 || !arg1;
return arg0 && arg1 || arg2;
return (!arg0 || !arg1) && !arg2;
return arg0 && (!arg1 || arg2);
return !arg0 || !arg1 && arg2;
```

这些 Java 运算符优先级下的组合保留原分支顺序；驱动完整执行结果证明了 side-effect 次数与顺序。

生产入口从 [`Engine::class_source_with_evidence`](../../../../src/facade.rs) 进入 `prepare_physical_class_source`，再为每个方法调用 [`recover_for_class_source`](../../../../crates/jarde-java/src/report.rs)。`jarde-java::region::Walker` 将无法由普通不相交 `If` 臂表达的共享短路决策图作为私有 `Region::ShortCircuitValue` 持有；[`prove_short_circuit_value`](../../../../crates/jarde-java/src/build.rs) 核验真实边、测试来源、producer/phi/唯一消费以及副作用后才把条件值交给现有 emitter。两终端返回图由私有 `Region::TwoExitReturn` 路径单独表达。这轮没有发现已测形态的生产缺口，因此不提出 OpenSpec、不改实现。

## 与 JADX 账本的边界

- `TestConditions` 的 `(a && b) || c` 正向表达式属于上面实测的短路 CFG 子形态；增加调用计数与三入参只是用来证明实际跳过路径和负向组合。
- `TestBitwiseAnd` / `TestBitwiseOr` 源测试的 `&` / `|` 在其测试类中作用于字段，测试只断言输出含逻辑运算符；按 Java 语义，布尔位运算仍是 eager。它们不能替代有 side effect 的语义测试。固定 [eager 反例](../cf01-eager-boolean/report.md) 已实际表明 JADX 1.5.6 对 eager 布尔 `&` / `|` 输出 `&&` / `||` 会改变副作用次数；Jarde 保留原物理位运算。该 JADX 不佳处与本报告已证明的真实短路 CFG 分开记账。
- 没有检查账本中其余十余个 CF-01 变体、一般 `if` 语句条件、循环头组合、异常边内组合或不可呈现副作用；不从一组短路返回值向这些范围外推。

## 重放

在仓库根目录运行：

```sh
python3 openspec/evidence/java-syntax-2026-09-27/cf01-short-circuit/replay.py
```

脚本用 `TemporaryDirectory` 保存原始 class、JADX/Jarde 构建输入、三组 Java 编译产物和私有 `CARGO_TARGET_DIR`，结束后自动清理。可用 `JADX_ROOT`、`JADX` 或 `JARDE_CLI` 指向固定 checkout/现成可执行文件；默认验证 JADX commit 与结果中记录的固定账本。
