# 2.4 Root 独立验收

2026-09-26，Root 在代理交接的最终代码上复跑 `jarde` 库单测 57/57（其中枚举 26/26、类源码 20/20）、`tests/class_source` 47/47、`jarde-java` 单测 180/180。完整证据与预算矩阵见 [代理记录](verification-2.4.md)；Root 还把完整 JSON 中委托构造器 BCI 7 的物理 descriptor 来源加入定向测试，并复跑该测试通过。终端构造器的 BCI 7 helper 调用、BCI 12 字段写入仍指向原 `(String,int,int)` 方法；默认/all 的类正文相同，物理字段和七个方法的 item、恢复状态及正文不变，source map 只按请求物化。`value()I` method-only 的正文和来源与类报告内原方法一致。

Root 使用 SHA-256 为 `8299ace8b147b900d7471db3e988a7500836335cf552182a7b472938f47a5df5` 的 CLI，从冻结源码在仓库外分别编译 `-g`/`-g:none`，独立运行默认/all class-source、完整 Java 8 重编和 JVM `-Xverify:all`。两组原/JADX/Jarde 都输出 `values=ZERO:0,ONE:1`、`effects=2:0,1`、`declared-constructors=2,3`；Jarde 默认/all 类正文 SHA 均为 `c748f28b586e93a7763f83314a7ba9db7e21cf294d4846283de1fa1a4f68ded5`，JSON 均保留四字段、七方法。Root 将冻结的九组 verifier-valid 拒绝控制脚本复制到临时目录，仅将预期 CLI SHA 改为最终值后重放：9/9 的原 class/JADX 都通过 verifier，Jarde 9/9 拒绝整组投影；原证据文件未改。

紧预算与取消的定向测试覆盖 `ir_items=0`、`elapsed_millis=0`、预取消和最终输出收费不足，不发布 `ZERO, ONE(1)` 或 `this(0)` 的半成品。普通 Stage/Measure、构造器与 `<clinit>` 的相关测试通过。`cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-proved-enum-constructor-delegation --strict` 通过；`cargo clippy --locked --target-dir /tmp/jarde-enum-delegate-projection-target -p jarde --lib --no-deps` 退出 0，有 10 条相邻既有 warning。

两个不属本变更的全量门禁未闭合：`jarde-reader --lib` 为 176 通过、1 失败，失败用例把 `tests/fixtures` 总量固定为 `(164,1152,98,381,8)`，当前共享树实测 `(310,1645,144,880,8)`；排除此计数测试后 176/176 通过。`jarde-cli --test class_source_cli` 为 15 通过、1 失败，普通类 `HistoricalControlFlow` 的旧断言要求单独 `// @bytecode 9`，当前来源标记为 `// @bytecode 9 10 13 14`；排除此测试后 15/15 通过。两条均记录为相邻债务，不混入枚举实现。
