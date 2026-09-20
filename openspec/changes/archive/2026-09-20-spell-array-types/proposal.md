## Why

数组 descriptor 直接泄漏进 Java 类型位置，产物在声明处就不是 Java：`byte[] local1 = arg0;` 恢复为 `[B local1 = arg0;`，`String[] local1 = arg0;` 恢复为 `[Ljava.lang.String; local1 = arg0;`；javac 在 `[B` 处报非法表达式开头（本次复核独立复现，真实样本亦为 `DSTU4145Signer.hash2FieldElement` 的 `[B local2 = reverse(arg1)`）。

根因位置由 [完成复核](../../completion-review.md) 指出：`crates/jarde-java/src/build.rs::spell_reference` 只对对象 descriptor 去掉 `L…;` 包装并替换 `/`，数组 descriptor 原样通过；而帧对引用的命名（`jarde-jvm` 的 `RefType::Named`）本来就是 descriptor 形式，数组也在其中，于是 `value_type` 把它当成 Java 源拼写交给局部声明。同一 crate 的 `lambda.rs::parse_type` 已经有正确的 descriptor→Java 拼写（`int[]`、`java.lang.String[][]`），而 `spell_reference` 自己的注释就写明了「第二份拼写必然漂移」。

## What Changes

- 类型位置的数组 descriptor MUST 拼成合法 Java 数组类型（`byte[]`、`java.lang.String[]`、`int[][]` 等多维形态），元素类型复用既有的对象名拼写；不能拼成合法 Java 类型的 descriptor MUST 使该区域拒绝，MUST NOT 原样输出。
- 实施 MUST 枚举呈现写类型的位置（局部声明，以及 `new` 的类型、静态 owner/`Path` 等呈现实际使用的位置），逐条记录可达性与覆盖证据；MUST NOT 只改声明路径就声明整类关闭。
- 验收：原始数组、引用数组与多维数组在类型位置的受控 fixture（含已实测的声明位置与呈现实际使用的其它位置）、产物声称 Java 时 javac 接受该包装器、恢复缺陷的变异，以及合法非数组类型逐字不变的正向对照。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：新增「类型位置拼成合法 Java 类型，否则拒绝」的要求（主规格此前没有覆盖 descriptor→类型位置拼写的要求）。
- `recovery-validation`：新增「类型位置的产物在成员自己的签名下编译，或以其拒绝边界验收」的验收要求。

## Impact

实现落在 `crates/jarde-java/src/build.rs`（`spell_reference` 及其类型位置调用点；与 `lambda.rs` 的既有拼写合并成一条或明确由测试固定两处一致）；不改 `value_type` 对非数组类型的决定，不改 `emit.rs` 的打印规则与任何 AST。本 change 与 [group-call-receivers](../group-call-receivers/proposal.md)、[type-boolean-contexts](../type-boolean-contexts/proposal.md) 都改 `crates/jarde-java`，三者 MUST 按 [路线](../../roadmap.md) 顺序**串行**实施（本 change 为第 3 个），彼此不依赖对方代码。

交付包含受控 fixture（源码、class 字节、来源 README、`tests/fixtures/README.md` 登记、fingerprint 再生成、reader fixture census 更新）、精确文本回归、javac 编译对照、变异与正向对照。反例 MUST 在修正前先记录（命令、正文、javac 的拒绝信息）。

非目标：不新增 crate、依赖或 verifier；不做泛型、擦除或数组协变推理；不新增类型模型或 `Type` 变体；不重开 R8/R9 与已归档的递归界、打印修正；不声称一般语义等价，也不声称覆盖整类类型缺陷；不做性能工作（`optimize-demand-workloads` 保持 0/22）；不修 body 解码重新解析类的债务。

当前仅完成修正规划，实施任务全部待办；历史归档与既有验证记录保持原状。
