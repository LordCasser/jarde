## Context

见 [proposal.md](proposal.md) 和 [固定 DT-22 实测](../../evidence/java-syntax-2026-09-27/dt22-nested-annotation/report.md)。reader 已解析 `AnnotationDefault`，`class_source` 已用 `@interface` 写顶级头部。缺口在 `Engine::class_source_with_evidence` 的类家族装配：普通具名成员路径明确排除 annotation/interface，DT-13 只接 enum。JADX 的 `ClassGen.addInnerClass` 递归写子类型，`AnnotationGen` 管元素默认值；Jarde 可以借鉴递归装配，但不能仅凭二进制 `$` 名推导词法 owner。

## Goals / Non-Goals

**Goals:** 一个 Java 8 直接成员注解，根级输出 `Holder.A`，child 保持独立物理报告；默认值继续交给现有解析/字面量 writer；错误证据与停止原子拒绝。

**Non-Goals:** 多 child、两层以上注解、注解用值、任意 interface/member class、方法体内对成员注解的引用改写，或者把 DT-13 enum 证书放宽成 annotation 证书。

## Decisions

1. **来源关系复用现有精确事实。** 在 root 完整 `InnerClasses` 中只选择一个 `ACC_ANNOTATION|ACC_INTERFACE` 的直接 child，要求 source simple name 合法、`ACC_STATIC`、`root$Simple` 一致且没有重复/冲突行。按当前 selected environment 唯一读 child，调用现有 `child_relation_agrees` 核对 child self row、owner、名称和 flags。检查 Java 8 版本、class flags、无 `EnclosingMethod`、完整字段/方法表。这里的 `$` 只做一致性检查。普通成员家族的构造/捕获证书不适用，所以不能只删除其 forbidden-flags 门槛。
2. **源码装配复用 DT-13 的结构化 writer 边界。** 允许对 DT-13 `NestedEnumSourceText` 的通用“待嵌入声明文本与 derived ranges”形状做局部改名/提取，使一个根级 writer 接收已证的嵌套声明；enum 专属 `prepare_enum_constant_source_projection` 保持独立。注解 child 用当前物理 `ClassSourceDeclaration`、`ClassSourceMethod` 装配，不解析或裁切 `child.text`，不发明 `AnnotationDefault` 的新值路径。仍要求物理 writer 对 child 输出可由同一声明/方法记录复现，防止另一投影被静默丢弃。
3. **首片不改方法文本。** 冻结 `Holder` 只有普通构造器，root 没有对 `A` 的方法体引用。若 root/child 出现需要内部 `$`→`.` 改写的源码引用、额外成员关系或其他家族投影，先拒绝，而不是做全局字符串替换。`Runner` 的 `Holder.A` 是外部源码消费，不属于待改写的物理方法。
4. **报告和停止。** 根报告给出候选、物理 child 身份、证明/拒绝状态和嵌入声明范围的 owner/child 锚；child 自身查询仍保留物理名及默认值。所有额外 class/read/attribute/output work 使用同一个请求预算，递归深度固定为 1。只有全部事实与 writer 均完成时才替换 root text。预算/取消沿用现有 execution/diagnostic，不能转成“普通无候选”。

只复用本库的 reader、类来源选择和 writer；这里没有外部依赖的能力缺口。已有 DT-13 路径提供更贴近架构的装配边界，复制 JADX 的 `ClassGen` 代码或新增独立 backend 都没有必要。

## Risks / Trade-offs

- [普通成员家族与 enum 家族可能同时命中同一 root] → 首片要求没有其它待投影家族；writer 不能重建原 root text 时拒绝，不覆盖已有投影。复杂并存单独验收。
- [一个 child 物理报告中的 default 不完整] → 对每个元素核对 `AnnotationDefault` 声明与同轮已解析默认值；不把缺失当成“无 default”。物理类仍可独立查看。
- [报告类型与 DT-13 的 enum 名称耦合] → 只提取确有共同语义的声明文本/范围容器与关系核对；保留 enum 常量组与 annotation element 的各自证明，不建立猜测性的任意嵌套类型框架。
