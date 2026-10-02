## Context

[巡查证据](../../evidence/java-syntax-2026-10-02/super-default-patrol/README.md)：两拒绝形（单限定/菱形双限定）字节码 = `aload_0; invokespecial InterfaceMethod F1$A.name` （append 链内）。通道与拒绝点：build.rs:21191 附近。快照 header 读取先例：snapshot-hierarchy-widening（interfaces 数组 walk、A16 计费、构造性不可达 classpath）；成员读取先例：member-family/enum 家族按需子类报告。**第一个取证义务**：读 21191 附近证明选择逻辑——确认失败环节（未尝试读快照事实 vs 判据过严如要求多接口冲突触发）、以及该证明的消费点是否已有"直接超接口"白名单雏形（菱形场景两限定符都需各自证明）。

## Goals / Non-Goals

**Goals:** 快照内单限定与菱形双限定恢复；普通 super/this、抽象限定（目标无体）、快照外接口零变化。**Non-Goals:** `X.super.m()` 之外的 invokespecial interface 形态（private 接口方法为 Java 9+，Java 8 不存在）；菱形中类**未覆写**直接调用（Java 8 非法编译产物）；快照外接口（登记，单类输入典型场景）；静态接口方法调用（invokestatic，既有）。

## Decisions

1. **快照事实判据**：限定符 ∈ 调用类 header 直接 interfaces（extends 链上的接口不算——Java 语法要求直接实现才可限定）∧ 目标方法在该接口成员表非 abstract、非 static。读取走既有按需成员读取与计费；读不到（快照外）保持现拒绝文本。
2. **呈现**：`X.super.m(args)` 限定拼写 + 既有实参/结果通道；菱形两个限定调用各自独立证明（同判据）。
3. **验收锚定**：F1 全家族 fam.jar（`AB`/`I:A`）+ 变体（菱形+自身覆写混合、super 默认返回 void、带参默认方法）；负例（限定符非直接接口〔经 extends 继承〕、目标 abstract、快照外接口）。

## Risks / Trade-offs

- **"直接"判据过宽**（间接接口被接受 → 源码非法）→ 严格只认 header interfaces 列表；间接形负例钉死（保持拒绝）。
- **成员读取计费膨胀** → 仅在 interface-special 位触发读取，非全局扫描；预算/取消原子性测试沿用。
