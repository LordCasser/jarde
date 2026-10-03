## 1. 取证与基线

- [ ] 1.1 读两个门的完整上下文并回答 design 的三项取证义务：(a) `member_inner.rs:270` 零参数门的下游消费（字段读取词法替换、分配点唯一性是否依赖"物理参数 == 捕获参数"的一一对应）；(b) `facade.rs:4562` 起 `project_class_source_anonymous_super` 的实参发射路径当前如何假设"全部物理实参都是父类实参"；(c) 两片的原子发布接缝是否同一个（决定本片是一个投影还是两个投影的组合）。
- [ ] 1.2 重放冻结 fixture `tests/fixtures/proved-java-structure/anonymous-super-args/`（SHA 核对）：记录当前基线（完整源集 `javac --release 8` 退出 1、`AnonymousSuperArgs$1` ctor 的 verbatim 呈现）、原 class 的事件日志（`java -Xverify:all`）与 JADX 对照。
- [ ] 1.3 冻结至少五个负例：参数无消费、同一参数被两类角色消费、super 实参序与物理序不一致、捕获字段二次写入、多分配点；各自 `java -Xverify:all` 通过并记录实现前后呈现。

## 2. 两门放宽与参数角色划分

- [ ] 2.1 `member_inner.rs:270` 零参数门改为"父类构造器实参是物理参数的有序子序列，剩余参数逐一对应一个 synthetic 捕获字段写入"（design 决策 1）。
- [ ] 2.2 `facade.rs:4606` 的 `field_count != 0` 门改为"允许捕获字段存在，每个物理参数角色被唯一证明"（决策 1）；实现参数角色划分判据（决策 2）与全部拒绝条件。
- [ ] 2.3 `anonymous-super-args` 完整源集 `javac --release 8` 通过、`java -Xverify:all` 事件日志与原 class 逐行一致；隐藏项（捕获字段声明、构造器、字段写入）由 javac 重建（决策 3）。

## 3. 回归与验收

- [ ] 3.1 两个已交付片的既有正例逐字不变：`recover-proved-anonymous-inner-this`（8/8）、`recover-proved-anonymous-local-capture`（6/6）、`inline-proved-anonymous-super-arguments`（8/8）的全部测试，以及 `recover-ctor-reorder-dispatch-guard` 的三向负例；`cargo test --workspace --tests --locked --no-fail-fast` 全绿（当前主线 294 目标 / 2918 passed；已知 flake 家族见 handoff.md，单测复跑两轮判定）。
- [ ] 3.2 门禁：fmt；clippy **从 `.github/workflows/ci.yml` 逐字生成命令**（含 `--all-features`，29 项 `-A`；见 handoff.md 的实测教训——漏 features 旗标会在 `p5_optimize_workloads` 误报 `unit_arg`/`let_unit_value`）；`openspec validate --all --strict`（当前 266 项）；corpus 双腿扫描（差异应仅混合参数匿名形）；磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，报告前必 clean。
- [ ] 3.3 root 独立复核参数角色划分判据、两门放宽的原子性、三方行为与 5.3 里程碑剩余范围，更新 EM 账本（匿名类域）与 `present-proved-java-structure` 5.3 的状态说明；确认该 fixture 不再依赖 ctor 重排。
