## 1. 取证与基线

> **合并范围（root 2026-10-04，Q2 判别实验后）**：本片在派发时**并入**父类嵌套名参数化 MVP（原"姊妹片"，从未立项；见 design 的两段 root 更正）——即同一函数 `project_generic_signature` 的**相邻分支**一并放开：接口走三道门放行（本文件 1.1/2.x），父类走**池形参数化**（新增 1.5/2.4/2.5，不重拼、不动 `names.rs` 自嵌套规则）。**开工时所有行号按锚点名重验**（`project_generic_signature`、`direct_parent_candidate`、`prove_direct_generic_superclass_parent`、`parent.binary_name.contains(&b'$')`、`parent_name.contains(&b'$')`），勿照抄本文件数字。

- [ ] 1.1 重放固定 BR 家族（SHA 核对）：读 `src/class_source.rs` 的三道门（6372 总门不看 interfaces、6435–6441 `direct_parent_candidate` 分支互斥拒绝接口、6455–6461 else 分支要求全部接口无实参）与既有拼写循环（6516–6519，已对 `parsed.interfaces` 逐项 `spell_ordinary_signature_type`，交 `class_declaration_with_types` 6526）；读 `src/facade.rs:13736` 的 `prove_direct_generic_superclass_parent` 确认其 13765 行 `ACC_INTERFACE → false`（不可复用）与 `prepare_physical_class_source` 内 6429 闭包的唯一注入方式；确认 reader 的 `prove_class_signature_erasure`（signature.rs:263，315–340 已逐个校验 interfaces 擦除）无需扩展；记录 `BR$Impl`（`implements Comparable` 裸）基线类头文本与 bridge 片前置的现拒绝态。
- [ ] 1.2 冻结至少四个变体/负例：单接口参数化（`implements Comparable<Impl>`，正例）、多接口部分可证（`implements A<X>, B`）、接口不可解析（保留裸类型 + **桥保持可见**）、arity/擦除不符（拒绝）；各自 `java -Xverify:all` 通过并记录实现前后类头文本与桥可见性。
- [ ] 1.3 （并入的父类 MVP）重放 `Spec`/`BR$StrBox` 基线：父类头**池形裸**（`extends BR$Box`）+ 参数收窄桥**可见**（`cc4b6f11` 交付的现状），记录类头文本与桥可见态；确认 `BR$Box` 满足 `prove_direct_generic_superclass_parent` 除 `$` 外的全部判据（`super_class==java/lang/Object`、`interfaces.is_empty()`，root 已 javap 实证）。
- [ ] 1.4 （父类 MVP）判别实验复验：`extends D$Base<String>` 池形参数化在真 javac 8 与 javac 23 都 exit 0 且产物与 source 形恒等（superclass 项 + InnerClasses 行）——root 已在 `/tmp/rq2d` 做过，实现片以冻结 fixture 重做入证据目录（含深嵌套 `D$Mid$Leaf<Integer>` 一格）。
- [ ] 1.5 （父类 MVP）负例冻结：`$` 父名但**其余判据不满足**（父类自身有非 Object 父/有接口，或 arity/擦除不符）→ 保持拒绝；多段嵌套名中段含 `$` 的边界形如实记录现拒绝态。

## 2. 类头接口投影

- [ ] 2.1 复用既有类头投影通道把边界从父类扩到 `interfaces` 列表（design 决策 1）；`BR$Impl` 类头呈现 `implements java.lang.Comparable<BR$Impl>`，整类 `javac --release 8` 通过、`-Xverify:all` 运行与 orig.out 一致（`0`，含经接口引用调用 `compareTo`）。
- [ ] 2.2 消隐前置不变量落地（决策 2，与 `recover-bridge-admission-gates` 的共享契约）：类头未带类型实参时拒绝桥投影、保持桥可见；负例钉死"契约丢失 + 桥隐藏"的双重损失不出现。
- [ ] 2.3 负例保留裸类型与物理来源；既有父类参数化投影、成员声明参数化（`recover-ordinary-parameterized-signatures` 9/9）与裸类型回退路径 diff 逐字不变；预算/取消原子性不变。
- [ ] 2.4 （父类 MVP）放宽**两处** `$` 拒绝（`class_source.rs` 的 `parent.binary_name.contains(&b'$')` 与 `facade.rs` `prove_direct_generic_superclass_parent` 的 `parent_name.contains(&b'$')`），其余判据**逐字保留**；投影产出**池形参数化头** `extends BR$Box<String>`（不重拼、不动 `names.rs`）；`class_scope` 置位后确认 bridge 前置自动失效、`Spec`/`BR$StrBox` 的 `set(Object)` 桥转为隐藏（这是本分支的验收主锚——与 `cc4b6f11` 的四向表闭环）。
- [ ] 2.5 （父类 MVP）零回退锚：顶层形（`Specialized`）与已参数化父类形渲染逐字节不变；`cc4b6f11` 的 CI 测试（`the_superclass_header_precondition_keeps_a_raw_header_parameter_bridge_visible`）**须随桥转隐藏而更新为新期望**（前置失效态），更新方式为改断言到新终态、不得删除测试；corpus 双腿扫描差异类恰为 `Spec`/`BR$StrBox` 两个池形裸头类（root 普查 454 类中含 `$` 父 + 桥形仅此，越界即停）。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 direct-parameterized-superclass、ordinary-parameterized-signatures、bridge 家族、nested-headers 全部既有测试）、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，报告前必 clean。
- [ ] 3.2 BR 家族与变体三方对照：原 class/固定 JADX（dev）/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA；corpus 双腿扫描（差异应仅类头与桥家族）。
- [ ] 3.3 root 独立复核类头投影判据、消隐前置不变量与三方行为，更新 EM 账本（泛型接口/类头域）与巡查记录。
