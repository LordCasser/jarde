# DT-27：静态、绑定实例与构造器方法引用

## 范围与对照

本审计固定 JADX checkout 为 `/Users/lordcasser/workspace/testzone/jadx`，commit `2fb1b16386941660fda07e9017285aec40fcb37f`。对照测试是 `jadx-core/src/test/java/jadx/tests/integration/java8/TestLambdaStatic.java`、`TestLambdaInstance.java` 与 `TestLambdaConstructor.java`。生产路径为 `jadx-core/src/main/java/jadx/core/dex/instructions/invokedynamic/CustomLambdaCall.java` 的 `buildMethodCall`：它读取 implementation handle 与 effective method prototype，在参数类型列表相同且未内联合成方法时标记 `useRef`。`InsnGen.makeInvokeLambda` 选择引用输出，`makeRefLambda` 根据静态、实例或构造器 handle 写出限定符和 `::`。

固定夹具只覆盖三个独立的 Java 8 形态：`Math::abs` 适配到 `IntUnaryOperator`，`this::number` 绑定到 `IntSupplier`，以及 `RuntimeException::new` 适配到自定义顶级 `Maker`。这些源文件都没有泛型类型声明；`Runner` 输出 `2:-3:RuntimeException`。`replay.py` 从干净源码重编 class，打包四个夹具类型，分别调用固定 JADX 与指定 Jarde CLI 生成完整源码，并把三个版本连同同一 Runner 用 `javac --release 8` 重编、`java -Xverify:all` 执行。JADX/Jarde 文本还必须包含三种精确引用拼写，且不得包含 `lambda$` helper。

## 结果

| 输入 | Java 8 编译 | `-Xverify:all` 运行 | 标准输出 | 语法断言 |
| --- | ---: | ---: | --- | --- |
| 原始夹具 | 0 | 0 | `2:-3:RuntimeException` | 原始源码含三种方法引用 |
| JADX | 0 | 0 | `2:-3:RuntimeException` | `Math::abs`、`this::number`、`RuntimeException::new`；无 `lambda$` |
| Jarde | 0 | 0 | `2:-3:RuntimeException` | 三种精确引用均保留；无 `lambda$` |

冻结输入、三方完整源码输出、四个 Jarde JSON 报告、结构化结果与 SHA-256 清单位于本目录。报告的运行时 `usage.elapsed_millis` 归一化为 `<elapsed>`，预算上限和其它计数保留，以便重复回放的哈希稳定。使用 `JARDE_CLI=/tmp/jarde-dt21-audit-target/debug/jarde-cli python3 replay.py` 可重放；脚本检查 JADX commit 与 clean 状态，并使用临时目录编译，不在仓库创建 Cargo target。

## 边界

这只是 DT-27 的一个非泛型隔离切片，结论为**部分已测、待扩验**。固定 JADX 测试还包含泛型 `Supplier`/`Function` 目标、静态 `Integer::parseInt` 与函数目标差异、`Object::toString`（对应断言仍为 TODO），以及 fallback 形态；本夹具没有验收这些情况。它只证明一组成功的重编/运行，不覆盖多种重载选择、参数/返回适配、装箱拆箱、异常与捕获阶段语义，也不证明所有方法引用都应恢复为 `::`。不得据此宣称整个 DT-27 已追平。
