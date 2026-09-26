## Why

Jarde 已能依据 `ACC_VARARGS` 打印方法声明，却把同一类中的简单调用保留为 `test1(new int[]{1, 2})`。JADX 的 `TestVarArg` 将这种内联数组恢复为 `test1(1, 2)`；当前 Jarde 已有数组构造闭合证明，只缺调用目标绑定和展开安全门。

## What Changes

- 在调用处仅对已证明的目标声明和直接内联数组初始化展开最后一个可变参数，保持实参求值顺序与物理来源。
- 普通数组参数、未解析或歧义目标、可能改变重载绑定的同名方法、以 `null`/数组作为唯一元素等情况保留显式数组调用。
- 用 Java 8 正反例和原/JADX/Jarde 三方输出、重编及执行验证，不改变已有 `T...` 声明恢复或通用数组字面量恢复。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：在调用目标和数组初始化均闭合且源级绑定不变时，允许可变参数调用的源级展开；其余调用保留显式数组。

## Impact

影响 `jarde-java` 的调用目标证据消费与 `Call` 表达式装配，以及同类成员声明事实的只读使用。`jarde-jvm` 的 IR、物理 class 事实、CLI/API schema 和依赖不变。
