## Why

Java 8 的 `outer.new Inner(arg)` 在字节码中以成员类构造器的隐式外层首参和调用点的空值检查实现。当前 Jarde 在 `new/dup` 后遇到该形状便拒绝，完整类源码缺少调用表达式；直接把物理首参当普通实参或照搬 JADX 的局部 `SKIP_FIRST_ARG` 门会改变绑定或求值顺序。

## What Changes

- 在已选物理环境能唯一确认成员类定义时，证明公开可访问的非泛型成员类关系、隐式首参和外层捕获字段，恢复调用点的 `outer.new Inner(args)`。
- 复用 `new@1` 的构造站点、SSA 来源和现有 Java AST，为已证明的成员构造增加限定接收者；将可证明的 null-check 纳入同一构造站点，保持普通实参与前置效果的顺序。
- 缺失目标定义、关系/参数身份不一致、null-check 或副作用时序无法证明时保留原有拒绝与来源，不靠 `$` 名称或首参数类型猜测。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 增加已证明的非泛型成员内部类限定构造调用恢复合同及其保守拒绝边界。

## Impact

影响选定环境中的目标类读取、`jarde-java` 的 `new@1` 私有证明、现有 `New` AST/发射器及 class-source 方法回收的事实交接。核心仍保持 host-agnostic，CLI 仅用于对照验证。首片不覆盖非公开访问关系、成员类声明投影、匿名/局部类、泛型内类签名与类级变量作用域；这些属于后续独立变更，不能用调用点恢复宣称完整嵌套类源码已可重编。
