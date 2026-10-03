## 1. 取证与基线

- [ ] 1.1 读 design"第一个取证义务"的四项并逐条回答：(a) `prove_anonymous_double_capture`（member_inner.rs:541–660）哪些判据是 double 专属、哪些可泛化，尤其 `anonymous_double_constructor_shape`（759，6 指令固定形）对三参形为何不适用、如何按参数角色重写为按序核对（含 long/double 占两槽的 slot 宽度）；(b) `prove_family_capture`（158）的 `this$0` 判据（179–191）与新路径的边界（具名内部类形必须仍走原路径）；(c) `project_class_source_anonymous_super`（facade.rs:4562 起）的实参发射路径当前如何假设"全部物理实参都是父类实参"，改为子集后序保持与副作用计数如何调整；(d) 两侧原子发布接缝是否同一个（决定本片是一个投影还是两个投影的组合）。
- [ ] 1.2 重放冻结 fixture `tests/fixtures/proved-java-structure/anonymous-super-args/`（SHA 核对）：记录当前基线（完整源集 `javac --release 8` 退出 1、`AnonymousSuperArgs$1` ctor 的 verbatim 呈现——即 `recover-ctor-reorder-dispatch-guard` 合入后的状态）、原 class 的事件日志（`java -Xverify:all`）与 JADX 对照。
- [ ] 1.3 冻结至少六个负例：参数无消费、同一参数被两类角色消费、super 实参序与物理序不一致、捕获字段二次写入、多分配点、多 `val$` 字段；各自 `java -Xverify:all` 通过并记录实现前后呈现。

## 2. 新捕获路径与参数角色划分

- [ ] 2.1 在 `prove_anonymous_capture`（facade.rs:17563 分派处）新增第三条分支与 `prove_anonymous_val_capture`（沿用 double-capture 骨架，descriptor 按字段实际类型、构造器形按角色划分核对）；`prove_family_capture` 与 `prove_anonymous_double_capture` 判据**逐字不动**（design 决策 1）。
- [ ] 2.2 放宽 `facade.rs:4606` 的 `field_count != 0` 门为"允许捕获字段存在，每个物理参数角色被唯一证明"（决策 2）；实现参数角色划分判据与全部拒绝条件（决策 3）。
- [ ] 2.3 `anonymous-super-args` 完整源集 `javac --release 8` 通过、`java -Xverify:all` 事件日志与原 class 逐行一致；隐藏项（捕获字段声明、构造器、字段写入）由 javac 重建（决策 4）。

## 3. 回归与验收

- [ ] 3.1 三条既有捕获证明路径的正例逐字不变：`recover-proved-anonymous-inner-this`（8/8，`this$0` 具名形）、`recover-proved-anonymous-local-capture`（6/6，double 形）、`inline-proved-anonymous-super-arguments`（8/8，无捕获 super-args 形）的全部测试，以及 `recover-ctor-reorder-dispatch-guard` 的三向负例；`cargo test --workspace --tests --locked --no-fail-fast` 全绿（当前主线 **295 目标 / 2922 passed**；已知 flake 家族见 handoff.md：p4_plugins 计时、bulk_recovery_delivery、p3_two_exit_return、export_cli、gateway 族、observable_equals、ordinary_generic_projection，单测复跑两轮判定）。
- [ ] 3.2 门禁：fmt；clippy **从 `.github/workflows/ci.yml` 逐字生成命令**（含 `--all-features`，29 项 `-A`；见 handoff.md 的实测教训——漏 features 旗标会在 `p5_optimize_workloads` 误报 `unit_arg`/`let_unit_value`）；`openspec validate --all --strict`（当前 **268 项**）；corpus 双腿扫描（差异应仅混合参数匿名形）；磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，报告前必 clean。
- [ ] 3.3 root 独立复核参数角色划分判据、新路径与既有两路径的边界、原子性、三方行为与 5.3 里程碑剩余范围，更新 EM 账本（匿名类域）与 `present-proved-java-structure` 5.3 的状态说明；确认该 fixture 不再依赖 ctor 重排。
