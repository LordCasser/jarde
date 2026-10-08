## Why

当前字段写证明把字段声明类型当成每个访问点的选择类型；raw receiver 写入 `T` 字段时实际选择擦除类型，因此合法字段 Signature 被误拒。现有巡查已证明 raw 参数和保留 raw local 可保留字段 T，而被 renderer 折成 this 的别名不能据此放行。

## What Changes

- 在既有 class-source 恢复路径保留按物理写入点关联的实际 AST receiver 事实；支持直接 raw 参数和实际发射 raw 声明的 local，不从 SSA 别名猜测源类型。
- 方法投影完成后，只使用同一物理方法实际发布的参数类型决定 receiver 是否 raw；字段声明与该访问点的选择类型分别检查。
- raw 同类实例字段访问按字段擦除类型验证封闭 RHS 来源；this、已发布参数化 receiver 及未证明 receiver 保留现有泛型写证明边界。
- 冻结原 class/JADX/Jarde 在真实 JDK8/23、debug/no-debug 四腿的完整重编译、JVM 行为与泛型反射，并保留拒绝和 JADX 失败。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 同类实例字段 Signature 投影须区分实际 raw receiver 的字段选择擦除与字段泛型声明。

## Impact

影响 jarde-java 既有 Program 构建后的同次恢复事实、facade 同类字段使用清单与已发布参数事实，以及 class_source 字段提交；不增加 pass/parser/IR/fixpoint 或依赖。前置条件是主线 `564e22c1` 已交付的字段写来源和构造参数证明。普通泛型调用适配、this 委派、参数化 receiver 通用代换、复杂别名/phi/继承字段选择属于独立后续片，不借本片扩大方法 Signature 发布。
