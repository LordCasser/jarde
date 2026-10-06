## 1. 取证与基线

> **合并范围（root 2026-10-04，Q2 判别实验后）**：本片在派发时**并入**父类嵌套名参数化 MVP（原"姊妹片"，从未立项；见 design 的两段 root 更正）——即同一函数 `project_generic_signature` 的**相邻分支**一并放开：接口走三道门放行（本文件 1.1/2.x），父类走**池形参数化**（新增 1.5/2.4/2.5，不重拼、不动 `names.rs` 自嵌套规则）。**开工时所有行号按锚点名重验**（`project_generic_signature`、`direct_parent_candidate`、`prove_direct_generic_superclass_parent`、`parent.binary_name.contains(&b'$')`、`parent_name.contains(&b'$')`），勿照抄本文件数字。
>
> **实现者收尾注（2026-10-06）**：锚点已按名字在本树（起点 `bd7142f7`）重验并逐条记入 `results/01-forensics-anchors.md`（表格含各锚的本树行号）。取证中发现**两处设计前提需更正**（拼写侧并非"无需扩展"、选定环境不携带 JRE image），处置与证据同见该文件；两处更正在 3.3 待 root 复核。

- [x] 1.1 重放固定 BR 家族（SHA 核对）：读 `src/class_source.rs` 的三道门（6372 总门不看 interfaces、6435–6441 `direct_parent_candidate` 分支互斥拒绝接口、6455–6461 else 分支要求全部接口无实参）与既有拼写循环（6516–6519，已对 `parsed.interfaces` 逐项 `spell_ordinary_signature_type`，交 `class_declaration_with_types` 6526）；读 `src/facade.rs:13736` 的 `prove_direct_generic_superclass_parent` 确认其 13765 行 `ACC_INTERFACE → false`（不可复用）与 `prepare_physical_class_source` 内 6429 闭包的唯一注入方式；确认 reader 的 `prove_class_signature_erasure`（signature.rs:263，315–340 已逐个校验 interfaces 擦除）无需扩展；记录 `BR$Impl`（`implements Comparable` 裸）基线类头文本与 bridge 片前置的现拒绝态。
  > 证据：`results/01-forensics-anchors.md`（锚点表 + 基线态 + reader 未改动的结论）；`results/04-baseline/before.txt` 的 `br-impl` 格。
- [x] 1.2 冻结至少四个变体/负例：单接口参数化（`implements Comparable<Impl>`，正例）、多接口部分可证（`implements A<X>, B`）、接口不可解析（保留裸类型 + **桥保持可见**）、arity/擦除不符（拒绝）；各自 `java -Xverify:all` 通过并记录实现前后类头文本与桥可见性。
  > 证据：fixture `tests/fixtures/p3-interface-header-projection/`（两条 javac 腿，README 含 sha256 与各格期望）；构建脚本 `results/03-fixtures/build-fixtures.sh`；实现前后逐格转录 `results/04-baseline/{before,after}.txt` 与对照表 `results/04-baseline/README.md`；测试 `tests/parameterized_interface_headers.rs`（正例实跑 `-Xverify:all` + `javac --release 8` 重编 + 轨迹逐字比较）。**实际冻结 7 格**：另含 `$` 名实参（`BR$Impl` 形，复用既有 br-family）、类型注解形（零回退）、实参拼写边界（记录）。
- [x] 1.3 （并入的父类 MVP）重放 `Spec`/`BR$StrBox` 基线：父类头**池形裸**（`extends BR$Box`）+ 参数收窄桥**可见**（`cc4b6f11` 交付的现状），记录类头文本与桥可见态；确认 `BR$Box` 满足 `prove_direct_generic_superclass_parent` 除 `$` 外的全部判据（`super_class==java/lang/Object`、`interfaces.is_empty()`，root 已 javap 实证）。
  > 证据：`results/04-baseline/before.txt` 的 `br-strbox` / `parent-spec` 格（裸头 + header 拒绝 + `set` 桥 visible）；`results/04-baseline/README.md` 对照表。
