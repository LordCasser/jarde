## Context

[巡查证据](../../evidence/java-syntax-2026-10-01/capture-ctor-order-patrol/README.md)：两模式字节码序与非法输出已冻结。合成字段事实来源：字段名（`this$` 前缀 + 数字 / `val$` 前缀）与 classfile 合成标志；ctor 呈现层在 `crates/jarde-java` 的构造器语句发射处（先定位现有 ctor 体呈现对 super 调用与字段存的次序处理）。第一个取证义务：确认现呈现是否已有"super 检测"结构（枚举 ctor 呈现 `super(arg1,arg2)` 首句已是先例——普通类 ctor 的 super 位置由字节码序决定，故此缺陷只在 javac 合成模式出现）。

## Goals / Non-Goals

**Goals:** 合成 pre-super 存组重排后呈现，super 首句；C1/C2 family 可编译、行为一致；普通 ctor/枚举 ctor 呈现逐字不变。**Non-Goals:** 隐藏合成字段声明（faithful 保留；隐藏属 family 内联呈现的后续选择）；`this$0`/`val$x` 的语义重命名；用户字段 pre-super 写（人为字节码）的重排；匿名类内联进外类源码。

## Decisions

1. **判据**：ctor 语句序扫描——super 调用前的连续 store 组，每个 store 满足（目标字段为合成：名字 `this$\d+` 或以 `val$` 起头，或字段的 access_flags 带 ACC_SYNTHETIC）∧（值为 ctor 参数的直传 aload）。整组确证才重排；任一不满足保持逐字。
2. **重排呈现**：`super();` 首句 + 重排组按原序 + 其余语句原序。呈现层实现，不触 SSA/证明。
3. **验收锚定**：C1（含 `C1$1/$2`）、C2（`C2$Inner`）family 联编运行一致（`25`/`6`、`10`）；负例：非合成字段 pre-super store（手工字节码或不可重排形态）保持现行为；普通/枚举 ctor 既有测试全绿。

## Risks / Trade-offs

- **合成字段判据误伤用户命名 `val$x`** → 佐证 ACC_SYNTHETIC 优先，名字模式仅后备；名字命中但无合成标志时保守不重排（负例钉死）。
- **多组/间隔形态**（pre-super 存与其它指令交错）→ 判据要求连续组；交错形态保持逐字（如实登记）。
