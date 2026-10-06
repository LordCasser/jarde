# Tasks

> 纪律：开工先读 [design-forensics](../../evidence/java-syntax-2026-10-05/bound-receiver-nullproof/design-forensics.md)；每处"落点"在动手前做最小门控实验证实（拒绝诊断逐字比对定位产生路径，勿凭读码推断）；行号按锚点名重验。

- [x] 1.1 门控实验先行：冻结锚 fixture（`StringBuilder sb = new StringBuilder(); Optional…ifPresent(sb::append)` 形）双腿（真 javac 8 `/Library/Java/JavaVirtualMachines/corretto-1.8.0_432` + javac 23 `--release 8`）；渲染确认现拒绝文本逐字来自 `lambda.rs` 的该分支且 `parameter_adaptation` 为真（若为假→停手报告：锚走的是别的分支，本片前提失效）；临时放宽该分支观察锚行为变化，证实落点。
- [x] 1.2 负例冻结三形：可空参数接收者（`p.ifPresent…` 参数直传）、可空字段读接收者、捕获后重写形（`sb = new StringBuilder(); …; sb = other; o.ifPresent(sb::append)` 或等价重写）——各冻结类与现拒绝文本；`typed-functional-method-references` 既有绑定拒绝锚重放基线。
- [ ] 2.1 实现两道门（非空门：捕获值 SSA 定义为 `Operation::Allocate`；互斥门：捕获点后该槽无 store）+ 同 verdict 点放行逃逸（`LambdaForm::Lambda`）；判据块其余部分与拒绝文本逐字不动。
- [ ] 2.2 对照测试：锚双腿 0 引注 + 剥离编译 exit 0 + `-Xverify:all` 输出与原一致（`[S]` 形）；负例三形拒绝逐字；`recover_typed_functional_method_references` 既有测试全绿零回退。
- [ ] 3.1 全门禁（cargo test --workspace --all-targets --all-features --locked、fmt、ci.yml 46-76 逐字 clippy、openspec validate --all --strict）+ corpus 指纹再生（差异仅本形）+ 分逻辑提交（不 push）。
- [ ] 3.2 root 独立复核：门控实验记录、两道门与判据零放宽 diff 逐条、锚/负例/零回退实测、账本更新（第 5 族 critical 17 锚关闭）。（留 root）

## 任务 1.1/1.2 结果与阻塞（coder，2026-10-06）

- **1.1 已勾**：落点与前提**成立**——HEAD 干净构建下锚两条腿的 ⑤ 诊断均逐字出自该分支，四条位点
  （OP 锚 + BRN 三负例）实测 `parameter_adaptation=true`（记录 `results/01-experiment-report-*.txt`，
  模型自身位点记录 `results/00-lambda-site-record-*.txt`）。**但门控实验证伪"仅加两门逃逸即可恢复锚"**：
  锚的捕获值 SSA 定义是 javac 绑定接收者丢弃空检查尾的 `dup`（`captures.0.bci = 10`），非
  `Operation::Allocate`；放宽该分支后锚改为被复制族值级拒绝，方法体仍整体引注（`results/01-experiment-relax-*`）。
  按 handoff「spec 未枚举的门 → 停手 + 提出方案」纪律**本片未落生产改动**，`git diff` 对 HEAD 为空。
- **1.2 已勾**：三条负例双腿冻结（`results/fixtures/`，`v8/OP.class` 与巡查 jar 的 `OP.class` 逐字节相同），
  基线拒绝文本三条逐字、两腿整类文本逐字相同（`results/00-baseline-*-BRN.txt`）。
- **2.1/2.2/3.1 阻塞**：完整实验矩阵、**实测充分的最小机制**（两门逃逸 + 站点自有丢弃空检查尾的识别/
  经尾读回 + 该尾三 BCI 记入站点所有）、行为回放（6/6 输出 `hi/none/none/3/0/42/false/S/`）与待裁决的
  设计问题（尾证明的所有权落点、窄/宽口径、守卫交互）见
  [results/01-gating-experiment.md](results/01-gating-experiment.md)；实验补丁存
  [results/01-experiment.patch](results/01-experiment.patch)（已还原）。**待 root 重新设计落点后再派 2.x/3.x。**
