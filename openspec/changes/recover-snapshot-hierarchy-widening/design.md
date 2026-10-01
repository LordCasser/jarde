## Context

[巡查证据](../../evidence/java-syntax-2026-10-02/user-hierarchy-widening-patrol/README.md)：I1 main 的 BCI 27–45（`new I1$En; dup; invokespecial; ldc; invokestatic viaInterface(LI1$Greet;String;)`）。既有通道：`reference_overload_calls`（build.rs 分派第 5 回答，per-BCI 证明由有环境的 pass 填充——**结构先例**）；平台闭集两表。类 header 读取先例：member-family/enum-constant 家族装配按需读子类报告与 header（A16 计费）。第一个取证义务：定位 `reference_overload_calls` 的生产 pass（谁填、如何拿环境/快照类表）与类 header（super/interfaces）在 reader 层的既有读取面，确定证明收集的挂点。

## Goals / Non-Goals

**Goals:** 同快照双物理类的 T→U widening（extends 链 + interfaces 传递，有界）；I1 四路径恢复、行为一致；既有回答与负例零回退。**Non-Goals:** 单边快照外（用户类→平台目标、平台→用户）；数组协变（数组闭集已有）；返回位/赋值位（本片实参位）；动态代理/反射层级；跨 snapshot。

## Decisions

1. **per-BCI 证明集合 + 分派消费**（复刻 `reference_overload_calls` 结构）：pass 扫描方法调用实参位，对呈现 T、要求 U 且现回答全不命中的位，若 T、U 均为本快照物理类则沿 header 链 walk（深度上限如 8、访问集防环）；命中产出 `snapshot_hierarchy_widens{bci, source, target}`；build.rs 分派在该集合查询后走 `cast_argument`。
2. **header 链事实 = 快照自身字节**：super_class + interfaces 数组逐级读取（被分析工件内的类，非 classpath）；每级读取计费（A16 语义：按需 header）。链上类缺于快照（如 extends 平台类）即停该支——该位回退拒绝。
3. **边界**：T==U 走同名分支；数组双方走数组闭集；证明失败/超界保持现拒绝文本。负例：快照内无关联两类（En implements Other → 传 Greet 位）拒绝；`final` 类传 Object 走既有 Object 分支。
4. **验收锚定**：I1（`hello:v` 回归）+ 变体（两级继承 En→Mid→Greet、接口多实现 En2 implements Greet,Other、匿名类传接口）；三方重编运行一致。

## Risks / Trade-offs

- **快照类表查询越界** → 只查本快照物理类名；缺类即停（decision 2），不引 classpath。
- **环/深链** → 访问集 + 深度上限；超限拒绝。
- **与平台闭集重叠** → 平台对先行（现分派序不变），本集合是其后回答；顺序固定并在测试钉死。
