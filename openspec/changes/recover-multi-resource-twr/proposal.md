## Why

2026-09-26 巡查（自写 `Patrol.multiResource`：两个 `BufferedReader` 资源，javac --release 8）发现：多资源 try-with-resources 在 jarde 中整方法退为字节码引用，拒绝码 `jre_guard_span`（`RangeEnd`：「the protecting row does not end where this shape requires」）。现有 `twr` 证明已按层级链构建资源（`chain` 逐层向外），但几何校验要求「每一层的行止于内一层 handler 的 span 末」，而 javac 的多资源 lowering 是**嵌套语句**几何：外层行止于它自己的语句体末（内层资源声明 + 体 + 内层正常 close 链之后），不等于内层 handler 的 span 末。

jadx 1.5.6 不还原 TWR 语法（展开为手写嵌套 try + close + `addSuppressed`）；单资源 TWR 已是 jarde 的反超点，多资源还原可扩大差距。`recover-null-resource-headers`（8/8）已处理资源头部的 null 检查形状，本 change 处理多层级几何与呈现。

## What Changes

- 多资源 TWR 按嵌套语句几何证明：每层行的范围 = 该层资源的初始化起点到该层正常 close 链起点，行的 handler 覆盖该层的异常路径；层级链的构造（严格包含、`closes_something`）保持，几何校验改为「层 i 的行止于层 i 的语句末」这一可从字节码证明的事实。
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

实现集中在 `crates/jarde-java/src/guard.rs`（层级几何校验、层级与呈现的数据结构）与必要的 build.rs 子句装配。TWR 的 close/addSuppressed 证明、monitor 规则、any 行 finally 规则不变。无新 crate、pass、生产依赖。

非目标：单资源形状改动；`ifnull` 资源头的新证明（沿用 null-headers）；TWR+多捕获；资源 close 顺序的新放宽。
