# Tasks

> 纪律：开工先读 [design-forensics](../../evidence/java-syntax-2026-10-05/bound-receiver-nullproof/design-forensics.md)；每处"落点"在动手前做最小门控实验证实（拒绝诊断逐字比对定位产生路径，勿凭读码推断）；行号按锚点名重验。

- [ ] 1.1 门控实验先行：冻结锚 fixture（`StringBuilder sb = new StringBuilder(); Optional…ifPresent(sb::append)` 形）双腿（真 javac 8 `/Library/Java/JavaVirtualMachines/corretto-1.8.0_432` + javac 23 `--release 8`）；渲染确认现拒绝文本逐字来自 `lambda.rs` 的该分支且 `parameter_adaptation` 为真（若为假→停手报告：锚走的是别的分支，本片前提失效）；临时放宽该分支观察锚行为变化，证实落点。
- [ ] 1.2 负例冻结三形：可空参数接收者（`p.ifPresent…` 参数直传）、可空字段读接收者、捕获后重写形（`sb = new StringBuilder(); …; sb = other; o.ifPresent(sb::append)` 或等价重写）——各冻结类与现拒绝文本；`typed-functional-method-references` 既有绑定拒绝锚重放基线。
- [ ] 2.1 实现两道门（非空门：捕获值 SSA 定义为 `Operation::Allocate`；互斥门：捕获点后该槽无 store）+ 同 verdict 点放行逃逸（`LambdaForm::Lambda`）；判据块其余部分与拒绝文本逐字不动。
- [ ] 2.2 对照测试：锚双腿 0 引注 + 剥离编译 exit 0 + `-Xverify:all` 输出与原一致（`[S]` 形）；负例三形拒绝逐字；`recover_typed_functional_method_references` 既有测试全绿零回退。
- [ ] 3.1 全门禁（cargo test --workspace --all-targets --all-features --locked、fmt、ci.yml 46-76 逐字 clippy、openspec validate --all --strict）+ corpus 指纹再生（差异仅本形）+ 分逻辑提交（不 push）。
- [ ] 3.2 root 独立复核：门控实验记录、两道门与判据零放宽 diff 逐条、锚/负例/零回退实测、账本更新（第 5 族 critical 17 锚关闭）。（留 root）
