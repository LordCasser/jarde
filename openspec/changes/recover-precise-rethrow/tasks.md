## 1. 固定现状与三方证据

- [ ] 1.1 自写 Java 8 fixture（最小类：两个受检异常的条件抛出、`catch (Exception e) { log(e); throw e; }`、窄 `throws`、一条 any 行 finally 负例、一条多捕获行负例、一条重抛非参数值的负例），javac --release 8 编译，冻结 class SHA；记录原 class 执行输出、jadx 1.5.6 输出（含其 `throws Exception` 宽化偏离）、jarde 修前输出（整方法 `jre_guard_finally_copy` 引用）。

## 2. guard 判别与子句呈现

- [ ] 2.1 `finally_copy` 候选加 `catch_type == 0` 前置；具名行不再产生 `Unproven::FinallyCopy`。定向测试：具名行精确重抛不再被该候选拒绝，any 行候选与既有 finally/TWR 证明全部保持。
- [ ] 2.2 具名单行 handler 末条 `athrow` 的操作数为入口存储槽位时，子句体末呈现 `throw <参数名>;`（复用 throw 语句路径）；来源含入口存储与 `athrow` BCI。多行到达 handler 交给既有 catch 路径，不加保留旧误判的阻断；呈现测试含正例文本、来源锚点，以及多捕获的已证明/仍拒绝部分和非参数重抛控制。

## 3. 对照与门禁

- [ ] 3.1 恢复文本 Java 8 重编译，与原 class 三种路径（两类异常、正常完成）执行对照，比较异常类型/消息/返回值；jadx 输出的 throws 宽化写入对照记录为 jadx 偏离。
- [ ] 3.2 复跑 guard、typed_catch、twr、throw、execution_comparison 既有回归与 `cargo fmt`、`cargo clippy -p jarde-java -p jarde --all-targets -- -D warnings`、`openspec validate recover-precise-rethrow --strict`；golden/语料计数若变则重录并说明。
