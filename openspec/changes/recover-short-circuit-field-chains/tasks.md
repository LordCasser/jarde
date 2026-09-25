## 1. 有序 Region、证明与发射

- [x] 1.1 将私有 `ShortCircuitValue` 的固定双测试字段改为按执行顺序的有界列表，在普通嵌套 `If` 前一次认领共享生产者、另一生产者与唯一静态字段消费块；仅在完整闭合时提交所有权，未证明值的整组指令默认完整引用。验证：冻结 `ChainOrField.assign(ZZ)V` 不再将 BCI 19 误判为循环，全证据 source map 均覆盖 0/1/4/5/8/11/14/15/18/19/22；原两测试 shared-false/shared-true 与 exception-edge Region 回归通过。未闭合额外入口单列 2.1。
- [x] 1.2 将既有短路字段值的 CFG/SSA 证明逐测试推广：解码跳转、精确前驱、局部值依赖、1/0 生产者、唯一同槽 Phi、唯一 `putstatic Z`、无真实异常边及预算/深度拒绝。验证：冻结三测试 OR 的 BCI 1/5/11、14/18、19 正例证明通过；原双测试 `&&`/`||`、重复消费者、非 1/0、额外效果拒绝回归通过；三测试链第二消费者和非 1/0 亦拒绝。三测试链独立效果单列 2.1。
- [x] 1.3 用现有 `Conditional` 按逆序构建一次字段赋值，保持每级顺序边才执行后继测试，消费块后缀原位发射，失败整体引用。验证：新完整类 `assign` 只含一次字段写入、无 `@bytecode`，source map 覆盖全部必要 BCI，左侧两级为真时 RHS 调用次数为 0；见 [独立验收](verification.md)。

## 2. 拒绝矩阵与独立验收

- [ ] 2.1 用各自 fresh recovery 的控制核验第二消费者、非 1/0、独立测试效果、额外入口和真实异常边；不能正面证明的局部保持完整引用，不能重访消费点而遗漏生产者。验证：逐指令 quote/source map、Fallback 质量与无虚假字段赋值的断言通过；直接字节补丁先用 StackMap/JVM 验证区分可执行证据与纯 IR 稳健性证据。

  已验收的子集：第二消费者 Java 8 class 四路径可验证，三测试非 1/0 补丁的 fresh proof 拒绝为 `Producer`；[真实异常边 class](../../../tests/fixtures/p3-conditional-values/short-circuit-exception-chain/README.md)五路径可验证，11 个受保护指令整体引用。独立效果尚未固定。[真正额外入口的 32 路径反例](../../evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry/analysis.md)已复现当前来源与所有权缺口，因此 2.1 保持未勾选；该例含内层三元条件，不能错误归作已证明的纯同极性链。
- [x] 2.2 冻结 class 与源码字节一致性，原 class、JADX 1.5.6、Jarde 的完整类分别用 `javac --release 8` 重编，并在 `java -Xverify:all` 下对照四条字段值与 `calls`；复跑 Base 共享 false、两测试共享 true、左假、非规范 `Z` 与异常拒绝控制。验证：Jarde 与原 class 的四行逐字一致，报告为完整结构化且每个测试/生产者/写入 BCI 可追；JADX 只记录事实，不作正确性判据，见 [证据](../../evidence/java-syntax-2026-09-25/short-circuit-chain-shared-true/analysis.md)。
- [x] 2.3 运行 `cargo fmt --all -- --check`、相关定向/邻近测试、适用 Clippy 与 `openspec validate recover-short-circuit-field-chains --strict`，在独立 verification 文档记下命令、实际通过/失败、剩余边界和磁盘清理。验证：命令与旧 Clippy 限制见 [独立验收](verification.md)；2.1 仍未完成，不顺手修类型或匿名类架构债务。
