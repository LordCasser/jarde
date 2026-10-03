## Why

[嵌套泛型头巡查](../../evidence/java-syntax-2026-10-03/nested-generic-header-patrol/README.md)确认链头缺口：嵌套泛型类（`Z1$Box<U>`）的类头 Signature 投影被 `class_generic_source_unproved` 拒绝（nesting/kind/name 缺忠实头位证明）→ 成员 U 域链式失效（`jvm_signature_scope_unproved`）→ 折叠被 Signature 防线拒——三重阻断。顶层泛型类头已呈现（含 bound），既有 10+ 泛型切片程序成熟；本片把头投影扩展到嵌套位。

## What Changes

- 嵌套类（静态/非静态成员类）的类头 Signature 投影：nesting/kind/name 位证明（InnerClasses 行的源名与类名一致、kind 与 access_flags 对应、名字可拼——复用折叠片的逐行判据）后按 Signature 呈现 `<U>` 头（分离文本与折叠文本同判据）。
- 成员 U 域链解锁：头投影成功后 `U value` 字段等成员投影获得类型变量域（`jvm_signature_scope_unproved` 依赖的 scope 建立）。
- 折叠防线语义不变：带 Signature 的子类折叠仍需防线评估（头可投影后按既有防御语义重新评估——如实报告评估结果，若防线另有保守位保持拒绝并登记）。
- Z1/Box 家族：`Z1$Box` 头 `Box<U>` 呈现、`U value` 字段投影；同类绑定链（`generic_call_binding_unproved`）不在本片。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：嵌套泛型类按 Signature 投影类型参数头，成员投影获得类型变量域。

## Impact

`crates/jarde-reader/src/signature.rs`（头投影的 nesting 位证明）与消费层及测试；复用折叠片逐行判据。既有泛型切片与折叠防线零回退。
