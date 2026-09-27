## Why

固定 JADX 的 `TestEnumsWithTernary` 使用单个 String 条件表达式作为每个 enum 常量的构造实参。Jarde 虽已分别证明 int 三元实参和 `String...` 字面量数组，却仍把这个完整、可编译的普通 enum 保留为物理字段与 `<clinit>`，使生成源码无法重编；这是 DT-11 单 String 实参与 DT-14 条件实参的交叉缺口。

## What Changes

- 对准确的 `(String,int,String)` enum 构造器、唯一同 owner String 字段写入和完整常量前缀，证明标量 String 实参；首批仅接收 ASCII 字面量或同类无参布尔静态调用控制的双 String 字面量条件表达式。
- 复用现有 int 条件实参的分支极性、两个 arm、唯一汇合与构造器消费约束，以及 DT-11 的 String literal 解码/转义；保持标量 String 与 `String...` 数组的不同证书。
- 任何额外效果、分支或成员身份歧义、未解释指令、异常边、预算停止或取消都原子拒绝完整 enum 常量组，保留物理来源。
- 用原 class、固定 JADX 与 Jarde 完整 Java 8 源码编译及 `java -Xverify:all`，覆盖 true/false 两 arm、条件调用次数与现有 int/varargs 控制。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：增加受证明的标量 String enum 构造实参及其有界条件表达式恢复与原子拒绝边界。

## Impact

涉及 `src/enum_constants.rs` 的普通 enum 组、实参与构造器证明，及 `src/class_source.rs` 的源声明/常量参数投影；只复用现有 reader、Code/SSA、来源、预算和 class-source 装配。无需新 crate、通用 CFG pass 或 host 依赖。固定对照见 `openspec/evidence/java-syntax-2026-09-27/dt14-enum-init/`。复杂 String 表达式、非 ASCII 字面量、匿名 enum 常量体、任意 `<clinit>` 与普通方法中的三元表达式均不纳入。
