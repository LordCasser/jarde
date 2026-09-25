## Why

Java 8 的 `try (T resource = null)` 仍编译出完整的资源关闭与异常抑制协议。当前 jarde 在进入 TWR 证明前把 `aconst_null; astore` 判作普通赋值，随后把合成 `Throwable` 处理器误写成用户 `catch`；独立整类输出因 `Object.close()` 无法编译。原 class 与 JADX 的整类执行一致，问题与已有非空资源恢复边界可分离。

## What Changes

- 在现有 TWR 守卫规则内有界接受常量 `null` 资源初始化；只有完整关闭、抑制、重抛、范围和顺序证明成立才认领，不放宽普通赋值后 `try/catch` 的分类。
- 从同一已证明 TWR 的 close 调用和当前类声明约束中恢复可编译的资源类型，仅覆盖首个正例所需的当前类 `AutoCloseable` 类型；来源不足时保守引用。
- 固定源码、class、JADX、jarde 整类重编译及 `-Xverify:all` 行为对照，包含普通 nullable 资源、多资源顺序和负面形状。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 已证明 `null` 资源初始化及其真实类型可在现有 TWR 结构中呈现；未证明时不得把合成清理写成用户 `catch`。

## Impact

影响 `crates/jarde-java` 的 TWR 守卫准入、资源声明值的类型选择与对应集成测试；复用现有 `Shape::Resources`、`ResourceDecl`、SSA 来源、预算和 source map。无需新 IR/AST 节点、外部层级解析器、StackMapTable 读取或依赖下载。Java 9 `try (r)`、任意外部 `close` 属主、用户 `catch` 组合及未完成的 return 尾部任务不在本 change 范围。
