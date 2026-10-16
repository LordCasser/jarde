## Why

原patrol矩阵记录2026-10-05历史基线：当时把continue/break/空finally与finally返回值形报告为已恢复，把noCall/finReturn/loopless报告为整方法拒绝。这些旧结论不作为当前能力声明；相同“local crosses”文本也不证明同一缺口。

当前主线f4a81440d已交付FO静态前置更新与无循环条件finally。后续noCall子片产品 `02de294f7` 已完成本地验证，最终CLI fd7b33ce…ff25 的实际20命令重筛保留两编译器来源的完整FY5方法、FX6方法、FO5方法：FY/FO均原样重编通过，各在Java8/23运行main，总计8次逐字匹配原oracle；它们的逐方法报告均structured/java且无fallback。FX.finContinue/finReturn/finBreak仍为mixed/fallback，其中finReturn使两份整类重编缺return，无重建运行。noCall产品自身CI38872425446已终态独审通过（双seed各3663/0/98），该子片正常同步规格并归档交付；父剩余任务仍保留。

父范围仍包含FX.finReturn、其它未证明形与完整JADX/零回退验收。固定FY/FO的完整成功不代表所有finally或父计划已经完成。

**核心可证性**：Java 语义要求 return 表达式先求值（恰一次）、finally 副作用后执行（[FO 基准](../../evidence/java-syntax-2026-10-05/return-effectful-finally-patrol/fixture/FO.java)：`try{return bump();}finally{i=100;}` 输出 `1/100`）。朴素把 finally 复制到 return 语句之前会错序——正确形是 **temp 绑定**（`t = <expr>; <finally body>; return t;`）或等价可证形式。

取证：[return-effectful-finally-patrol](../../evidence/java-syntax-2026-10-05/return-effectful-finally-patrol/README.md)。

## What Changes

- 复用已存在的Finally/SavedReturn及私有continuation，按可证叶片推进单return穿副作用finally，不另立通用temp框架。FO与loopless已归档；noCall由 `recover-local-increment-loop-finally` 承接，FX.finReturn仍需后续有界证明。
- 调查结论与历史问题对应：
  - **Q-i（历史）**：loopless的原始拒绝链与修复见 `2026-10-16-recover-conditional-return-finally-copies` 的实际CFG/SSA/全源码归档；不重复实现。
  - **Q-ii**：noCall已用真实Facts/Region定位到iinc副本、handler phi与循环scope，子片只授权已证return块；finReturn须独立核实际值流/清理异常及出口，不能凭诊断同名合并。父级instrumentation.md尚未创建，不把子片报告当作该文件已完成。
- 拒绝边界保持：多 return 出口竞争同一 finally、finally 内含非局部出口（自身 return/break）、求值序不可证（副作用与 return 表达式交织）——维持整方法响亮拒绝。
- 对照零回退：空 finally、continue/break 分布、finally 返回值形逐字节不变。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：扩充可证明的 return 穿副作用 finally 恢复，保持求值序、副作用次序与拒绝边界。

## Impact

主要涉及 `jarde-java` 呈现层（exit/finally 等价分布）与局部 def-use 词法绑定；验收 fixture：[patrol](../../evidence/java-syntax-2026-10-05/return-effectful-finally-patrol/) 的 FY.java（noCall/loopless/emptyFin 锚）+ FX.java（finContinue/finBreak/finReturn）+ FO.java 求值序判别。
