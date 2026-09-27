## Why

DT-22 的 `AnnotationDefault` 值已经能忠实恢复，但 `Holder.A` 仍被输出为独立 `Holder$A` 顶级注解。固定原始源码与 JADX 的完整 Java 8 源码均可用原 API consumer 重编并验证运行；Jarde 在 `Holder.A` 编译失败。证据见 [DT-22 报告](../../evidence/java-syntax-2026-09-27/dt22-nested-annotation/report.md)。

## What Changes

- 对直接成员 `@interface` 使用双方 `InnerClasses`、唯一物理定义和完整 child 声明事实证明词法 owner，再把声明写入根类型。
- 复用 DT-13 已有的结构化嵌套声明装配与物理 child 报告边界；将公共词法关系和 writer 输入从 enum 专属常量组证明中分离，不建立第三条平行家族路径。
- 关系、annotation kind、成员表或默认值不完整，以及预算/取消停止时，整棵待投影子树不发布；顶级 `$` 注解保持顶级。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：增加已证明的直接成员注解类型在词法 owner 中恢复的要求。

## Impact

影响 `jarde` 类级来源关系证明、根级源码 writer 和嵌套声明报告；不改变 reader 的物理身份、`AnnotationDefault` 的值解释、CLI 协议或依赖。首片只支持一个直接成员注解与本目录的顶级 default 控制；多 child、任意深度和注解使用值不在本项。
