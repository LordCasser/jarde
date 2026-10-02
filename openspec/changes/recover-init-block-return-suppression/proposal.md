## Why

[初始化块巡查](../../evidence/java-syntax-2026-10-02/init-block-return-patrol/README.md)实锤：静态初始化块（`<clinit>`）呈现发射尾随 `return;`——JLS §8.7 禁止初始化块含 return，javac 拒编 → **含静态块的类整类不可重编**（F2 固定复现；构造器/方法中同形合法且为既有忠实呈现，不受影响）。

## What Changes

- 初始化块语境（静态 `<clinit>` 与实例初始化块）的语句发射抑制 `return;`（尾随与块内冗余形）；仅初始化块呈现通道，方法/构造器呈现零变化。
- F2 整类可重编且行为一致；既有 `<clinit>` 呈现（含 enum 折叠家族）除 return 抑制外逐字不变；非 return 语句不动。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：初始化块呈现不含 return 语句，输出合法 Java。

## Impact

仅 `crates/jarde-java` 初始化块呈现通道及测试；无证明层改动。既有方法/构造器呈现零回退。
