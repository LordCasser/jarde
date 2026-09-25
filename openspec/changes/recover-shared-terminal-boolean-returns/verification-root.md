# Root 独立验收（2026-09-25）

用独立的 `/tmp/jarde-two-exit-root-target` 构建 `jarde-cli`，二进制 SHA-256 为 `7e4107e7d8d6c0d3c22bf9d13a46f3105e7c59ed6a18c1b4d6db8cfbf888b984`。输入正例 [`TernaryInIfProbe.class`](../../evidence/java-syntax-2026-09-25/ternary-in-if/TernaryInIfProbe.class) SHA-256 为 `1d633a6b4b6d1408d6e6039ac81302ec175217bac7bd610da32d77f2c17041d1`。CLI JSON 与完整类 Java 留在同一证据目录的 `root-after-positive.*`；`bothMatch` 是 `structured/java`、零 fallback、一个 `return`，10 个物理块恰有 10 个不同 owner，30 个实际指令 BCI 在 source map 全部可查。原源码、JADX 和 Jarde 三份完整类分别经 `javac --release 8 -g:none -Xlint:-options` 重编，`java -Xverify:all` 八路径输出逐行相同；编译、运行日志以 `root-after-{original,jadx,jarde}-TernaryInIfProbe-*` 保存。

非 1/0 叶、非 `Z` 描述符、独立 `System.nanoTime(); pop2` 三个 class 的 CLI 方法结果均为保守引用，未折成单 Boolean return；第三出口、回边、异常边与额外入口由边界 fixture 的定向测试覆盖。Root 使用 `ControlRunner`、`BoundaryRunner`、`CountingRunner`、`ExtraEntryRunner` 在 JVM `-Xverify:all` 下重放六套控制，分别得到与冻结文本相同的 8/8/8/7/8/8 行；记录为证据目录的 `root-after-*-verified-run.txt`。控制 class 的 SHA-256、javap 和 JVM 结果见 [`fixture README`](../../../tests/fixtures/p3-two-exit-return/README.md)；31 个本 change 的语料文件路径与大小均对应指纹清单。全局指纹检查仍有另 121 个其它目录未登记的文件，已单独记为[架构/语料债务](../../evidence/java-syntax-2026-09-22/architecture-debt.md)，不影响本 change 的控制证明。

审读 `Region::TwoExitReturn` 与 Builder：候选在普通 `If` 递归前按真实 taken/fallthrough、exact predecessors、单入口和无环边界整体认领，`visited` 到最后才提交；Builder 先证 `Z` 与两个精确 `iconst_1/0; ireturn`，再按物理边逆向组合测试表达式，失败时不发射部分 return。现有 overlap validator 仍在。`CountingProbe` 的八条 `equals` 调用次数与原类相同，覆盖了惰性求值。低预算与预取消由定向测试检查停止状态。

Root 运行 `p3_two_exit_return` 7/7、`p3_region_owner_overlap` 1/1、`p3_short_circuit_transfer_gateway` 6/6、`jarde-java` 的 `p3_conditional_values` 2/2 与 `p3_proved_boolean_conditional_returns` 3/3，均通过。`cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-shared-terminal-boolean-returns --strict` 通过；`cargo clippy -p jarde-java --lib` 退出零，仍报告此前已有的 17 条警告。私有 Cargo target 在验收后清理。

相邻的双条件构造器委托样例仍有独立缺口，见[构造器条件实参分析](../../evidence/java-syntax-2026-09-25/constructor-conditional-delegation/analysis.md)；不混入本变更。
