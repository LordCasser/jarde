## Context

现有能力见 `openspec/changes/recover-proved-integer-constant-names/` 与 `openspec/changes/recover-int-array-constant-names/`。候选在 `src/facade.rs` 的单次 root class preparation 中由完整字段表构造；只接收可写声明、`static final int`、唯一 `ConstantValue` 和有效 Java 名称，并去掉同名/同值歧义。root methods 的 AST 同轮保留，通过 `PhysicalMethodId` 与对应 method report 配对。

`project_class_source_integer_constant_names` 仅重写允许的整数 switch 标签、直接整数返回和现有直接 int-array 返回叶。参数和 statement 内局部名占用检查保留；array 路径继续做 current-body 与 staged member 检查、emitter replay 精确映射。最终 `source_text_with_method_projections` 先要求 ordinary writer 对当前完整 `report.text` 逐字重建成功，再提交 method text 和 derived spans。每个名称的 derived entry继续锚到 root 物理字段 identity/index 和 root 物理方法 identity/BCI，且最终范围必须位于输出文本中。

当前调用点只允许 `member_family == Absent`。该门原用于避免与 family child/source 的投影竞争，却把 `Refused { child: None }` 也视为有 child text。冻结的 `TestSwitchLabels$TestCls` 同时存在 self row 和 direct-child row；scanner 顺序使 root report 以 `RefusedNone` 结束，不持有 child。root 自身的物理字段、方法 AST与普通 root text仍可独立使用。

## Goals / Non-Goals

**Goals:** 只把整数名称投影的 member-family 准入从 `Absent` 扩为 `Absent | Refused { child: None }`。真实冻结目标 root 源码使用 `CONST_ABC` / `CONST_CDE`，并保留物理恢复事实与 derived source-map anchors。

**Non-Goals:** 不修改 `scan_family_root` 判定或顺序，不解释 refusal reason，不将 direct child 变成 nested source，不改变 child recovery，不改三个 sibling gates，不扩充 integer candidate/AST 准入，不做 outer/parent/inherited constant lookup，也不宣称反推出原源码 token 或整 CF12 完成。

## Decisions

1. **只改既有 gate。** 保持 `report.execution == Complete` 外门与三个 sibling family 的精确 `Absent` 条件。将 member family 单一 `Absent` 判断改为允许 `Absent` 或 `Refused { child: None, .. }`。不新增 predicate、enum、boolean sidecar、测试镜像 gate 或 refusal reason 控制流。
2. **由原投影证据承担语义证明。** `RefusedNone` 没有 child report，也没有该 family 发布的 child source text。候选和 AST来自同一 root 物理 class read；只做本类 compile-time integer field 的名称投影。direct-child 类型关系不改变 root 方法内引用本类静态常量的解析。现有完整 ordinary-writer equality 仍是最终提交条件。
3. **保留已有拒绝状态。** `Refused { child: Some(..) }`、`RefusedPair` 与 Prepared family variants仍不能进入该入口。三个 sibling 状态仍由同一处现有调用条件要求精确 `Absent`。不为这些状态复制生产 gate 的 `matches!` 或新造直接构造 `ClassSourceReport` 的测试。
4. **验证使用真实 fixture 与既有回归。** 在 `src/facade.rs` 的 `integer_constant_name_tests` 为真实 frozen class 增永久 reader/API 测试。用现有成员 family / sibling tests覆盖真实状态构造和保持物理报告的行为；这些独立 fixture不冒充具有候选的本目标端到端 gate 测试。对现有入口进行穷尽源码审查，核对每一支状态是否满足/阻止该 gate；若公共 API 不能产生“integer candidate 同时存在且某兄弟 family非 Absent”的报告，不造 public fixture、不加测试 hook，而在验证记录中说明这部分由现有 sibling gate 的精确条件与对应 sibling integration tests覆盖。
5. **比较固定完整源码，限制验收声明。** 使用冻结 `TestSwitchLabels.test` 输入，比较 upstream Java、固定 JADX 与修后 Jarde root/child 源码，并用既有 runner 的同输入结果校验可编译与行为。只报告此 fixture 的变化，不把本窄 gate标成整 CF12 完成。

## Existing API test inventory

永久新增的唯一核心目标测试使用 `integer_constant_name_tests` 现有 `source` / `source_with_snapshot` reader helper、`ClassSourceReport`、物理 identities 和 derived projection 类型。永久测试读取 frozen `TestSwitchLabels$TestCls.class`；不要求额外 javac、变异解码输出或生产测试接口。

以下既有测试是状态与不变量的真实 API 保护证据，可在任务验收时复跑/引用，不必复制它们的 fixture 或断言：

