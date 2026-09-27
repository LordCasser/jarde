## Why

CF-08 的五个循环切片已恢复，但固定 JADX `TestNotIndexedLoop` 对应的完整 Java 8 类仍在内层分支安全拒绝，无法重编。当前双出口循环证书把结果出口限定为单调用返回值、禁止体内调用，并要求内层汇合值立即消费；固定类同时含构造器出口、虚调用谓词和跨一层汇合的值转送。只放宽其中一个门槛不能追平这个固定测试。

## What Changes

- 扩展已有双出口 `while (true)` 的有界证明，接受一个由同次分配、构造和保存完成的对象结果出口，以及一个由数组元素经保序虚调用链决定的直接 break 出口；每个调用和结果只在原物理路径执行、消费。
- 复用已证明的二层 `if` 三来源内 join 与单块尾续接，再证明该结果经外层 null 分支的第二次合流后用于判空、`deleteOnExit` 和返回；不靠扩大局部词法范围掩盖不闭合的 SSA。
- 以固定 `NotIndexedLoop` 原/JADX/Jarde **完整类**的 Java 8 重编和四组运行结果作为正门槛，配物理边、对象身份、调用消费及双层合流的 verifier 有效负例；失败保持整体安全引用。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：扩充已受证嵌套双出口循环的对象构造、谓词调用和跨两层合流值的准确恢复与原子拒绝边界。

## Impact

主要涉及 `crates/jarde-java/src/region.rs` 的既有双出口循环和内外 join 证书；若实际 SSA/Builder 验收指出局部呈现阻碍，只在同一固定结构内修正 `build.rs`。复用现有对象构造和虚调用呈现，不新增 AST、通用 CFG 重写、依赖或独立局部绑定机制。异常边、不可约循环、任意深度分支与不受证的调用链不在范围。[物理阻碍记录](../../evidence/java-syntax-2026-09-27/cf08-endless-loops/not-indexed-fixed-triage-2026-09-28.md)及固定 JADX 算法/测试哈希为阶段前提；提案不表示固定类已恢复。
