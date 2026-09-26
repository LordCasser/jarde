## Why

2026-09-26 巡查（自写 `Patrol.multiResource`：两个 `BufferedReader` 资源，javac --release 8）发现：多资源 try-with-resources 在 jarde 中整方法退为字节码引用，拒绝码 `jre_guard_handler_range`（`RangeEnd`：「the protecting row does not end where this shape requires」）。现有 `twr` 证明已按层级链构建资源（`chain` 逐层向外），但几何校验要求「外层行止于内层 handler 的 span 末」。仓库原有 `Guarded.two` 正例恰好满足此式：外层行 `[6,46)` 包住内层 handler `[26,46)`。新样例的外层行却是 `[5,33)`，止于自己的正常 close 起点 33；另一行 `[44,71)` 单独把内层 handler 的异常清理保护到同一个外层 handler 71。只替换结束 BCI 的等式会漏掉这条同目标保护行。

jadx 1.5.6 不还原 TWR 语法（展开为手写嵌套 try + close + `addSuppressed`）；单资源 TWR 已是 jarde 的反超点，多资源还原可扩大差距。`recover-null-resource-headers`（8/8）已处理资源头部的 null 检查形状，本 change 处理多层级几何与呈现。

## What Changes

- 多资源 TWR 按每层正常 close 的真实起点证明行的结束 BCI；内层异常清理必须由外层原行覆盖，或由范围精确等于该清理段、且 catch 类型和目标 handler 与外层原行相同的第二行覆盖。保留原有单行形状；不把任意多出的同目标行当作清理证据。
- 对体内结果在 close 后才 `load; return` 的形状，证明返回值在体内产生且尾部只有纯读取/返回，再把 `return` 放回 `try` 的体内；否则维持拒绝，避免局部声明落在体内而返回落在体外的不可编译文本。
- 呈现为一条 `try (T n1 = …; T n2 = …) { body }`，资源按初始化顺序，正常路径 close 逆序保持既有证明；异常路径的 close + `addSuppressed` 链按层保持既有证明（`close_handler`、`Suppressed`）。
- 与 catch 组合（TWR + 用户 catch 行）保持既有规则；`enclosing_clauses` 几何不变。
- 几何或 close 证明任一失败保持整方法引用；不降级为部分 TWR。
- 自写 fixture（javac --release 8 与 9+ 两种 lowering 各冻结一份；9+ 的 `ifnull` 形状已在 null-headers change 覆盖，本 change 的正例至少覆盖 8 的无检查形状），三方对照与重编译执行（正常返回、异常路径的 suppressed 顺序）。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：新增多资源 try-with-resources 的层级几何要求与整语句呈现要求。

## Impact

实现集中在 `crates/jarde-java/src/guard.rs`（同次层级几何、同目标保护行与返回尾部的局部证明）与必要的 `build.rs` 语句装配。复用既有 close/addSuppressed 与 return 表达式构建，不新增公开 pass、crate 或生产依赖；monitor 与 any 行 finally 规则不变。

非目标：单资源形状改动；`ifnull` 资源头的新证明（沿用 null-headers）；TWR+多捕获；资源 close 顺序的新放宽。
