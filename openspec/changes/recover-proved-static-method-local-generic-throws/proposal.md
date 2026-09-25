## Why

Java 8 静态直接返回参数的方法 `<T, X extends Exception> T echo(T) throws X` 在 class 的 `Signature` 中保留了方法自有异常变量。本地 [三方对照](../../evidence/java-syntax-2026-09-25/static-method-local-generic-throws/analysis.md)显示 JADX 1.5.6 输出 `throws Exception`，Jarde 则整体退回物理 `Object` 方法；两者的重编类都改变反射结果，且原合法强类型调用方无法重编。

## What Changes

- 在已有静态直接参数返回的同轮 AST/SSA 证明成立时，同时恢复方法类型参数、`T` 参数/返回及严格限定的方法局部 `throws X`。
- 复用 reader 的方法 `Signature` 解析、局部变量作用域与异常擦除验证；保留无法证明的类层级、正文或调用绑定上的局部物理回退，不引入新解析器或跨方法推断。
- 以 `-g`/`-g:none` 下原 class、JADX、Jarde 的完整类重编、反射与显式类型调用验证，并独立记录负例。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 已证静态直接返回方法能保留方法局部泛型异常的源级声明与反射语义。

## Impact

仅触及完整 class-source 的泛型方法头投影及其测试/证据；reader grammar、方法体候选、CLI 协议与 crate 依赖保持现状。首片限顶层非泛型普通类、Object 直接子类、无接口、单个方法局部 `throws X` 且 `X` 有已知 JDK Throwable 根作为唯一 class 界；复杂正文、自定义异常界、继承/隐藏和本类调用绑定单独处理。
