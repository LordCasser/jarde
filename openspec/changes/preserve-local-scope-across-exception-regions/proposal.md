## Why

> **root 锚数据点（2026-10-05）**：最小常见形实测——平铺 `try{return 100/n}catch(Exception){return 1}finally{println}`（[NA-b 探针](../../evidence/java-syntax-2026-10-05/try-catch-finally-slot-reuse-patrol/README.md)）整方法拒绝（"local 1 crosses a quoted fallback region"）：javac 复制 finally 进三出口并复用 slot 1 为 try 结果 int 与 catch 异常对象。**catch 内嵌 try 本身已恢复**（同探针 NA-a）。建议 NA-b 并入 task 2.3 验收锚。同因追加：`EX.nestedFin`（finally 嵌 finally，[exception-shape-family-patrol](../../evidence/java-syntax-2026-10-05/exception-shape-family-patrol/README.md)）——锚家族 = NA-b（平铺 t/c/f）+ EX.nestedFin（嵌 finally），同一 "local crosses a quoted fallback region" 拒绝。

> **root 追加锚（2026-10-05，[rethrow 巡查](../../evidence/java-syntax-2026-10-05/finally-rethrow-patrol/README.md)）**：`rethrow`（try 抛出+catch 打印+finally 打印+尾部 return——finally 复制三出口中的**异常 slot 跨引注区**）同因拒绝；同巡查证健康面：catch 内重抛 `throw e`、**multi-catch 精确 rethrow**（`catch(A|B e){ throw e; }`）全部恢复且行为精确。锚家族四形、[catch 链+finally 第 5 数据点](../../evidence/java-syntax-2026-10-05/catch-order-recursion-patrol/README.md)（order：三段 catch 层级序+finally 的异常 slot 跨区）。

恢复器目前把跨 try/catch 的局部变量按方法级“已声明”状态处理，可能在 catch 块内声明变量，却在 catch 之后输出读取它的语句；生成文本因此连 Java 词法作用域都不成立。该缺口已由真实 Java 8 class、原/JADX 行为及 javac 错误确认，需要先让局部声明可见性与 region 结构一致，再谈更广的异常路径恢复。



> **root 追加锚（2026-10-05，[equals-contract 巡查](../../evidence/java-syntax-2026-10-05/equals-contract-patrol/README.md)）**：**equals 契约无异常表形**——`HE that = (HE) o` cast 局部 + getClass 分支 + 字段比较链触发同一 "local crosses a quoted fallback region" 诊断（无异常表参与）；剥离文本编译失败=安全方向，但 equals 是最普遍方法形状应可恢复；实现时核实与异常区形是否同一 crosses 机制（锚家族第 6 形）。**第 7 锚（[composite-scanner 巡查](../../evidence/java-syntax-2026-10-05/composite-scanner-patrol/README.md)）**：for+switch 复合（词法扫描器形状：局部 SB 在循环 switch 各分支消费+continue+prev 状态）整方法 crosses——幸存文本空=安全，jadx 完整解；新控制流形状（循环×switch 交叉，无异常表）。**第 8 锚（[nested-loop 巡查](../../evidence/java-syntax-2026-10-05/nested-loop-accumulation-patrol/README.md)）**：嵌套循环累积（`for(i){ for(j){ s += m[i][j]; } }`）整方法 crosses——幸存空 body 编译失败=安全，jadx 完整解；判别点：单层循环累积恢复（#103）、嵌套层拒——局部定义-使用切片跨外层循环。

## What Changes

- 恢复输出中的局部声明与读取遵守 Java 词法作用域；嵌套 region 内声明的局部离开作用域后不得被继续引用。
- 无 LVT 时以 catch clause 与实际定义—使用证据区分同一 JVM slot 上的独立 handler 参数，避免把兄弟或嵌套 catch 误判为一个逃逸变量。
- 当异常边或 quoted/fallback 区域使跨 try 的定义/使用关系无法完整恢复时，拒绝会产生越界读取或丢失原始效果的相关结构，并按既有 refusal 契约保留相关 bytecode、origin 和停止/拒绝信息。
- 为 try/catch 局部作用域补充正向、拒绝及边界验收，涵盖无调试表 class、嵌套作用域与区域外读取。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：补充跨异常 region 的局部声明可见性与拒绝边界要求，不把现有局部声明恢复另立能力。

## Impact

- 影响 `crates/jarde-java` 的 region、局部声明规划、AST 构建与恢复测试；不改变 `src/class_source.rs` 的类级成员拼装职责。
- 前置是现有 canonical CFG、exception table、Frame/SSA、Region 和局部身份事实；未知的 reaching definition 或越界 scope 关系必须拒绝，不能从名称或 BCI 邻近关系推断。
- 不属于 `recover-conditional-values` 的条件值准入，也不处理 bridge/class-source 投影；这些仍由各自变更负责。
