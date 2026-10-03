## Why

已有的 `Signature` 语法与擦除证明能形成泛型方法、字段声明候选，但类源码层只要在常量池看到同类同名 `Methodref` 或同类同名同描述符 `Fieldref` 就整项拒绝。固定 Z1 家族分别触发 8 个 `generic_call_binding_unproved` 和 4 个 `field_generic_body_unproved`；常量池候选并不能说明实际使用点或 Java 源码重载绑定，因此需要在现有类级装配中证明真实使用关系后才发布候选。

## What Changes

- 对选定类中实际执行的同类方法调用与字段访问建立有界使用点清单，分别核对物理目标、当前源码表达式的静态类型、声明候选及可能竞争的同名成员；未使用的常量池条目不再单独阻断投影。
- 仅在受影响使用点均可证明仍绑定原物理目标、字段读写和正文类型均可表达时，原子发布对应泛型声明；否则保留现有物理记录、擦除声明和明确拒绝原因。
- 用无重载、相邻重载、字段读写及不完整扫描等正反例验收 Java 8 重编、原 class 行为、泛型反射、来源和预算停止语义。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：同类引用存在时，泛型方法与字段声明可在实际使用点及源级绑定得到证明后呈现；无法证明时必须拒绝，不能凭擦除相同宣称绑定不变。

## Impact

前置是 `recover-nested-generic-class-headers` 对嵌套类头和类型变量域的验收；本变更复用已交付的 reader `Signature` 解析、类级成员读取与方法恢复证据。预计触及 `src/class_source.rs`、`src/facade.rs` 及定向测试，不新增 crate、依赖、公开 IR 或全局 XRef。Z1 家族用于诊断回放；其 lambda、非静态内部类折叠等独立缺口不计入本变更的整类成功。外部类调用、运行时派发、任意泛型类型推断、现代 Java 源码语法、语料门禁维护和文档总览校正均不在本范围。
