## Context

见 [proposal.md](proposal.md)及[CF-06 物理对照](../../evidence/java-syntax-2026-09-27/cf06-inner-assignment/baseline/javap.log)。`lengthBranch` 的 BCI 8 调用 `String.length`，11 `dup`，12 `istore_1`，14 比较，19 再读局部返回；`assignedAndChecked` 的 BCI 9 `getfield`，12 `dup`，13 `astore_2`，14 判空，17 再读局部调用。`ExprKind` 目前没有局部赋值表达式，`Operation::Duplicate` 除已证链式赋值外会 quote；后者的 `ShortCircuitValue` 候选在 SSA 测试闭包处拒绝共享值。局部赋值语句、变量复用与声明、SSA/CFG、条件组合和来源映射已有，不需要第二套图。

## Goals / Non-Goals

**Goals:** 用一个源级局部赋值表达式表示物理值只计算一次而在同一条件里写局部并继续使用的形状；使两种固定方法全类重编运行，保留其它重复值形状的拒绝。

**Non-Goals:** 任意赋值表达式泛化、字段/数组左值写入、无 `dup` 的代数重构、CF-07 循环 scope、带异常 handler 的等价变换、JADX Smali 警告测试的正向背书。

## Decisions

1. **最小 AST 承载局部赋值。** 增加只表达 `local = value` 的 `ExprKind` 形状，类型取已证目标局部的声明类型；打印为 Java assignment 优先级，在比较、判空等更紧的操作数位置自动加括号，保留物理 `dup`、store 和 RHS 的来源。用裸字符串塞进 `ExprKind::Local` 会绕开类型、来源与优先级，不采用。用多条提前语句拆短路条件虽能覆盖部分执行语义，但不能一般地保证只在相应分支赋值，也会对后续循环头条件引入另一套结构改写，因此本切片不采用。
2. **从物理复制与 SSA 闭合建立候选。** 只认同一正常路径中的一次 `dup`、紧随的局部 store 与后续条件测试；证明两份复制值分别只到该 store 与测试，store 的目标变量/类型和后续读取来自同一个 SSA 变量，右值来源可被现有 `render_value` 只写一次。对调用或字段读取保持原可抛/有副作用位置；有额外用途、不同目标、phi/异常路径或跨局部作用域即拒绝。将证书交给现有条件构建和 `ShortCircuitValue` 测试闭包消费，store/dup 被这一表达式独占，不能再作为独立语句写第二次。
3. **原子构建与现有预算。** 条件及赋值 RHS、声明、所有被领取指令先证明后发布；通过既有停止传播拒绝预算/取消。保持 `StmtKind::Declare` 在条件前且无错误初值、`StmtKind::Return` 及现有局部作用域检查。新增遍历按预算计费；比较原 class、固定 JADX、Jarde 的完整 Java 8 源码与运行，不依赖外部库：已有 noak/SSA/区域层能提供所需物理事实，第三方源码重写器无法提供本项目的所有权和来源证书。

## Risks / Trade-offs

- [将 `dup` 两份值都写成 RHS，重复触发调用/字段读取] → 只允许一个复制值到 store、另一份到比较，测试副作用调用次数和字段写后读取。
- [赋值表达式类型或括号错] → 类型取局部声明而非 JVM int 栈形；以 Java 8 全类重编和表达式优先级定向测试验证。
- [局部声明移到短路分支内导致后续读取无定义] → 保持现有词法计划、验证 `lengthBranch` 的早退/正常退出与 `assignedAndChecked` 的判空路径。
- [CF-07 并行修改相邻区域代码] → 该变化限定表达式/短路值与局部归属；若必须改循环 Frame，先停止并由 root 拆分，不与 CF-07 混合。
