## Why

循环 try/catch 的完整类对照已修复 retry 区域，但四组 retry-v2 的 `exc()V` 仍遗漏 `new AssertionError("uncaught-error:" + c)`，使应逃逸的 Error 变成正常返回。现有 concat 拒绝位于 append 之间的 `getfield`，继而使既有 constructor composition 无法识别真正的 String 实参；这是独立表达式缺口，不能由 retry-v3 替代验收。

## What Changes

- 为已有字段证明支持的、原位单次消费的实例字段读值补齐 concat operand 证明，保持成员身份、转换、顺序、次数和异常范围。读取可以是 volatile；许可来自保持原操作，而非认定字段纯净。
- 复用现有 concat / field / init / AST / source-map，不放宽嵌套构造器必须提供真实实参的不变量。新增证明使用现有预算并原子提交。
- 保留四组 retry-v2 原 class、JADX 与未修改 Jarde 整类对照，以 Error 类型、消息、事件和调用次数验收；增加字段读值及效果/边界/停止反例。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 增加有序实例字段读值作为拼接实参、并组合为真实构造实参的恢复要求。

## Impact

影响 `jarde-java` 现有 concat 规划入口及字段事实传递、永久 fixture 与测试。无公共 API、依赖、CFG 或新 pass。前提是已有 SSA、精确字段呈现事实、物理异常表与 constructor composition；不声称这项能力已经实现。

本变更独立于 `recover-loop-body-try-catch` 的 region 修改；父项完整类门仍必须通过 v2。非目标包括任意 receiver 表达式、跨块字段拼接、getstatic 常量折叠/类初始化证明、一般表达式效果调度，以及旧 concat 扫描器计费债务的全面重构。
