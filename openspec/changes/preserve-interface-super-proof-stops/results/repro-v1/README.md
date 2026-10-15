# 公共接口证明停止反例的接手证据

这里保存只读复制的最小测试、patch、baseline pins、实际两条 exact 测试命令及其完整 stdout/stderr、最终 clean 命令与 raw。`manifest.json` 对每个复制文件记录 bytes/SHA256，并列出原外置 source-before/after gzip 的绝对路径与 pins。完整历史证据保留在 `/home/lordcasser/.cache/jarde-review/20261015/interface-proof-stop-repro-v1/`，没有重写；规划导出只含列出的最小子集，不声称包含整套编译闭包。

测试源 SHA256 为 `f285e92259533d5a55c2c351436c5107b68433d92774181d7285b6ac8b83420f`。它原位于私有仓库 `tests/interface_proof_stop_repro.rs`，三个 `include_bytes!` 相对路径以仓库 tests 目录为基准；这里的副本用于审阅和后续实现迁入，不作为原位可编译 target。fixture 仍是仓库 `tests/fixtures/p3-special-dispatch/v8/{SpecialProbe,BaseProbe,DefaultProbe}.class`，SHA/bytes 见 manifest，未生成新 class。

实跑基线为私有 HEAD `f01402867ffa20085e421a680a6ebbe878e63338` 上的 13 个候选文件加 build `b973da39da521dad9a71c53114c22901eaf056f63fa55823949004b2cc234bb8`，详见 baseline-pins；不是 primary `644882d` 的全源编译。宽预算真实 1 passed/0 failed，原生公共 `Engine::recover_target` 返回 Complete 且正文有 `DefaultProbe.super.value()`。depth0 真实 0 passed/1 failed、退出 101，编译成功，失败于要求 Partial/DependencyDepth 的断言：实际 analysis Complete、恢复 ExplanationOnly/Fallback、presentation Complete/stop=None。每条有 1 filtered，不计覆盖。

本证据仅证明该预算反例；取消、期限、class-source、原合法/拒绝来源的修复回归和最终 fix 产品自身 CI 都尚未执行。负控制的正常源码拒绝存在，不能替代丢失的 execution stop。原停止原因从真实证明路径与唯一 depth 限额变化确定，不伪造被吞 Error 的内部 trace。

实跑沿用 stable/offline/jobs1/incremental0/debug0/testthreads1/default stack、每秒 5 GiB free/1 GiB target 守卫。负控制失败后自动严格 cargo clean 成功；另一次最终 `argv=["cargo","clean"]` 退出 0，target absent；完整自动清理收据仍在外置历史目录，复制的 audit.json 引用其 hashes。本轮规划不再运行 Cargo/JDK/产品 CLI，也不改变生产或 tests。
