## Why

DT-26 的固定 Java 8 样例中，Jarde 已识别 LambdaMetafactory 及捕获参数，却将编译器生成的 `lambda$...` helper 调用和声明写入完整源码。`javac` 重编该源码时生成同名 helper，发生符号冲突；原始源码和固定 JADX 输出均可重编、验证运行。证据见 [DT-26 对照](../../evidence/java-syntax-2026-09-27/dt26-lambda-capture/report.md)。

## What Changes

- 在 DT-25 的无捕获 helper 投影和整类引用清点已通过验收的前提下，将同一证明边界扩至一个稳定 `int` 参数捕获，以及 `this` 加一个稳定 `int` 参数捕获。
- 从站点的精确 bootstrap/handle、捕获栈值与 helper 参数/receiver 的映射证明创建时值，再把已证明的 helper 计算写入 lambda 体；仅在全类引用清点闭合时原子省略 helper 声明。
- 对捕获来源不稳定、值可能被重复求取、额外引用、方法体或预算证据不完整的情况拒绝投影，保留物理方法与可定位的拒绝信息。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：增加已证明捕获值的合成 lambda helper 源码投影与整类安全省略契约。

## Impact

影响 `jarde-java` 的 lambda 站点/方法体证明和 `jarde` 的类级源码装配；复用 DT-25 的 helper 身份、引用清点和原子提交边界。保留物理 class facts 与方法查询，不改 CLI/API 协议、依赖或 `jarde-jvm`。本项只覆盖固定 DT-26 首片，泛型 `removeIf`、字符串局部捕获、效果性捕获表达式及一般闭包转换留待后续验收。
