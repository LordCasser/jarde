# 精确重抛验收

2026-09-26，root 审查 `guard.rs::finally_copy`：新增条件只允许异常表的 any 行成为合成 finally 拷贝候选；具名行继续走已有 catch/throw 恢复路径，没有新增异常语法机制。冻结正例及 any、两行多捕获、非参数 throw 控制见[三方证据](../../evidence/java-syntax-2026-09-26/precise-rethrow/analysis.md)。

Root 独立运行 `CARGO_TARGET_DIR=/tmp/jarde-precise-root-accept-target cargo test -p jarde --test p3_precise_rethrow`（2/2）和同目录的 `cargo test -p jarde-java --lib finally_copy_tests`（8/8）；检查 handler 存储 BCI 34 与重抛 BCI 40 的来源映射。将修后完整类与冻结 runner 用 `javac --release 8` 重编，并执行 `java -Xverify:all`；正常返回 23、`ParseException("parse")` 和 `IOException("io")` 三行均与原 class 逐字一致。JADX 1.5.6 的正例运行亦相同，但把窄 `throws` 宽化为 `throws Exception`。`cargo fmt --all -- --check`、`git diff --check` 与 `openspec validate recover-precise-rethrow --strict` 通过。

代理复跑的 `p3_typed_catch` 6/6、`p3_guard` 13/13、`p3_twr_catch` 1 通过/1 既有 ignored；`p3_throw` 6 通过、1 失败、1 ignored。失败的 `lower_parameter_stack_value_is_not_an_extra_throw_read` 读的是无异常表的普通 throw 方法，不经过本次改动的 finally 候选；单独记录，不在本语法点修改。人工运行原本 ignored 的 execution comparison 于 `p3-handlers/v8 syncBody` 失败，该方法只有 any/monitor 行，仍不经过新增具名行条件。严格 Clippy 被仓库其它文件的 21 条现存 warning 阻断，不能声称门禁通过；这些回归与 lint 债务均留作独立处理。证据及测试文件没有重录其它 golden/语料计数。