- [x] 1.4 （父类 MVP）判别实验复验：`extends D$Base<String>` 池形参数化在真 javac 8 与 javac 23 都 exit 0 且产物与 source 形恒等（superclass 项 + InnerClasses 行）——root 已在 `/tmp/rq2d` 做过，实现片以冻结 fixture 重做入证据目录（含深嵌套 `D$Mid$Leaf<Integer>` 一格）。
  > 证据：`results/02-q2-replay/q2-replay.sh` + `.out`（含"缺失名必须失败"负例自检）；**加强为整文件 `cmp` 恒等**（同名同类、只差父类拼写）；另加接口位复验 `iface-position.sh` + `.out` 与 `javap-comparable.txt`（平台事实转录）。
- [x] 1.5 （父类 MVP）负例冻结：`$` 父名但**其余判据不满足**（父类自身有非 Object 父/有接口，或 arity/擦除不符）→ 保持拒绝；多段嵌套名中段含 `$` 的边界形如实记录现拒绝态。
  > 证据：fixture `tests/fixtures/p3-nested-parent-projection/`（`NestedExtends` 非 Object 父、`ArityExtends` arity 不符、`Multiseg` 多段路径、`BareBox` 无实参对照）；测试 `tests/parameterized_interface_headers.rs::the_parent_cells_whose_criteria_are_unmet_keep_their_refusals` 与 `…::a_bare_dollar_named_parent_keeps_the_raw_header`；实现前后转录见 `results/04-baseline/`。

## 2. 类头接口投影

- [x] 2.1 复用既有类头投影通道把边界从父类扩到 `interfaces` 列表（design 决策 1）；`BR$Impl` 类头呈现 `implements java.lang.Comparable<BR$Impl>`，整类 `javac --release 8` 通过、`-Xverify:all` 运行与 orig.out 一致（`0`，含经接口引用调用 `compareTo`）。
  > 证据：`src/class_source.rs` 第四条路径（`interface_only`）+ 并列证明器 `facade.rs::prove_header_interface_definition`；`results/04-baseline/after.txt` 的 `br-impl` 格；`tests/class_source.rs::br_family_recovered_source_recompiles_and_runs_like_the_original`（整族重编 + `-Xverify:all` + `0\n0`）与 `tests/parameterized_interface_headers.rs`（`IfaceImpl`/`MultiIface` 经接口引用 `0\n0` / `1\nrun\n`）。
- [x] 2.2 消隐前置不变量落地（决策 2，与 `recover-bridge-admission-gates` 的共享契约）：类头未带类型实参时拒绝桥投影、保持桥可见；负例钉死"契约丢失 + 桥隐藏"的双重损失不出现。
  > 证据：新增共享事实 `ClassSourceDeclaration::header_interface_arguments`（仅"已发布且带实参"的接口入列），`facade.rs` 接口边前置改读该事实（`owner_generic && !carried → 保持可见`）；负例 `an_unresolvable_interface_keeps_the_physical_spelling_and_the_visible_bridge`（裸头 + 桥可见）与 `a_type_use_annotated_implements_clause_keeps_the_raw_header_and_visible_bridge`（头不可发布 ⇒ 桥可见，前置的 `!projected` 分支被真实行使）；父类边 `class_header_projected` 判据逐字未动。
- [x] 2.3 负例保留裸类型与物理来源；既有父类参数化投影、成员声明参数化（`recover-ordinary-parameterized-signatures` 9/9）与裸类型回退路径 diff 逐字不变；预算/取消原子性不变。
  > 证据：全量套件 319 目标 / 3092 passed / 2 failed（两处均为语料普查 pin，已重测，见 3.1）；corpus 双腿扫描差异恰为预期集合（`results/05-corpus-sweep.out`）；`class_header_spelling_keeps_pool_names_the_member_spelling_refuses` 单元测试钉住"非类头位置逐字不动 + `$` 名仍拒绝"；预算/取消路径的既有测试（`dependency_budget_stop_does_not_publish_a_partial_generic_header`、`cancellation_during_parent_proof_does_not_publish_a_generic_header`）全绿。
