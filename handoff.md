# HANDOFF — jarde 当前接续入口（2026-10-09）

先核对 Git 和最新 HEAD 的 CI，再按下方队列继续。历史交接已在 Git 中保留，不从旧任务勾选数、分支名或旧 CI 结果推断当前状态。

## 当前收尾状态

基线 `790579e2` 的四个 CI job 已成功：[run 37786347619](https://github.com/LordCasser/jarde/actions/runs/37786347619)。前一片泛型字段源码写证明已经合入。本轮 `recover-class-scope-constructor-parameters` 实现、独立对照与本地完整门禁已完成，代码和证据一起合入 main 并推送。fmt、CI 同口径 clippy、两固定 seed 各 3,243 passed/0 failed/93 ignored、显式 ignored P3 3 项/构造实参 1 项/绑定引用 1 项、strict OpenSpec 321/321 全部通过。实现提交 `698a219e` 的远端 [run 37804470415](https://github.com/LordCasser/jarde/actions/runs/37804470415) 第二 seed 在既有 bulk 预算并发断言失败，后续 JDK25 oracle 未执行。已按原 OpenSpec 修正测试并加入确定性 probe/charge 对照，生产预算与语法恢复逻辑未变；详情见 [CI 预算验收](openspec/changes/recover-class-scope-constructor-parameters/ci-delivery-verification-root.md)。修正后的最终远端 CI 必须按最新 main HEAD 查询，本地通过不能替代实际 JDK25 oracle。

构造器片沿既有 candidate → method commit → published parameters → field commit 恢复 Object() 后直接 this 字段初始化的类作用域参数。T/T[]、上界、多变量、重复参数加载及宽槽均使用完整 AST/Code/SSA/InitRecord 与物理字段身份。未增加 pass、parser、IR、fixpoint，也不以未发布字段 Signature 循环证明构造参数。方法级字段赋值、任意调用/EH/this 委派仍有独立边界。

root 首次对照抓到 ThisDelegateHold 新回退：callee 发布 T，但 caller 仍是 Object。最终用既有调用清单的实际 SSA receiver 阻止未证明的委派目标投影；raw new 保持放行，发射端 raw 形及两个受限 diamond 来源已审查。`new; dup` 的 receiver 深度 1 误拒也由冻结 fixture 抓到并修复。初版失败证据保留，没有删掉测试缩小结果。

最终 20 族 × 真 Corretto8/OpenJDK23 × debug/no-debug：基线与候选均 72/80 完整编译并行为一致，无新增回退；9 个正例/36 输入完整 class/ctor/field 泛型反射一致。CrossHold 恢复 U 参数但擦除不兼容 T 字段；PeerNewHold 只恢复二参数 callee，caller/字段保持擦除；不能计为全恢复。前片 23 族重放每腿仍 22/23 编译并行为一致。见 [构造器 root 验收](openspec/changes/recover-class-scope-constructor-parameters/verification-root.md) 和 [字段 root 验收](openspec/changes/prove-generic-field-write-source-types/verification-root.md)。

## 下一步队列

1. **raw receiver 字段选择类型**：未立项。新证据在 [只读巡查](openspec/evidence/raw-receiver-source-selection-patrol/summary.md)。static raw 参数与保留 raw local 的字段 T 反事实可编译；InstanceRawLocal 被 renderer 折成 this，字段 T 反事实失败。因此不能只用 SSA 来源或物理擦除判断 receiver raw，需实际发射 AST 与已发布方法参数事实；先 direct raw 参数最小闭环，alias 分拆。原型取证不等于当前已恢复。
2. **普通泛型调用实参适配**：未立项。构造器 evidence 的 CallHold/ExceptionHold 正文已恢复，但 Object 参数传给实际发布 T 的 identity 方法，整类仍编译失败；JADX 四腿可编译。这是参数完整使用/实际已发布 callee 类型证明，不能误写成“正文拒绝”，也不能猜 cast。this 委派同类目标本片只安全拒绝，并未恢复其泛型链。
3. **构造器其他形与泛型剩余边界**：method-formal 字段赋值、this 委派/non-Object 父类/复杂正文、成员类 TestGeneric8 的完整恢复均不在本片；SCGB 原有 main 正文拒绝、DeferredSetter sink 反射及 ArraySetter T[] 参数仍有前片记录。不能把少量 fixture 的闭环当成整个泛型单元追平。
4. 接着按旧队列确认 ScopeRefusalsEscape 合法未变异形、LoopTestValues.storeTest 真实源形、switchBody guard。Class 字面量绑定引用的复制值/check 与完整 LG 的局部类型复用/finalize 是独立片，不混入 generic。
5. 依据本地 `/Users/lordcasser/workspace/testzone/jadx` 的测试/算法及 [71 单元账本](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md) 继续。71 是验收单元，不是成功率；先冻结源码/原类/JADX/Jarde差异，再写 OpenSpec，确定性实施用 Luna，root独立验收。JADX 可参考提取代码与算法，但语义由原 JVM 行为裁决。

## 构建与交接纪律

只允许 root 使用主仓共享 Cargo target：`CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1`，辅助树不建独立 target。20 GiB 可用空间为停建线，验收后清理残留。保留冻结输入 class/jar 和验收文本，清理生成类/临时 Driver，不删除另一项目的 target。

预算、取消、完整类编译与 JVM 行为需要同时验收。自述头和非空类文本必须断言；CLI 0/4 不是源码正确性判据。重编和执行 classpath 不含原 jar，不能删拒绝方法后冒充整类成功。反射核对 GenericDeclaration 身份，不以同名 T 或相同擦除合并 binder。

本地门禁：fmt、CI 明列债务白名单之外 -D warnings 的 workspace clippy、两固定 seed 5350648285461741569/70、strict OpenSpec、ignored P3/functional-constructor/bound-receiver。JDK25 instruction-boundary oracle 在远端实际 JDK25 核对，本地8/23不能冒充25。最终远端状态必须按最新 main HEAD查询。

收尾已核对 15 个工作树：辅助树全部 detached、干净且 HEAD 为 main 祖先；本地和远端只剩 main，没有分支占用。共享 Cargo 曾清理 7,690 个文件、17.1 GiB；CI 修正复验的残留在最终验收后再次清理。历史固定保护的 detached 副本，Codex 归档明确拒绝删除，保留这些副本，不绕过保护。

```sh
git status -sb
git rev-parse HEAD origin/main
git branch -avv
git worktree list --porcelain
gh run list --repo LordCasser/jarde --limit 5
df -h /System/Volumes/Data
```
