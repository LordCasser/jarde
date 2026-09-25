## Why

Java 8 的字符串 `switch` 在 class 中被 javac 降为 hash/equals 判别和整数 `switch`。jarde 对已测输入虽能给出行为一致的完整 Java，却把编译器中间形态暴露为两层分派；JADX 能还原原来的 `switch (String)`。这是独立的源码形态质量缺口，优先级低于错值或不可编译的恢复。

## What Changes

- 在同一方法内完整证明 javac 风格的字符串分派映射时，呈现单个 `switch (selector)` 与字符串 `case`，保留原 arm、fallthrough、default、null 异常和 selector 只求值一次。
- 不完整、变形或受预算限制时沿用现有等价的两级 Java 或明确停止，不凭 `hashCode` 名称猜测原语法。
- 不扩展枚举 switch 的跨类映射、Java 14+ switch expression、任意反混淆或全局类型求解。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：增加经结构证明的 Java 8 字符串 switch 源码呈现要求。

## Impact

涉及已有 `Region::Switch`、SSA/常量池形状证明、switch AST 标签及 emitter、来源/预算与测试；不改变 core/CLI 分层、不增加依赖。635 B/10 行三方可执行基线见 `../../evidence/java-syntax-2026-09-22/string-switch/`。实施前须先固定更多正反例，不能把这一份行为等价样本误记为运行错误。
