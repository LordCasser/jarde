## 1. 共享 true 的证明与发射

- [x] 1.1 在现有 `ShortCircuitValue` 证明中按 decode target 与 canonical 正常边识别外层跳转直达 true、顺序边到内层；核对两个生产者的精确前驱、唯一消费块及原有无异常边/入口/效果条件，证明结果仅记录发射所需的边极性。验证：冻结 `SharedTrueShortCircuit.assign(Z)V` 在 BCI 1/7、10/14、15 证明通过；原 shared-false Base 仍通过，非 1/0、第二次写入和错误字段身份仍拒绝。架构师独立运行共享 true 聚焦测试 1 项通过，并核查 [完整 CLI 报告](../../evidence/java-syntax-2026-09-25/short-circuit-shared-true/jarde-after-report.json)的 `structured`/单次字段写入；额外入口及预算停止仍归 2.1 的最终复核。
- [x] 1.2 用现有 `Conditional` AST 原子发射共享 true 的字段赋值，外层顺序边才求值内层 RHS，外层跳转边直接取已证明的 1；沿用 `putstatic Z` 转换与 source map，消费块后缀保持原顺序。验证：`assign` 正文只写一次 `result`，BCI 0/1/4/7/10/11/14/15 均有来源且无 `@bytecode`；架构师将 Jarde 完整类重新编译并在 JVM 验证下运行，左真 `calls=0`、左假 `calls=1`，与原 class 及 JADX 三方逐行一致，见 [复核](../../evidence/java-syntax-2026-09-25/short-circuit-shared-true/analysis.md)。共享 false Base 回归仍归 2.2 的最终复核。

## 2. 拒绝边界与三方行为验收

- [ ] 2.1 用各自新恢复报告测试误极性、第二写入/消费者、非 1/0、错误字段身份及无法嵌入的独立效果；拒绝时两分支、生产者、字段写入、后缀同时在 quote 与 source map，方法质量不为完整 structured。验证：聚焦负例及原 `recover-conditional-field-writes` 拒绝矩阵通过；字节补丁必须先核对 JVM StackMap，可验证运行证据与纯 IR 稳健性证据分开标注。
- [x] 2.2 将冻结类的原始、JADX 1.5.6、Jarde 完整源码分别用 Java 8 模式编译，临时 runner 在 `java -Xverify:all` 下对比 `left=true,result=true,calls=0` 和 `left=false,result=true,calls=1`；重跑普通 `?:`、共享 false Base、非规范 `Z` 控制。验证：架构师独立将三份完整类重编并运行，两行逐字一致，见[对照](../../evidence/java-syntax-2026-09-25/short-circuit-shared-true/analysis.md)；又用当前 CLI 重新生成并重编 Base，原/Jarde 都输出 `observed=captured-value`、`visibleDuringSuper=true`，左假控制原/Jarde 均为 `false-result=false,calls=0`、`true-result=true,calls=1`。普通条件值 2 项及 40 行非规范 `Z` JDK 控制通过。JADX 只记录事实，不作正确性判据。

  2.1 已补的受控子集：共享 true 的双写入 Java 8 class 与从**单写入** class 改 `iconst_1`→`iconst_2` 的补丁均经 `java -Xverify:all`，fresh recovery 保留全部 quote/source-map BCI，后者精确拒绝为 `Producer`；原 shared-false 独立效果、错误字段、非 1/0 与双写入矩阵也通过。曾作“额外入口负例”的直接分支改写实际构成可证明 OR，现为正向极性测试。真正异常入口及误极性负例还需独立控制，故 2.1 不勾选；三个判断共享生产者的 [独立缺口](../../evidence/java-syntax-2026-09-25/short-circuit-chain-shared-true/analysis.md)不混入这项双测试发射。
- [ ] 2.3 运行 `cargo fmt --all -- --check`、聚焦和相邻回归、适用的 Clippy、`openspec validate recover-disjunctive-field-writes --strict`；在独立 verification 文档列出精确通过/失败及未覆盖边界。验证：所有本 change 的已完成任务才勾选，不混入匿名类 5.3 或异常边 Region 债务。
