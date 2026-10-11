## 1. Gate 与范围

- [ ] 1.1 在 `src/facade.rs` 现有 `project_class_source_integer_constant_names` 调用 gate 中，只把 member-family准入从 `Absent` 扩为 `Absent | Refused { child: None }`；不解析 reason、不增加 helper/predicate/state，不改三个 sibling gates、Complete gates或候选早返回。
- [ ] 1.2 对照入口逐项审查 `Absent`、`Refused { child: None }`、`Refused { child: Some(..) }`、`RefusedPair`、所有 Prepared variants；确认只有前两种可进入。确认 childSome与Prepared状态仍由原 family report/presentation通道负责。
- [ ] 1.3 记录独立债务：外层 `Inner → outer` 常量绑定不在本变更范围；不根据本类投影结果推导 enclosing/inherited 常量名。

## 2. 永久 reader/API 回归

- [ ] 2.1 在 `src/facade.rs` 现有 `integer_constant_name_tests` 中，为冻结的 `TestSwitchLabels$TestCls.class` 增加唯一核心目标测试。用现成 `source` / `source_with_snapshot` reader helper 和 class-source API，断言 root execution Complete、member family是 `Refused { child: None }`、三个 sibling family均 Absent；不检查或固化 refusal reason；同一核心测试独立读取冻结 Inner，核本类私有3294的命名与 outer-only 2748仍为数字。
- [ ] 2.2 在同一真实报告中断言 root 完整源码对 `f1(I)I` 输出 `case CONST_ABC:` / `return CONST_CDE;`，物理 method recovery text仍为 `case 2748:` / `return 3294;`。按物理 fields报告核对两个字段的唯一 identity/index和值；对两条 derived projection逐项核对确切源码 slice、Field anchor、同一个 `f1(I)I` MethodPoint anchors及 BCI 1/20；确认 root 输出不包含已装配的 `Inner` source；独立 Inner 唯一派生case锚为自己的字段与f1 BCI1，physical case/return仍是3294/2748，assembled return仍2748且无outer CONST_ABC。
- [ ] 2.3 复用既有真实 API测试验证状态覆盖，不创建测试专用 gate、测试镜像 `matches!`、测试 hook或伪造 `ClassSourceReport`：
  - `tests/member_family_identity.rs::child_body_stop_keeps_root_and_child_physical_coverage`：真实 `Refused { child: Some(..) }`；`capture_analysis_stop_keeps_completed_physical_child` 与 `selected_family_keeps_two_physical_reports_under_one_budget`：`Prepared`。
  - `tests/inner_class_static_mixed_folding.rs::n1_folds_stat_and_inner_together_with_the_synthetics_elided`：`PreparedFold`；`tests/member_class_static_folding.rs::multi_child_family_folds_five_declarations_and_reproduces_the_baseline`：`PreparedStatic`。
  - `tests/member_family_identity.rs::declaration_pair_near_misses_never_publish_half_a_root`：`RefusedPair` / `PreparedPair`；`missing_or_duplicate_selected_child_refuses_family_without_losing_root`：`Refused { child: None }` 公共reader状态。
  - nested enum：`src/facade.rs` 的 `nested_enum_projection_is_proved_and_budget_stop_is_atomic`、`nested_enum_with_unproved_body_obligations_stays_field_by_field`；nested annotation：`tests/p3_nested_annotation_source.rs::proved_annotation_is_nested_and_the_physical_child_is_preserved`、`malformed_child_kind_refuses_projection_but_keeps_the_child_report`；anonymous-interface：`tests/class_source.rs::proved_two_level_anonymous_interfaces_project_as_one_nested_root_expression` 与 `anonymous_interface_projection_refuses_*` 系列。
- [ ] 2.4 对上列 fixture 的证据边界作记录：其职责是验证各真实 family状态与物理/投影行为；不声称每个 fixture还同时含整数名称候选、因此逐个端到端触发本入口。若现有 public API 无法形成某个“候选存在且 sibling 非 Absent”的组合，仅通过审读既有调用点证明相应 gate仍精确要求 sibling `Absent`；不得复制 gate表达式来模拟，也不得新增不真实 fixture。复核 `project_class_source_integer_constant_names` 原有 candidates-empty、Complete、方法 AST/quality、shadow/duplicate、ordinary-writer全文重建和 derived-anchor路径未被改动。

## 3. 冻结源码与行为对照

- [ ] 3.1 用仓库冻结的 `TestSwitchLabels.test` 输入复核完整 class bytes、root/child物理身份和已有 baseline；比较 upstream Java、固定 JADX 与修后 Jarde root/child源码。确认对照差异限于每个独立所选class自身可证明的常量名：root CONST_ABC/CONST_CDE及Inner CONST_CDE_PRIVATE；family拒绝、child分离与outer-only return2748仍在。JADX提供输出与算法参考，本窄gate无需新移植代码或依赖。
- [ ] 3.2 使用仓库固定 runner 对原 class、固定 JADX结果和 Jarde完整结果执行相同的 Java 8 编译/`-Xverify:all`/运行输入，逐项核对行为；保存实际源码/class摘要、runner结果和物理 anchor事实。仅记录实际观测，不预填通过数量，不将一个 case 的结果宣称为整 CF12验收。

## 4. 仓库验收与交付（root执行）

- [ ] 4.1 在5GiB机器余量/target1GiB一秒守卫下，对改动文件执行仓库要求的格式检查；运行同范围 Clippy 与有限的整数常量、class-source/source-map、family相关回归。根据共享代码影响和本仓要求决定是否补全 workspace 检查；记录准确命令与结果，不预设聚合数字。
- [ ] 4.2 运行冻结 CLI / full replay 对照并检查完整源码差异；按本仓 OpenSpec strict 流程验证 proposal/spec/tasks与实现一致。确认结论只覆盖该 change与实际运行的冻结 case，不称整 CF12完成。
- [ ] 4.3 在本地实现、检查与冻结对照均可审查后，由root为实际产品改动创建准确提交并推送；随后独立验收该提交对应的自身CI全部job/step/测试结果，不借用conditional或其他提交CI。提交不得包含 private 草案或无关改动。
- [ ] 4.4 按 root 当前工作流清理 main checkout、worktrees及handoff遗留状态；确认交付分支/提交、工作区和相关运行目录处于预期状态，再完成交接。记录清理事实，不改写未涉及的架构债务。
