## Why

已封存的 `FinallyCompletion.tryThrowsFinallyRuns()I` 没有正常 return：try 执行副作用并抛原异常，finally 完成清理后恢复该异常；现有源码仍 fallback。它是已登记的 finally 完成类型缺口，应独立于已验收的返回型 finally 继续推进，避免把“没有正常完成”错误解释为 void return 或正常清理副本。

## What Changes

- 先核两真实编译器输入的 canonical CFG、SSA、effects、异常覆盖及实际 guard 首拒，再决定既有私有 Plan/Builder 接缝所需的最小补充；不预设新公开机制。
- 恢复有界、直线 try 以原始 throw 完成且 finally 正常落穿的源码结构；普通 cleanup 调用可能抛错，其异常必须覆盖 pending primary，不能作为拒绝理由。
- 保留全部物理方法/指令来源，以完整独立类及独立 driver 检查 try/cleanup 副作用、恰好一次、异常对象身份及覆盖顺序；封存完整 FinallyCompletion 的其它红方法，不删方法凑可编译。
- 固定 JADX 同输入对照；以原始 class 行为为 oracle，保留外部工具真实失败。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：为经证明的仅异常完成 try/finally 增加正向源码恢复要求，保持拒绝、来源及分析停止边界。

## Impact

前置条件是 finThrow/nestedFin 组合产品 `4c6852ac8c4ab9ce88f0716f4d2bd702e15684c5` 已验收，当前主线为其同源文档提交 `0cf8eb77d16a89c2421d08bac35ffbed4ca1e612`。实现限于既有 `jarde-java` guard、Region/Builder 与测试/来源入口，不新增依赖或产品入口。

不包含 finally 自身显式 return/throw、任意嵌套/分支/循环异常图、通用异常仲裁框架或其它语法单元。父返回型 finally 的7/7及71单元/612测试总账不因本计划改变；本计划不表示功能已经实现。
