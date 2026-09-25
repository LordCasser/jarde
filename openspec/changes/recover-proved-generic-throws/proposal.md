## Why

Java 8 的 `throws E` 会把泛型异常变量写入方法 `Signature`，而 `Exceptions` 属性只保存其擦除。当前 Jarde 和 JADX 1.5.6 都把已发布类变量的方法声明写成物理 `throws Exception`，使原本可编译的泛型调用方报检查异常错误，并改变 `getGenericExceptionTypes()`；[三方证据](../../evidence/java-syntax-2026-09-24/generic-throws-signatures/analysis.md)已经固定这一缺口。

## What Changes

- 对无正文方法，在类头已发布且方法签名与物理参数、返回和异常擦除逐位置一致时，把可证明为 `Throwable` 子类的类级异常变量写回 `throws` 声明。
- 对未绑定、擦除不符、异常类型约束或源码位置无法证明的签名，保留物理声明及局部拒绝；保留无泛型 `throws` 后缀时既有 `Exceptions` 输出。
- 用原源码、JADX 和 Jarde 的同一 Java 8 类及强类型调用方重编、反射、执行对照验收，并覆盖调试信息差异、预算与证据选择。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已证类变量的无正文方法泛型 `throws` 能进入完整类源码，调用方的检查异常语义与泛型异常反射和原 class 一致。

## Impact

主要修改 `src/class_source.rs` 中已有普通方法 `Signature` 候选；复用 `jarde-reader::signature` 的解析、类作用域与逐位置擦除证明，不新增 pass、依赖或协议。方法自有 `<X extends Exception>`、有正文方法的异常控制流、外部自定义异常继承证明和嵌套类作用域另作独立工作。
