## Why

固定 JADX 的 `TestDeboxing` 对六种字面量装箱给出 primitive 源码断言，而 Jarde 保留显式 `Wrapper.valueOf(...)`。EM-25 冻结 Java 8 回放三方均可重编并运行一致，说明这里首先是源码质量差距；但 Java 8 对部分包装类型的自动装箱身份并无与显式 `valueOf` 等价的普遍保证，不能为追平文本而扩大重写。

## What Changes

- 仅当准确 `Boolean.valueOf(boolean)`、`Integer.valueOf(int)` 或 `Character.valueOf(char)` 的字面量处于 Java 8 规范和标准库共同保证身份的范围，并直接成为对应包装类或 `Object` 返回值时，恢复 primitive 源码形式。
- 保留调用与返回的物理来源和唯一消费者证明；不能证明 owner、descriptor、字面量范围、返回转换或身份边界时保持显式调用。
- 将固定 JADX 的 `Byte`、`Short`、`Long` 简写列为当前不采纳的源码形式：它们在本机 `javac` 运行一致，不足以证明任意 Java 8 编译器的引用身份等价。
- 变量流拆箱、任意调用参数、重载 cast、`instanceof` 收窄及通用装箱转换不纳入本切片。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：只在语言规范和标准库共同证明装箱身份的直接返回字面量中，恢复不改变对象身份的 primitive 源码形式。

## Impact

仅涉及 `jarde-java` 方法正文的有界表达式呈现、定向测试与 EM-25 对照证据；不新增 API、依赖、通用转换 pass 或 AST 节点。前提是同次方法/类证据具备准确调用 owner、名称、descriptor、primitive 实参和返回上下文。[冻结审计](../../evidence/java-syntax-2026-09-27/em25-boxing/report.md)与 [JLS §5.1.7](https://docs.oracle.com/javase/specs/jls/se8/html/jls-5.html#jls-5.1.7)说明边界。
