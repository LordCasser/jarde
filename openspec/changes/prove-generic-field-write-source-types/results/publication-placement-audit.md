# 字段写证明的提交位置（root 审查）

基线：main `2dea3217`；只读审计与 root 源码复核。该记录是实现位置裁决，不是运行验收，也不勾选任务 1.2。

`scan_member_uses` 当前在 `putfield`/`putstatic` 上仅记录 `read_expressible=None`。`prove_same_class_field_binding` 只拒绝读取的 `Some(false)`，因此写位虽进入清单，尚无源码类型判据。

方法的 `generic_signature_projected` 仅在 `project_generic` 完成正文/声明放置并成功收取输出预算后为 true。`SignatureProjection::Settled` 同时包括无 Signature、拒绝以及未能发布的情况，不能代替该状态。方法 Signature 仅在真实发布时可作参数类型来源；其余保留物理 descriptor。类型变量须区分类作用域与方法作用域，方法形式参数按词法遮蔽类变量。

同类 commit 之前，所有被分析的方法记录和成员使用扫描已汇集。方法 Signature 的延期提交依赖自己的 Signature、正文候选、类作用域以及成员绑定清单，不读取已投影字段声明。因此在原 commit 阶段先结清方法、后证明字段没有新的循环依赖；无需新增 pass 或 fixpoint。发生预算/取消停止时必须停止后续提交，不能把未结清方法当作发布成功。

后续 enum、lambda、array、assert、bridge 和 pool-spelled rerun 保留物理方法声明及泛型状态，只改正文/marker。member-family 输出会克隆 root `make`/child `id` 并重新投影克隆，不写回物理方法；今后若证明折叠后的 writer，必须另用该输出自身的事实，本片不得把两层状态混用。

最小写事实应指向物理方法身份和同轮 SSA 来源。参数 ordinal 必须由物理 descriptor 与 static/instance 槽布局导出，涵盖 long/double 双槽；不能用 caller 展示字符串作身份。直接 null 与直接未改写参数是正例；`Object local = null` 的局部变量仍是 Object 源类型，不能仅因 SSA 值为 null 将它当作字面量。未证明的 cast、phi、调用及参数改写均拒绝。

原有正例不仅有参数：`tests/same_class_generic_binding.rs` 的 SCGB（Map<String,List<T>> / raw new HashMap）、SCGE（List<T> / raw new ArrayList）和 SCGC（无写 List<T>）均必须保留。直接分配写来源应沿现有分配闭合与源码赋值证据验证，必要时复用已有 release 平台关系；不能添加按字段 owner 特判的列表，也不能借物理擦除把 Object→T 当作 raw→参数化转换。RHS 为有类型参数的 List<String> 也不能按 raw List 处理。

门控必须实际证明 Deferred writer 在方法结清后才作为字段正例，并验证 unknown/预算停止的拒绝与传播。SCGA.note(T) 的既有同类 Methodref 正例可作为形态起点。对完整类的重编译和 JVM 行为验收仍由 root 完成。

## JADX 参考与擦除根因

本地 JADX `SignatureProcessor.parseFieldSignature` 解析字段 Signature、扩展变量、对照物理类型后更新字段类型；`parseMethodSignature` 独立更新方法类型。这个现成解析顺序不提供“最终呈现 RHS 源类型可赋值”的证明，不能直接照搬到本片作为放行条件。Jarde 已有 reader 的解析与擦除验证，需补的是现有 class commit 的写位判据，不是第二套 Signature parser。

`ObjectSetter.put(Object)` 原源码的 `(T)value` 在冻结 javap 中已经没有 `checkcast`（T 擦除为 Object）。因此 JADX 输出缺少必要源 cast 的观测，不能归因为它删除了原 class 的 cast 指令；原 class 根本没有这样的指令。`CrossSetter`/shadow 也须按字节码擦除与源码作用域分析。未来恢复 unchecked 源 cast 属于独立带证明的源码适配片，本片按 spec 保守保留 erased 字段，避免凭元数据猜 cast。

root 独立补充探针在 `results/root-probes`：`ParameterShift`（long/double 前置）、`NullLocal`、`ParamReassigned`、`RawAllocationVariable` 的基线字段 T 导致完整源码编译失败；`ListWrong` 的参数虽然原 Signature 是 List<String>，基线实际发布 raw List，因此字段 List<T> 的源码赋值合法；`RawParamArray` 与 `StaticRawField` 亦是基线完整源码合法投影，应保留。最终运行结果由 `verification-root.md` 和 `root/summary.json` 记录，不能仅凭本段门控计为 JVM 行为验收。

## 已发布原始类上界的既有正例

root 实测 `RawBoundWriter` 的 `<R extends HashMap> void put(R raw, boolean flag)` 在基线实际发布方法 Signature，`Map<String,String>` 字段完整重编合法。最初仅按同一 binder 或原始参数 Class 证明的实现会错误丢弃这个已有字段投影。

修正复用 reader 的方法形式参数和现有 release 8 关系：仅当实际发布的源码参数是唯一同名方法变量、显式 class bound 是无参数的 Class、没有 interface bound、该 bound 与物理参数 descriptor 擦除相符，才允许对参数化 Class 字段使用已有 unchecked assignment。没有增加 bounds 推理引擎、类作用域上界替换或泛型子类型求解。参数化 bound、intersection/interface bound、目标类型变量和 release 9+ 的平台 widening 仍拒绝。

只读审计曾假设 `RawListMethod<T>` 的 `<R> R put(List raw, R marker)` 会发布 Signature；root 用真实 javac 和 CLI 证明它因为不是 direct parameter return 而拒绝，类似 void 变体也不在当前已发布形内。不能把该推测写成产品回归。真实回归锚为无 main 的 `root-anchors/RawBoundWriter.java`；外部 driver 读取字段，避免被既有 reader-consumer gate 混淆。

参数来源还要求 SSA BCI 唯一；canonical clone 造成同 BCI 多条 SSA 指令时，写来源统一未知。原始分配的各次 SSA write 不共享 ValueId，必须以 `Uninitialized.new_site` 关联 new/dup/constructor，再确认初始化后的 RHS 是实际 put 的唯一消费者；只比较 ValueId 会误拒绝既有 Map/List 初始化。
