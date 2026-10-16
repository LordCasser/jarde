# Proposal

## Why

立项时 a89 产品的 `FY.loopless([I)I` 仍以 `jre_guard_finally_copy` 拒绝；其一个受保护条件体有保存值返回与正常落穿两条出口，分别执行相同清理，另有异常清理副本。新独立完整类的实际12命令对照已确认：四次原类Java8/23 driver行为一致，但两份完整生成源码均缺return而不能重编，现有直线finally证明未覆盖这三副本形态。

## What Changes

- 只为一个catch-all、无循环条件体的“一个保存值return或正常落穿”证明两条正常清理及一条异常清理，恢复等价的结构化try/finally并继续真实后继；保持返回值先求得、清理恰一次和异常覆盖顺序。
- 复用既有清理等价、SSA/异常范围、Region与Try/Return发射；先核完整候选，再进行有预算的证明，证书与ownership原子发布，不扩大普通finally或全局CFG求解。
- 固定双编译腿完整独立类与未改driver，以空数组/[2]/[3]/null、清理抛异常和清理修改返回字段的对照验收；原FY完整输出另存，不删noCall来换取整类通过。

本片是 `recover-return-through-effectful-finally` 的有界子项，不替代其FY.noCall与FX.finReturn剩余目标。旧父计划把所有位置归为local-crosses及把FO当finally缺口的描述已经过时：FO静态prefix是独立缺口，a89新CLI的完整源码重编与运行已通过，父计划本身仍未完成。本片不修改旧父规划或其任务状态，也不把新独立类误称原FY同字节。

立项前置状态（历史）：从实际源码 a89d423c9a6e5d62ceb3f8185a818ebe3d67a17b 建独立工作树；静态prefix片自身CI和归档尚在完成中。本片可先规划/冻结红基线，最终产品交付须在前片完成归档后重绑定主线。该段记录立项前置，非当前完成状态。当前实施及验收状态以tasks.md和results/fixed-recovery-v1为准。

非目标：循环、多个返回竞争、嵌套/交叉handler、任意副本图等价、清理自身分支/非局部出口、跨请求求解器、全局类型/名称解析；不增加CF11/CF18/71/612等全局覆盖数字，不改变原Java/driver/oracle或生成源码来过门。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：新增已证明条件返回/落穿的三清理副本恢复，约束求值、清理次数、异常身份、来源、停止与完整源码实测。

## Impact

预计局限于jarde-java既有guard证书、bounded region和finally返回发射接缝，公共库及薄JSON CLI添加永久回归；真实fixture沿现有语料指纹纳入。无新依赖、crate、公共运行器或宿主协议。既有直线/条件清理/共享catch/循环finally规则保留自己的准入和计费，不靠放宽它们的判据接纳本片。
