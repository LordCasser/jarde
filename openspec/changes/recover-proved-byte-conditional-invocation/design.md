## Context

参见 [proposal.md](proposal.md) 的证据与动机。既有 `java8-recovery` 调用入口读取目标方法描述符，并在 `invocation_argument` 比较实参 `presented` 类型；`narrowed_constant` 只认单个 int 字面量，表达式 AST 已有 `Conditional` 与 `Cast` 节点。本项不改变描述符读取、条件树识别或全局 primitive 转换关系。

## Goals / Non-Goals

**Goals:** 仅在调用参数精确要求 `byte` 且条件树两臂均有局部证据可呈现为 byte 时，保留调用并写出可由 Java 8 编译的 byte 条件表达式。两个分支继续保有自己的来源锚点，条件及分支只执行一次。

**Non-Goals:** 不给 int 变量、未知生产者或超范围常量增加窄化；不推广到 short/char 或其它 primitive；不在赋值、返回、字段写入或普通条件值位置改变类型；不重建任意源级 cast，也不做生产者以外的方法体分析。

## Decisions

1. 在既有调用参数类型约束处处理，不增加解析阶段或新 crate。以调用 descriptor 的 `B` 为唯一目标事实，并只识别已有的 `Conditional` 表达式；不会从调用点反推未知条件树的类型。
2. 为两臂分别要求可复核的无损 byte 呈现证据：已有 byte-typed/cast 表达式可沿用；int 常量仅在值可由 byte 表示时允许在该分支加显式 byte cast。固定 classfile 的 `iconst_1`/`iconst_0` 不含 `i2b`，因此不能反推原源码曾写 cast；新增 cast 是由常量范围及目标 descriptor 共同证明的源码呈现。随后把两臂都写成 byte，令 Java 条件表达式自身成为 byte 类型，再满足调用上下文。相较对整个 int 条件结果强转，该放置可逐臂核对 producer/BCI，而不声称恢复了原始 cast 拼写。
3. 任一分支不满足上述局部证明、目标 descriptor 不是 `B`、条件结构有歧义或预算停止时，沿用现有整体拒绝和来源保留。不得只恢复调用头或丢弃失败的条件值。
4. `invocation_argument` 已有窄常量检查只支持直接 int literal；不改变该规则的范围，也不把 conditional 视为单个常量。条件分支的允许路径应是一个小型、专用于 byte conditional 参数的分支证明，不重定义通用 `narrowed_constant`。
5. 验收采用本目录固定的源码和原 class，对照固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f`：支持组保持 byte 返回条件、直接收窄/扩宽、移位和 int/long 推广行为；byte 参数调用组通过完整源码 Java 8 重编和 `-Xverify:all`。无证据的 int 分支、错误目标 descriptor 和超范围常量作为拒绝对照。

## Risks / Trade-offs

- 过度接受 int 表达式会发明窄化并改变值 → 只看条件 AST 的两条已证明分支，各自验证 byte 来源或范围内常量。
- 把调用目标 byte 误用作分支类型证据 → 目标只选定要验证的类型；每个 arm 独立满足证明条件，否则拒绝。
- 新 cast 丢失 BCI 或被报告为原始 cast → 沿用表达式的 `OriginSet` 与现有 derived source-map 约定，不伪造字节码中不存在的 cast 指令。

## Migration Plan

无数据迁移或外部 API 变化。先实现并运行定向正反例，再执行相关 Rust 测试及 OpenSpec strict 校验；若验证失败，回退限于本项的条件实参分支。
