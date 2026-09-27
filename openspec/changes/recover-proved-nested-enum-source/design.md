## Context

见 [proposal.md](proposal.md) 与 DT-13 [实测报告](../../evidence/java-syntax-2026-09-27/dt13-enum-shapes/report.md)。单类声明 writer 已从 class flags 和 interfaces 写出简单 enum header；`enum_constants::prove_group` 证明一个物理 enum 的常量组。两者都不证明不同 classfile 的词法父子关系。具名成员类已有 class-source family 入口，但其来源、调用和捕获投影不自动证明 enum 组可嵌入。

## Goals / Non-Goals

**Goals:** 对物理 class family 中有完整 InnerClasses 证据的两级嵌套 enum 恢复源声明位置，并保留每个 child 的独立物理事实。未解释的不确定性采用原子拒绝。

**Non-Goals:** 改 enum implements header；处理 DT-11 的构造实参、DT-12 的常量专属匿名体、任意非 enum member types，或将 CLI `class-source` 定义为完整项目 decompiler。

## Decisions

- 将成员关系判定放在类级装配边界：使用当前请求的 selected environment 解析 owner 与 child，要求定义身份唯一，并将 classfile 两侧的 InnerClasses row 按 owner、inner class、simple name 与 flags 精确核对。仅 `$` 命名和单侧名称关系不足以投影。
- 对每个 child 仍调用现有物理 class-source 准备路径，并要求其 enum group 为完整 Proved；根级 assembler 只在整个两级家族通过后生成一份带嵌套声明的文本。不得以对 child 的手写简化替代 enum proof。
- 复用现有 `ClassSourceMemberFamily` 的同次来源、物理 child 报告、根报告投影状态和 derived anchors 思路；不把当前普通成员 family 的候选条件直接放宽成“所有有 InnerClasses 的 class”。为嵌套 enum 明确表示嵌套层级所需的最少私有数据，避免新建通用 decompiler/backend。
- 根级投影产生的声明范围锚定 owner/child 的物理 class definition 与 enum 常量组物理成员；原有 child 报告和方法级 origin 保留。任一级拒绝/停止，整个相关子树不写入 root text；各物理类仍独立可查询。
- `enum implements I` 继续由现有 class declaration writer 根据 enum kind 与接口事实输出；不需复制到成员投影机制。`TestEnumsInterface` 中的方法覆盖匿名常量体与本 change 分离。

## Risks / Trade-offs

- [InnerClasses 表不总能单独反映全部源码关系] → 同时验证 owner 和 child 的精确 row、解析身份及源名/可访问条件；缺失时拒绝，不以二进制名补猜。
- [独立 class-source child 文本可能带非项目级 fallback 标记] → 不直接拼接 CLI 展示文本；在既有 class-source 内部结构上投影已证明的声明与 enum 常量，仅保留物理方法结果语义。
- [预算或取消发生在递归家族中] → 在声明提交前完成整棵受支持子树证明与 charge/poll；停止不返回半份嵌套文本，停止状态与每个物理报告继续可见。
