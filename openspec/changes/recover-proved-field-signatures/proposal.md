## Why

Java 8 字段 `Signature` 可以声明 `List<String>`、通配符、类变量及其数组，而当前 Jarde 类源码只按物理 descriptor 写字段类型，丢失泛型反射和调用方的静态类型。JADX 会读取该属性并更新字段类型；[三方字段样本](../../evidence/java-syntax-2026-09-24/field-generic-signatures/)同时检验其字段类型与方法正文可能冲突的边界，不能把能解析的签名直接当作可发布源码。

## What Changes

- 复用 reader 已有字段 `Signature` 语法树及类级已证类型变量作用域，增加字段签名对物理 descriptor 的擦除证明。
- 在类源码接缝投影可写成 Java 8、且本类正文不会因字段静态类型改变而失效的字段声明；同一个字段的属性来源、注解和初始化边界仍可查。
- 签名未绑定、擦除不符、type-use 路径或本类字段访问缺少兼容性证明时，保留物理字段声明并记录局部拒绝。预算、取消和 essential/all 的正文一致性沿用现有契约。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已证明的字段泛型 `Signature` 可进入完整类源码，并可用泛型反射和调用方编译验证；不能证明的字段签名不得改变声明。

## Impact

修改 `jarde-reader::signature` 的字段事实证明、`src/class_source.rs` 的字段声明候选和 `src/facade.rs` 的属性读取/发布顺序；增补字段三方样本及 Java 8 重编验收。不增加 crate、依赖、全局泛型求解器或对目标 class 的运行期执行。字段经本类方法访问时的泛型正文推理、嵌套类类型拼写和缺失依赖的源码闭包留作独立扩展；本变更先交付可证明的字段声明子集。
