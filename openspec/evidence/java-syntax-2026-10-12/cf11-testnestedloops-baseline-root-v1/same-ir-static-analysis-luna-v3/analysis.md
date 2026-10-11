# TestNestedLoops foreach 首拒诊断准备（静态分析 v3）

## v2 的 facts 驱动需要纠正

v2 建议在 `report.rs` 测试中手动构造 `MethodFacts::new("test", "(Ljava/util/List;Ljava/util/List;)V", 2)`，并按需添加 locals。这个 slot 数不正确：`parameter_slots` 包含实例 receiver；`src/class_source.rs:1785-1808` 计算签名参数槽时显式把 `u16::from(!is_static)` 加到 descriptor 参数槽数。因此 `test(List,List)` 的值为 3。`src/facade.rs:41717-41747` 的真实 `recovery_facts` 直接取 `MethodDeclaration::parameter_slots()`、方法 access flags、DeclaringClass 名称和 class flags，并挂入该 Code 的 `debug_locals`。`src/class_source.rs:15742-15749` 也有实例方法 `new(named, String, int, 3)` 的现成例子。

更重要的是，手工 facts 即使把 slot 修成 3，仍可能漏掉 class-source baseline 用到的 Code 调试事实、完整 descriptor 类型、声明和请求 profile。此诊断不应以手工近似 facts 声称复现 baseline。仓库没有 `src/recover.rs`；实际 facade orchestration 在 `src/facade.rs`。

## 最小真实驱动入口

未来临时诊断应由 `Engine::class_source()` 公共入口驱动 frozen JDK 23 原始 `cf11/UpstreamNestedLoops.class`，请求仍采用 baseline 的 `ClassRef::Name("cf11.UpstreamNestedLoops")`、`SnapshotAll`、`SingleClass`、`RuntimeProfile { java_release: 8, multi_release: Disabled, layout: Generic }` 和 loader `app`。现有 class-source 测试 helper `src/facade.rs:12729-12758` 已展示完整模式：`Engine::open(ArtifactInput::bytes(...))`，以打开的 snapshot 建 `ClassSourceRequest`，然后调用 `engine.class_source(...)`；只需将类名、冻结 class bytes 和 profile 绑定到此次 baseline，无需直接拼造 recovery facts。

调用链为：

- `Engine::class_source` 在 `src/facade.rs:1305-1317` 委托 `class_source_with_evidence`；后者在 `1325-1402` 构建请求环境并调用 `prepare_physical_class_source`。
- `prepare_physical_class_source` 为每个目标成员走 `recover_prepared_member`。它在 `src/facade.rs:19996-20007` 对同一份 `PreparedClass` 调 `jarde_jvm::analyze_prepared_method_ir`，随后在 `20097-20116` 把这次分析交给 class-source recovery/presentation。
- `recovery_from_with_class_candidates` 在 `src/facade.rs:41355-41408` 调 `recovery_facts(analyzed.ir().declaration(), analyzed.ir().code(), method)`，并从实际请求复制 profile。当前 class 有 Code debug table 时，`method_parameters` 的 fallback 分支不运行；facts 来自 Code 本身。
- `recovery_facts` 在 `src/facade.rs:41717-41748` 从真实 declaration 取名称、descriptor、`parameter_slots()`、access flags、declaring class/class flags/inner-class members，并把 `debug_locals(code)` 放入 facts。`debug_locals` 在 `41781-41844` 从同一 Code 的 LVT 与 LVTT 读取 slot、名称、起止 BCI、descriptor 和唯一匹配的 generic signature；它不是可选近似。Javap 和 frozen input 可用于检查记录，但不要手工重造其最终 `DebugLocal` 列表。
- `crates/jarde-java/src/report.rs:11364-11379` 从这些 facts 派生 declaration、descriptor parameter/return types，随后 `11379-11427` 将同一 `canonical`、SSA、operations、Code、profile、参数、receiver、类型、access flags 和 debug locals 送入 `build::build`。class-source 路径在 `src/facade.rs:41537-41595` 调 `recover_for_class_source_with_anonymous_ast`；这保持了真实 class-source pipeline，同时内部候选构建仍用目标 method 的这份 IR/facts。

所以最小 same-IR 首拒诊断是在 `build.rs` 的候选调用/拒绝路径临时记录事件，而让实际测试从上述 `Engine::class_source` API 进入。`loop_region` 递归构造与调用顺序、候选入口及拒绝出口的位置仍按 v2 所列：`crates/jarde-java/src/build.rs:18879-18991, 21224-21715`。记录应限定物理方法 `test(Ljava/util/List;Ljava/util/List;)V`，并保留 raw test BCI；baseline javap 的 outer/inner branch BCI 分别为 13/42，但事件应输出实际候选收到的 BCI，不应先验认为两者一定相等。候选内正常失败归为 `None`，budget/cancel 是 `Err`，临时事件应把两者区分。

此建议只说明诊断如何使用真实 class-source 事实；没有写插桩，也没有执行测试。仍未确认 inner 是否到达 candidate、两层具体首拒、或 generic Signature marker 与 loop refusal 是否有关。baseline 的 `ordinary_generic_source_unproved` 是另一个投影/来源状态，不能当作 foreach 拒绝理由。

## 若只能走 report::recover

`report::recover` 可以让同一 `MethodIr` 进入 `build::build`，但单独手造其 `RecoveryRequest` 不能自动证明 facts 与 class-source baseline 完全一致。至少要从相同原始 class declaration/Code 精确提供：实例 method 的 3 个 parameter slots、方法及 class flags、declaring class、descriptor 及 descriptor-derived argument/return types、完整 LVT/LVTT 记录（包括范围与唯一匹配的 generic type metadata），以及 Java 8/Generic/SingleClass 对应的 profile。还需要注意 class-source member/evidence sidecars 不由普通 method-only request 自然给出。遗漏任一项时，结果只能叫同 IR 下的 report-path 探测，不能叫 class-source baseline 首拒。公共 `Engine::class_source` 已能保留这些事实，优先使用它。

## 冻结 baseline 与静态边界

root 已提供的 baseline execution SHA-256 是 `5635e60b5565a19ba4fab116d0b6ae73f72e62568e0761edeff2365291c95435`。原始 JDK 23 target class SHA-256 是 `8ece89e4d38751c20529622b57417afc8601368fc20078656ea2b24c17ed8231`，对应 javap SHA 是 `c1e623df5fa6d4c6b4125dc2073debd9bcc1ba15ece01b30eb604be91393a272`。同一报告显示 Jarde 四个 profile 都保留两个 `while`，且呈现循环 body effects 和循环后的 cleanup；这只是输出观察。Javap 的 branch 位置及 v2 的 candidate gate 说明仍是候选诊断线索，不是候选内部 trace。

本次只进行了源码/文件只读核对并准备私有分析稿和 pins。没有修改共享代码，没有运行 Git、Cargo、Java、JADX、CLI 或 Atlas，也没有运行待议诊断。此文中的调用链与候选观察是源码静态推导，不是新实测结果。