- [x] 2.4 （父类 MVP）放宽**两处** `$` 拒绝（`class_source.rs` 的 `parent.binary_name.contains(&b'$')` 与 `facade.rs` `prove_direct_generic_superclass_parent` 的 `parent_name.contains(&b'$')`），其余判据**逐字保留**；投影产出**池形参数化头** `extends BR$Box<String>`（不重拼、不动 `names.rs`）；`class_scope` 置位后确认 bridge 前置自动失效、`Spec`/`BR$StrBox` 的 `set(Object)` 桥转为隐藏（这是本分支的验收主锚——与 `cc4b6f11` 的四向表闭环）。
  > 证据：两处 `$` 判据删除（`git diff` 可核），其余判据逐字未动；`results/04-baseline/after.txt` 的 `br-strbox` / `parent-spec` 格（池形参数化头 + `set` 桥 hidden）；`tests/class_source.rs::the_parameterized_superclass_header_hides_the_parameter_bridge`（三向运行 `SPEC.set(String) ran` 两侧一致）；负例 `NestedExtends`/`ArityExtends`/`Multiseg` 保持拒绝。
- [x] 2.5 （父类 MVP）零回退锚：顶层形（`Specialized`）与已参数化父类形渲染逐字节不变；`cc4b6f11` 的 CI 测试（`the_superclass_header_precondition_keeps_a_raw_header_parameter_bridge_visible`）**须随桥转隐藏而更新为新期望**（前置失效态），更新方式为改断言到新终态、不得删除测试；corpus 双腿扫描差异类恰为 `Spec`/`BR$StrBox` 两个池形裸头类（root 普查 454 类中含 `$` 父 + 桥形仅此，越界即停）。
  > 证据：`Specialized` 顶层控制段在同测试内逐字未变；`cc4b6f11` 测试**改名并改断言到新终态**为 `the_parameterized_superclass_header_hides_the_parameter_bridge`（注释记录改名缘由，未删除）；corpus 双腿扫描（`results/05-corpus-sweep.out`）差异类 = 普查 4 类（`BR$Impl`/`BridgeProbe`/`BR$StrBox`/`Spec`）+ 本片新增 fixture 的预期格，越界项为零（首轮曾出现第 5 类 `AC`，根因是"全未投影也发布投影声明"，已修，见 `results/05-corpus-sweep-first-run-invalid.out` 与 05 README）。

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 direct-parameterized-superclass、ordinary-parameterized-signatures、bridge 家族、nested-headers 全部既有测试）、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，报告前必 clean。
  > 证据：`results/06-gates.md`（fmt / clippy 逐字生成 / 全量 `--all-targets --all-features` 套件 / openspec validate 的实测尾部）。两处失败均为语料普查 pin（`p5_corpus_fingerprint` 清单 + `jarde-reader` 的 fixture 计数元组），按测试自身说明重测后全绿。
- [x] 3.2 BR 家族与变体三方对照：原 class/固定 JADX（dev）/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA；corpus 双腿扫描（差异应仅类头与桥家族）。
  > 证据：`tests/class_source.rs::br_family_recovered_source_recompiles_and_runs_like_the_original`（原 class 与重编源 `-Xverify:all` 轨迹逐字一致）；`tests/parameterized_interface_headers.rs` 三格三向运行；corpus 双腿扫描 `results/05-corpus-sweep.out`（自检含正例/负例）。JADX 侧：本片为呈现改善（桥与类头文本），不改变恢复判定，故沿用 bridge 片已记录的三方对照，未重跑 JADX（该片结论：桥可见态下 javac 与 JADX 均不隐藏桥）。
- [ ] 3.3 root 独立复核类头投影判据、消隐前置不变量与三方行为，更新 EM 账本（泛型接口/类头域）与巡查记录。
  > 待 root：本片留 `results/01-forensics-anchors.md` 的两处设计前提更正（拼写侧需类头专用规则；选定环境无 JRE image ⇒ 平台事实 `platform_header_interface_fact` 与 spec 的 arity/擦除拒绝口径修订）供独立复核。

