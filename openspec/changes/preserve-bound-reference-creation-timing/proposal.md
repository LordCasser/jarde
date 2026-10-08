## Why

root 完整 JVM 对照证明合法 major52 `NoCheck.make(Thread)` 的 LambdaMetafactory 虚拟句柄可捕获 null：原类创建成功、调用时 NPE；现有 renderer 输出 `arg0::start` 后重编却在创建时 NPE。构造参数片 e47e7940 暴露这个既有 lambda 证明缺口，必须先修复再验收，不能凭 javac 通常插入 null-check 推断所有合法字节码都如此。

## What Changes

- 直接绑定方法引用与现有 adapter 使用同一个 receiver_nonnull 证明；无法证明创建时等价就拒绝引用输出，保留完整 effect/origin。
- builder 复用同轮已证明的 receiver tail、真实 entry this、完成分配与 frame未知时的直接 `CONSTANT_String` 等明确非 null 事实；不能把 static local0 当 this。Class literal 的 `dup; getClass; pop` 形状仍保守拒绝。
- 独立验证 standalone 与 constructor position 的无 check 反例、真实 javac check 形及已证明非null正例。
- 前置条件是 frame/SSA 和既有lambda bootstrap/类型证明；非目标：补任意可空receiver的创建时检查机制、自动改写lambda并移动capture、泛型投影及外部层次推测。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 绑定方法引用创建阶段的null行为必须有同轮证明。

## Impact

`crates/jarde-java/src/lambda.rs` 既有receiver_nonnull gate与build.rs的事实提供；无需新IR/pass/planner/bool字段或依赖。此片作为recover-functional-constructor-arguments的前置，单独提交与验收，构造主片暂未合入。反例与原/错误输出在 `openspec/evidence/java-syntax-2026-10-08/bound-reference-creation-timing/`。
