## Why

Java 8 的有正文方法 `<X extends Exception> void run() throws X {}` 会把方法自有类型参数和异常变量一同写入 `Signature`。原 class 的显式 `<RuntimeException>` 调用合法；本地 JADX 1.5.6 保留 `<X>` 却丢 `throws X`，当前 Jarde 整体回退，二者都使同一强类型调用方编译失败。[三方证据](../../evidence/java-syntax-2026-09-24/body-method-local-generic-throws/analysis.md)已区分这两个缺口。

## What Changes

- 在已证空 `void` 正文的最小子集，恢复方法自身 `<X>`、其合法界和同作用域 `throws X`，完整保留构成反射和静态异常类型检查的两个位置。
- 保持 reader 的 Signature 解析/逐位置擦除、现有同轮空正文候选和原子声明发布；对非空正文、伪造签名或未证明的 Java 绑定局部回退。
- 用有/无调试表的完整类、反射、合法强类型调用与边界负例验证。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 有正文方法在严格空效果证明下同时保留方法自有泛型类型参数及其 `throws` 变量。

## Impact

涉及受控 class-source 方法声明投影及同轮正文候选消费，不引入 crate、依赖、CLI 协议或新 Signature parser。本轮限定顶层非泛型类、直接继承 Object、零接口、无参 `void` 空正文、单个方法变量与单个 `throws X`；不覆盖带参数/返回值的泛型方法、非空正文、自定义异常界、继承/覆写或跨类调用绑定。
