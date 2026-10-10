# Loop-latch validation/build 准备 v1

状态：**私有 runner，尚未运行**。此脚本只供 root 审阅并应用 `loop-latch-origins-luna-v2.patch`，再提交准备记录/规划 checkpoint 后执行。`--source-base` 对应该 checkpoint 的 HEAD；待验证候选产品源码可保持为未提交 worktree 内容，并由 source pins 精确冻结。脚本不改写产品源码、测试、OpenSpec 规划或既有证据，也不勾选 tasks。

脚本：[run-validation-build-luna-v1.py](run-validation-build-luna-v1.py)。输入参数 `--source-base` 必须由 root 传入届时实际 HEAD 的完整 40 位 SHA-1；脚本运行 `git rev-parse HEAD` 并逐字比较，不把当前 `69c2b321b` 当成未来提交点。输出固定为 `validation-build-root-v1/`、`/private/tmp/jarde-loop-latch-cli-v1`（权限 `0555`）与 `candidate-cli-v1.json`。三者已存在时拒绝覆盖。候选 metadata 沿用已接受的 v2 collector 契约：`cli_path` 和 `cli_sha256`；并记录 source/test/canonical pin、build execution SHA、source base 与未提交产品标记。

执行命令：

```sh
python3 openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-luna-v1.py \
  --source-base <root提交后的完整40位HEAD>
```

脚本沿用 `recover-int-field-multiply-updates/results/run-validation-build-luna-v5.py` 的磁盘守卫和串行 raw 记录结构：每条命令启动前及执行中检查本仓 `target` 不超过 1 GiB、机器可用空间至少 20 GiB；每条 stdout/stderr 以独占创建的 `.raw` 文件保存，已有输出目录拒绝重跑覆盖；失败保留当次 `execution.json`、已完成命令记录和 raw。设置 `CARGO_BUILD_JOBS=1`、`RUST_TEST_THREADS=1`、禁用增量编译及 debug symbols，匹配 CI 串行运行意图。最终 build 产物复制到新 CLI 路径并设为只读可执行，再生成 metadata。

命令顺序是 fmt check、与 `.github/workflows/ci.yml` 同 scope 的 workspace/all-targets/all-features Clippy（完整沿用 CI 的 29 个 allow lint 与 `-D warnings`）、`jarde-java` lib、五个 loop 回归 target、`jarde-reader` lib、corpus fingerprint 和 CLI build，共 11 项。静态核得应用 patch 后 test 数为：`p3_loop_exit_gateways` 5（当前 3 个加 patch 的 2 个）、根 `tests/p3_loop_arm_join.rs` 的 4 个、`p3_loop_body_double_jumps` 7、`p3_loop_terminal_return` 5、`p3_effectful_exits` 12；`p5_corpus_fingerprint` 为 5 passed、1 ignored。所有定向 target 与 corpus 使用精确 summary 校验；Java 与 reader lib 要求成功且至少有一个实际测试结果。gateways 命令启用 `--nocapture`，并额外逐字检查 `no_prefix_while_latch_keeps_physical_source_and_one_owner` 与 `no_prefix_latch_budget_and_cancellation_publish_no_partial_source` 两个新测试的成功行。

候选源码 pin 覆盖 workspace 与相关 crate manifests、`Cargo.lock`、`region.rs` 的证明、`build.rs` origin fold、`emit.rs` source map replay、`report.rs` 报告组装、Rust Java crate 导出、class-source/facade/CLI recovery pipeline。测试 pin 覆盖 CI workflow、本次五个 loop 回归 target 和 corpus fingerprint target。canonical pin 在 runner 运行时从这些测试文件重新解析每一个字面 `include_bytes!`/`include_str!` 路径并闭合；因此新 `NO_PREFIX` class 虽只在私有 patch 中新增 include，也会在 patch 已应用后自动进入 pin 集合。该冻结类 SHA-256 为 `a94a7af3f6258765adac1161d02d76b77f0de3ba69ab2fa3b74327b9819d0989`。

已读取的候选 patch SHA-256：`8ea53a8db72f126b46b21d622be908b7d6c443460499f7458c40f00770dfdd7a`。runner 只验证 build 与测试；双 JDK/JADX/候选 CLI 完整类重放及独立 CI verifier 由后续步骤执行。没有运行本 runner、Git、Cargo、rustfmt、JDK、JADX 或 CLI；仅对新 Python 源码做语法 AST 检查。

相对成熟 v5 runner 的最小 delta：保留其 source-base 精确 HEAD 门禁、20 GiB/1 GiB 守卫、每命令 raw、源码前后 pin、失败执行记录、只读 CLI freeze 和 metadata 结构；改用本 change 的产品/测试/canonical 路径与 v1 输出名，增加 gateways 两个精确测试名门禁，把命令矩阵替换为本任务指定的 11 项回归/构建序列，并固定 Cargo/Rust test 串行为 1。没有引入通用 runner 层或独立 verifier。
