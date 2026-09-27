## Why

CF-08 的纯整数 `while (true)` 双 `break` 已构成最小可执行反例：原 class 与固定 JADX 完整源码 Java 8 重编运行一致，Jarde 却只识别一个循环出口，遗漏另一条 BCI 15 `goto 24`。带 `@bytecode` 的 Jarde 输出恰好能重编，却在 `limit=4` 时不终止；恢复必须按真实出口目标闭合所有权，不能靠 Java 编译成功判定正确。

## What Changes

- 在现有循环 Region/Frame 中证明：循环头的纯转接出口与体内的纯转接 `break` 指向同一个实际循环后继时，它们分别保有物理来源、共同指向该循环的退出目标。
- 只在 CFG 入边、出边、SSA 指令、副作用和作用域均闭合时发出循环体内 `break`；未能闭合时保持带来源的引用，不能发布遗漏出口的完整方法。
- 固定纯整数双出口的三方 Java 8 重编与运行、来源、负例和预算/取消测试；`TestNotIndexedLoop` 的外层分支/局部汇合继续单列，不纳入本次实现。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：循环头正常退出经纯 `goto` 网关、另一个体内 `break` 网关直达相同后继时，保留退出目标和每条可达路径语义。

## Impact

主要涉及 `crates/jarde-java/src/region.rs` 的现有循环出口归属/区域走访，并在 `build.rs` 中只为双网关证书附着纯转移的派生来源；另有必要的定向回归及 CF-08 固定证据；沿用当前 JVM IR、SSA、预算、来源和原子拒绝合同。不引入新 crate、公共 pass 或额外 AST/Region 种类，也不改变其它循环形态的准入。
