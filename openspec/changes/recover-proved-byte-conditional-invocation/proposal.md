## Why

固定的 JADX DT-28 smali 测试覆盖 `z ? (byte) 1 : (byte) 0` 作为 `byte` 参数的调用；JADX 完整源码可用 Java 8 重编，而 Jarde 因未证明 conditional 到 byte 参数的转换而拒绝整个方法体。相邻的直接 primitive 转换、移位和 byte-valued 条件返回在同轮对照中已通过，应补上这个窄的调用上下文证据。

## What Changes

- 当调用描述符要求 `byte`，且条件表达式两臂各自有足够的、可重放的 byte 窄化证据时，恢复该调用参数，保留条件求值一次及左右分支语义。
- 对缺少精确 primitive 参数描述符、任一臂转换未证或涉及其它 primitive 类型的条件实参继续拒绝；不推断任意 ternary 的窄化类型。
- 增加 DT-28 固定样例的原/JADX/Jarde Java 8 重编与 `-Xverify:all` 回归。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 增加有证据的 byte-valued 条件表达式作为 byte 调用实参的安全恢复要求。

## Impact

影响调用参数表达式恢复及其定向 Rust 测试；不改 CLI/API、外部依赖、通用条件表达式规则或其它 primitive 转换。冻结证据位于 `openspec/evidence/java-syntax-2026-09-27/dt28-cast-audit/`。