- `tests/member_family_identity.rs::child_body_stop_keeps_root_and_child_physical_coverage` 通过真实预算停止取得 `Refused { child: Some(..) }`，并核对 root/child 物理覆盖。
- `tests/member_family_identity.rs::capture_analysis_stop_keeps_completed_physical_child` 保留 `Prepared` 与已完成 child；`tests/member_family_identity.rs::selected_family_keeps_two_physical_reports_under_one_budget` 覆盖完整 `Prepared`；`tests/inner_class_static_mixed_folding.rs::n1_folds_stat_and_inner_together_with_the_synthetics_elided` 覆盖 `PreparedFold`；`tests/member_class_static_folding.rs::multi_child_family_folds_five_declarations_and_reproduces_the_baseline` 覆盖 `PreparedStatic`。
- `tests/member_family_identity.rs::declaration_pair_near_misses_never_publish_half_a_root` 覆盖 `RefusedPair` 与 `PreparedPair` 的近失边界。其余 `Refused { child: None }` 的 reader 行为由 `missing_or_duplicate_selected_child_refuses_family_without_losing_root` 覆盖，但新的目标测试使用冻结 `TestSwitchLabels$TestCls`，不把不同 refusal cause 合并成断言。
- nested enum 的 `src/facade.rs` 私有测试 `nested_enum_projection_is_proved_and_budget_stop_is_atomic` 与 `nested_enum_with_unproved_body_obligations_stays_field_by_field`；nested annotation 的 `tests/p3_nested_annotation_source.rs::proved_annotation_is_nested_and_the_physical_child_is_preserved`、`malformed_child_kind_refuses_projection_but_keeps_the_child_report`；anonymous-interface 的 `tests/class_source.rs::proved_two_level_anonymous_interfaces_project_as_one_nested_root_expression` 及相关 `anonymous_interface_projection_refuses_*` 测试覆盖 sibling 各自非 Absent 的真实分支。

这些 fixture测试覆盖 sibling 或 member 状态自身的公共 reader 行为，但未必有同类整数候选，因此不声称它们逐个实测了新入口的候选保护组合。通过一次不改代码的 gate/source audit确认三个 sibling条件仍精确要求 Absent，`Complete` 与 candidate early-return未变，Prepared/RefusedPair/childSome不匹配新准入。任何尚无真实 API fixture可组合到候选上的分支应列为源码审查，不得另建复制入口逻辑的 unit test。

## Library and licensing

实现复用本仓既有 `jarde_reader` / class-source API、Java AST 与 writer，不引入依赖或新的外部代码。本片只需复用现有投影，JADX作为冻结输出和算法参考，无需新增移植代码或依赖。整体工程仍允许参考其实现算法；若后续确需复用代码，保持原许可证与归属。测试直接复用仓库已冻结的 class 输入，不另引入第三方代码或 fixture；实现与测试都不从 JADX 复制代码或 fixture。对照只读取仓库已保存的 JADX 输出产物。

## Risks / Trade-offs

- **Refusal 状态表达宽泛**：只放行 `child: None`，有 child report 与 Prepared claim仍被阻止；不依据 reason 文本判断。
- **self row 与 direct child 同时存在**：这是目标 fixture。family scanner仍可 Refused，child仍不装入 root；只投影 root 自身方法中的本类字段别名。
- **名称遮蔽/值歧义**：复用已有 candidate 与 AST checks，不创建第二个绑定算法。
- **派生源码位置失效**：复用 ordinary writer全文 equality、method/field物理 anchors和输出 span validation；物理恢复报告不改。
- **外类常量绑定**：明确排除 outer lookup。`Inner → outer` 是独立架构债务，须单列后续工作，不混入此 change。
- **验证过度声明**：仅声称冻结目标和实际执行的相关检查通过；不预设聚合测试数量，不宣称整 CF12完成。


## Selected Inner boundary

同一门也适用于独立选中的 Inner：当前实际报告是 Complete/RefusedNone/三个 sibling Absent，只有本类 `CONST_CDE_PRIVATE=3294` 候选。允许命名该 case 是现有 same-class 证明的自然结果；`return 2748` 无本类候选，必须维持数字。单 class snapshot 不含 outer 字节，InnerClasses 只给关系，不能证明 outer 字段及 ConstantValue；不能把 Prepared family 路径当作该真实 fixture 已可达。未来跨类名称需要显式同时提供两个物理类的已有统一 artifact/snapshot 入口，另立 change，不扩当前 patch。

永久唯一核心测试同时读冻结 root/Inner，分别验证各自物理字段与方法、组装别名和派生 spans；不得把 JADX local/global 同值冲突政策作为本类别名的额外阻止条件。单类的合法本类命名可优于该对照输出的保守选择，完整 Java 8 编译与运行决定有效性。

## Delivery prerequisite

conditional switch 自身CI和干净主线关闭后才应用补丁。按5GiB free/target1GiB一秒进程组守卫执行根验证，复用已有单次stat scanner及记录器，完成后保留冻结CLI/源码/raw并清理target。实现是单一既有gate，不新增依赖、入口或实体；预算/原子性契约继续由既有投影处理。
