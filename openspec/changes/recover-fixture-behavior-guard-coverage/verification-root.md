# Root 独立验收（2026-10-07，合并）

## 判据逐项

1. **零生产改动**：合并 diff `-- crates src` 为 0 行 ✓（扰动已完整回退，root 复核 `git diff HEAD~1..HEAD --stat -- crates src | wc -l` = 0）。
2. **负向自检复核**：`results/03-self-check-perturbation*.out` —— 扰动的正是 10-04 事故同款洞（`ctor_order.rs` 的 `Object.<init>` 三合取→true），**新默认套件测试失败于捕获顺序错置**（呈现腿是可见守卫；ignored 腿该运行不可观测——正是设计意图），回退后 7/7+6/6。守卫可判伪性实证。
3. **golden 出处复核**：六 fixture 的 `java -Xverify:all` 输出与既有证据目录 `original-run*` 逐字节交叉核对（自检脚本空产出即中止）；不可编译两件（anonymous-member-base/lambda-body-inline）的行为腿**明写**钉当前事实、恢复日更新分类非删测试。
4. **前提更正追认**：proposal/tasks 所记 `anonymous-top-level`"当前物理呈现"已过期——环 2（`23b69bcf`）已交付投影形，本片按**投影形**钉锚（钉物理文本会把环 2 交付钉成回归）——实现者开工复核更正正确。
5. **root 实测**：新套件 7 默认 + 6 ignored 全绿（ignored 腿真实执行 java+javac，`--nocapture` 核对 4 重编对照 + 2 运行+拒绝钉）；门禁（权威口径）：全量 exit 0、**332 targets ok、0 FAILED**；fmt OK；CI 逐字 clippy `Finished` 0；openspec **307/307**；corpus 指纹未动（零位移）。
6. **CI**：合并推送后 run 为准（监控在案）。

## root 裁定（残余 2 项）

1. **ignored 腿不接 CI step——维持设计意图**：呈现腿守默认套件是 design 本意；行为腿在验收时本地跑。加 step 属共享 workflow 决定，登记为后续可选（`fixture_behavior_guards -- --ignored` 与 `p3_execution_comparison -- --ignored` 同形态，若后续巡查发现呈现腿盲区再接）。
2. **handoff 第 128 行登记句刷新**：8 个未守卫 fixture 的补覆盖**已完成**（本片 6 件 + super-dispatch/super-args 既有守卫）——随本验收更新。
