## 1. 取证与基线

- [x] 1.1 重放固定 A10/A11/A12（SHA 核对）：定位类字面量准入判据与 nested-spelling 可拼性事实的复用接口；记录 A12 两形与 A11 三形基线（引注文本）。（判据即 `decode.rs::source_internal_name`，全仓仅 `class_literal_type` 两处调用；A12 前 6 / A11 前 15 条 `@bytecode` 引注，转录见 evidence `results-values/*.{before,after}.txt`。）
- [x] 1.2 冻结至少四个变体/负例：多段嵌套 `A$B$C` 字面量、折叠域内字面量（jar 口径）、本地类字面量（拒绝）、匿名类字面量（拒绝）；各自 `java -Xverify:all` 通过并记录实现前后行为。（N2/fn2.jar 多段+jar 折叠、LC 四类本地/匿名负例均为既有冻结；新增冻结 A10.class、A9 家族+a9.jar、a11.jar、`fixture-variants/WC1`（字面 `$` 顶层类）、`fixture-variants/WV1`（数组形），SHA 与前后行为见 `results-values/fixture-sha256.txt` 与 README。）

## 2. 准入扩展

- [x] 2.1 判据复用可拼性事实准入嵌套名（design 决策 1–2）；A12/A11/A9.main 恢复、整类重编运行与基线逐字一致；A10 五形 diff 逐字不变。（A12 6→0、A11 15→0、A9 17→0 引注；A12/A11/A9/A10 重编运行逐字一致（`results-values/*-jarde-recompile.out`）；`A10.after.txt` 与巡查主线转录字节一致。N2 三形方法级恢复，重编工程编译通过；结构反射两线偏离属折叠深度域遗留 1。**修正轮**：root 驳回 N2 的"可编译且行为不同"（消隐/呈现不变量），按修正判据补守卫——真嵌套（行集内）∧ 最终呈现池形 ∧ 结构反射消费 ⇒ 拒绝（build 层 `structural_reflection_over_pool_spelled_literal` + standalone 根旗标 `with_pool_spelled_members`）；N2 `multiLevel`/`recvChain` 退回响亮失败、`midLevel`（getName）仍恢复逐字；A12/A11 守卫前后字节一致（`*.zero-regression.diff` 空）；遗留 2（折叠 token-tie 锚不认数组描述符元素名）随守卫闭环——WV1 折叠恢复、三线重编运行逐字一致（`wv1-jarde-recompile-guard.out`）。）
- [x] 2.2 负例保持拒绝；呈现两口径（折叠/分离）按 nested-spelling 既有规则锚定；预算/取消原子性不变。（LC 2→2 保持拒绝；折叠域 A12/N2/A11/A9 简单拼写、分离域 N2/WV1/WC1 池形；预算/取消契约测试无改动全绿。**修正轮**：负例扩为三向钉死——池形+结构反射→拒绝（N2）、池形+getName→恢复（N2 midLevel / A12 standalone nestedRecv）、行集内折叠形+结构反射→恢复（A12/WV1 folded）；WC1 顶层 `$` 名凭"真嵌套"行集条件免于误拒。）

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 nested-spelling、类字面量、反射相关全部既有测试）、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，报告前必 clean。（第二修正轮实测 296 目标 / 2933 passed / 0 failed、fmt 干净、CI clippy `--all-features` exit 0、openspec 268/268、`git diff --check` 干净；corpus 465 类双腿复扫 0 差异；全程磁盘 ≥25Gi。）
- [x] 3.2 A10/A11/A12 与变体三方对照：原 class/固定 JADX（dev）/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。（`results-values/`：jadx-*.out、*-jarde-recompile.out、fixture-sha256.txt；JADX dev 的 defpackage 重打包使 getName 路径自身偏离原类，已按路径记录口径。）
- [x] 3.3 root 独立复核准入判据、负例边界与三方行为，更新 EM 账本（反射/类字面量域）与巡查记录。（**root 2026-10-04 验收记录，补记于合并主线 `78db127b` 之后**；全部独立复跑与端到端实测，不采信实现者自报：

