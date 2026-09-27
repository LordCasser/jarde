# Recover a proved debug-generic diamond local

## Why

JADX 的 DT-20 `TestConstructorGenerics` 在有调试信息时恢复 `Map<String,String> map = new HashMap<>()`。固定原 class 的 `LocalVariableTypeTable` 确实保存了两个 `String` 类型实参，Jarde 已读同一方法的 `LocalVariableTable` 名称却跳过该泛型表，输出 raw `HashMap`。[三方对照](../../evidence/java-syntax-2026-09-27/dt20-diamond-local/report.md)显示语义运行一致而源级泛型/菱形丢失。无 debug 时 JADX 自身也采用 raw 构造和强转，不能猜测原泛型。

## What Changes

- 在既有同轮 `Code` debug 读取中保留唯一、完整的 `LocalVariableTypeTable` 签名，按 slot/范围/名称与 `LocalVariableTable` 和真实局部身份核对；沿现有预算与停止通路传到声明规划。
- 只对固定 Java 8 `Map<String,String>` 局部、准确 `new HashMap()` 无参分配及单次存储/读取模式，证明源类型与构造菱形合法后，一次决定声明类型并在现有 AST `New.diamond` 上写 `<>`。
- 用 `-g`/`-g:none` 双分支三方完整源码 Java 8 重编和验证运行验收；损坏/缺失/冲突 LVTT、slot 重用和不完整代码保持 raw/cast 或既有拒绝，不发布半个泛型局部。

## Impact

这是 DT-20 的单一 debug 可证切片。读取 LVTT 是必要的新事实：擦除后的指令、描述符和 `new HashMap` 本身均没有两个 `String` 实参。它复用已有 Code 读取、Signature parser、局部身份和声明规划，不建立泛型推断器或覆盖其它集合类型。
