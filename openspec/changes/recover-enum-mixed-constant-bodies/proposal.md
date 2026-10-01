## Why

[混合形态巡查](../../evidence/java-syntax-2026-10-01/inner-enum-args-patrol/mixed-bodies/README.md)确认 enum family 的剩余组合：常量同时带构造实参与专属体（`ADD(1) { … }`，`TimeUnit` 式高频写法）。常量体折叠（constant-bodies 义务通道）与任意实参折叠（arbitrary-arguments grammar）各自已闭合，组合形态失败在其交叠——带体常量的匿名子类 ctor 为 `(String,int,<用户参>)`、桥为 `(String,int,<用户参>,$1)`，超出现有义务证明的零源参形状（`p.Combo` 固定复现，逐字段降级）。

## What Changes

- 常量体义务证明参数化：子类/桥 ctor descriptor 接受 arbitrary-arguments 已建立的参数集（int 族/String/getstatic 限定名/null，≤3 参）；委托链逐参转发证明（子类 ctor 依次 aload/ldc/getstatic 转发 + `aconst_null` marker）；常量步骤对齐。
- 呈现 `NAME(<args>) { <body> }`——实参拼写复用 arbitrary-arguments 呈现，体复用 constant-bodies 呈现。
- 纯带体（零参）、纯带参（无体）两能力输出逐字不变（diff 断言）；义务不满足（转发链断/实参种类不符）整组保守逐字段。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：构造实参与常量专属体的组合形态折叠为 `NAME(args) { body }`。

## Impact

根 crate `src/enum_constants.rs`/`src/facade.rs`/`src/class_source.rs` 的常量体义务参数化与呈现及测试；两个既有证书的交叠组合，无新机制。既有 enum 全部证书零回退。