**门禁（root 实测）**：合并后在干净主线复跑 `cargo test --workspace --tests --locked --no-fail-fast` → **296 目标 / 2933 passed / 0 failed**（首轮 `p3_short_circuit_transfer_gateway::protected_gateway_with_exception_edge_is_refused` 报 `scratch directory: AlreadyExists`，属 handoff 已登记的临时目录碰撞 flake，单测两轮复跑均通过）；`cargo fmt --all -- --check` 通过；clippy 从 `ci.yml` 46–76 行逐字生成（含 `--all-features`、29 项 `-A`）exit 0；`openspec validate --all --strict` 269/269；CI 于 `a5c4762b`/`ca135c9f` **ALL GREEN（4 jobs）**。
**准入判据复核**：生产 diff 仅 `crates/jarde-java/src/decode.rs::source_internal_name`——`!name.contains('$')` 改为 `name.split(['/', '$']).all(is_java_identifier)`，与 design 决策 1 逐字一致；`emit.rs`/`names.rs`/`class_source.rs` **呈现缝未动**（符合"呈现零新增"），`spell_reference` 身份缝保持池形。全仓 `source_internal_name` 仅两处调用方（`class_literal_type` 内数组元素名与对象名），故放宽只影响类字面量准入。
**负例边界复核（实现者以 javac 生成的真实二进制名钉死，非推理）**：`Outer$1`（匿名）、`Outer$1Local`（本地类）、`Outer$`（空段）、`Outer$$Inner`（空段）全拒——尾段数字开头即非合法标识符起始，同一判据自然排除，无需专门规则；`WC1$Top`（字面 `$` 顶层类）准入且两侧保持池形、行为逐字一致。
**三方行为复核**：A10 顶层五形、A11 反射读注解三形、A12 两形、A9.main（17 处引注清零）、WV1 数组形均恢复且重编运行与各自基线一致；A10 after 转录与巡查主线转录**字节一致**（零回归）；corpus 双腿扫描 465 类 **0 差异**。
**root 在验收中发现并退回的两个阻塞点（均已修正后才合并）**：
1. **深层名结构反射静默偏离**：N2 的 `multiLevel`（`getSimpleName`）/`recvChain`（`getEnclosingClass`）在准入放宽后由基线的响亮失败（10 处引注、方法体空、javac 报 missing return）变为**可编译且行为不同**（`Leaf`→`N2$Outer$Mid$Leaf`、`getEnclosingClass()` 返回 null → NPE）。守卫判据为"最终呈现文本仍含 `$` ∧ 值被 11 个结构反射方法之一消费"，落在 `build.rs`；四向验收（A12-jar 折叠成功→恢复、A12-single→拒绝、N2 midLevel `getName`→恢复、N2 multiLevel/recvChain→拒绝）经 root 用真实二进制逐一实测确认。
2. **family 折叠失败回退路径绕过守卫**（root 以 RF fixture 决定性实证）：`RF$Inner` 因 `<clinit>` 使折叠被拒 → 回退分离池形呈现 → 家族 `javac` exit 0 但 `getSimpleName()` 返回 `RF$Inner`≠原类 `Inner`。修复为 facade 在折叠失败回退处以 `pool_spelled_members=true` 重跑受影响方法（复用既有 `analyze_method_ir` re-recovery），采纳规则仅在重跑文本含守卫原句时 `replacen` 替换——root 读码确认该规则**不会**把好文本换成更差文本。修复后 RF-jar 退回响亮失败（家族 `javac` exit 1 `缺少返回语句`），四向复测全过。
**root 自查纠错（诚实登记）**：给实现者的首版判据（"名不在 `InnerClasses` 行集内"）被 javap 实证推翻——`N2.class` 的表**确实含** `Leaf=class N2$Outer$Mid$Leaf of class N2$Outer$Mid` 行。真实机制是 `names.rs:204-207` 的自嵌套规则 + 折叠单层（池形成因是折叠深度，非行集缺失）。root 在实现者据错误前提施工前发出纠正指令，并把正确机制写入 design 与 handoff（"池形类型名的结构反射陷阱"）。
**账本更新**：inventory **DT-24**（类字面量/注解值域）已于 `e116aaba` 回写本片闭合结论与残留债务（深层成员结构反射、[deep-member-fold-spelling](../../evidence/java-syntax-2026-10-04/deep-member-fold-spelling/README.md) 的单层折叠拼写不一致——后者为既有响亮失败，非本片引入）。
**遗留**：`recover-fixture-behavior-guard-coverage`（8 个冻结行为 fixture 无 CI 守卫）是本片能静默通过 CI 的根因，已立项待派发。）
