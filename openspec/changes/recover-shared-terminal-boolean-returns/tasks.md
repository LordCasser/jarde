## 1. 冻结图与反例

- [x] 1.1 保存 Java 8 正例的源码、class、javap、原/JADX 八路径运行、Jarde 拒绝及 BCI 62 双 owner/物理 CFG 追踪；见 [分析](../../evidence/java-syntax-2026-09-25/ternary-in-if/analysis.md)与 [Region trace](../../evidence/java-syntax-2026-09-25/ternary-in-if/region-trace.md)。
- [x] 1.2 制作 verifier-valid 的非 1/0 返回、非 `Z` 方法、第三出口或图外入口、独立可观察语句至少三种控制，保存 javap/JVM 运行与 fixture 字节指纹，并以定向测试证明不会误折叠。

## 2. 有界双出口归属

- [x] 2.1 在普通 `If` 递归前加入私有双出口候选，只收单入口、前向无环、有界测试/纯转接与两个终结返回；用定向 Region 测试证明正例每个块恰一 owner、负例不跳过 overlap validator。
- [x] 2.2 对每个测试核对真实 fallthrough/taken、所有内部前驱、外部入口、边类别及预算/取消，整体提交或拒绝；以第三出口、额外前驱、回边/异常边及低预算测试验证无部分 Region。

## 3. 布尔返回证明与发射

- [x] 3.1 Builder 证明方法描述符 `Z`、两个终结块各自精确 `iconst_1/0; ireturn`、每个测试表达式可呈现且无独立效果；用非 1/0、非 `Z`、中间效果控制验证拒绝。
- [x] 3.2 按真实边逆序生成已有 Boolean/Conditional/Return AST，一次发射并保留全部物理 BCI；正例完整类 Java 8 重编和八条 `-Xverify:all` 路径与原/JADX 相同，source map 与零 fallback 由测试核对。
- [x] 3.3 运行低预算、预取消及现有短路 Phi、普通 `if`、条件返回回归，证明失败不发布部分 Java 或改坏既有消费者；记录针对性测试结果。

## 4. Root 独立验收

- [x] 4.1 root 独立构建 CLI 并复跑冻结正例与控制 class，保存完整类编译/八路径执行、BCI 来源和 CLI/class SHA，独立确认行为与拒绝边界。
- [x] 4.2 root 审读所有权、SSA/极性、惰性效果及预算路径，运行定向回归、`cargo fmt --all -- --check`、适用 Clippy、`git diff --check` 与 OpenSpec strict，记录未完成债务并清理私有 Cargo target。
