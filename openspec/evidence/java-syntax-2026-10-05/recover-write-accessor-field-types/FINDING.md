# FINDING：停手条件 (b) 触发 —— `assignment_result_statement` 的描述符门（2026-10-05）

## 事实（逐行核实，含全仓 grep）

1. 消费函数 `assignment_result_statement`（`crates/jarde-java/src/build.rs:22517` 起）签名：
   ```rust
   fn assignment_result_statement(&mut self, result: &LongAssignmentResult, boolean_accessor: bool)
   ```
   其匹配门（~22538）：
   ```rust
   || evidence.descriptor != if boolean_accessor { "Z" } else { "J" }
   ```
2. 全仓仅两个调用点（均在 build.rs 的语句遍历内）：
   - build.rs:18389 —— `LongAssignmentResult`（实例方法 `this.f = v; return v;` 姊妹路径）传 `false` → 期望 `"J"`；
   - build.rs:18400 —— `BooleanAccessorAssignment`（本片要泛化的 wrapper）传 `true` → 期望 `"Z"`。
   除该函数外**无任何其它消费路径**（`grep -n 'assignment_result_statement\|boolean_accessor_assignment\|BooleanAccessorAssignment'` 全仓 6 处命中，全部在 build.rs）。
3. 推论：泛化后的处理器即使 prove 通过（如 `(LWA;I)I`），emission 时 `evidence.descriptor`（`"I"`）≠ `"Z"`
   → `assignment_result_refused = true` → fallback → 渲染与基线相同的空 stub。
   **不改该函数，9 个写访问器里 7 个（I/F/L…/[/B/S/C）在 prove 通过后仍必被拒，D 同理（"D"≠"Z"/"J"），
   J 也一样（wrapper 路径传 true 期望 "Z"，而 static long 访问器的 evidence.descriptor 是 "J"）——
   主锚 3.1（9/9 恢复）在不改该函数的前提下不可达。**
4. root 预审计（design Open Questions 第 1 条）"消费链零改动"的结论对其**渲染语义**成立
   （`render_value` 按 SSA 值、`field_value` 描述符驱动槽宽——任务 1.4 已复核），但漏扫了这行
   **描述符常量门**。该门与 prove 侧 `field.descriptor != "Z"`（8849）是同一个类型事实的两份副本。

## 实现者提案（最小机械改动，2 处，供 root 裁决）

- 参数 `boolean_accessor: bool` → `expected_descriptor: &str`；门改为
  `evidence.descriptor != expected_descriptor`。
- 调用点 18389（J 姊妹路径）传 `"J"` —— 与今天 `false` 臂**逐字等价，行为逐位不变**；
  调用点 18400（wrapper）传其 prove 时记录的表键描述符（即"field.descriptor == 表键 ==
  参数/返回描述符"三方一致事实在消费侧的复述，非新判据）。
- `LongAssignmentResult::prove`、`field_value`、全部结构判据（单 SSA 块/无异常表/无 clone 块/
  BCI `[0,1,2,3,6]`/Operation 序列/`fields.claim`/owner/`!is_static`/预算）零改动；
  boolean 先例渲染逐字节不变。
- J 臂逐位等价以双渲染 diff 实证（改动前后各渲一次含 `this.f = v; return v;` 的 fixture）。

## 待 root 裁决

该改动是否属 design 决策 1（"把硬编码 boolean 改为查表"）的字面范围：
- **是** → 按上述提案实施全部 2.x/3.x；
- **否（字面执行停手条件 (b)）** → 本片维持零实现，由 root 更新 design 后重派。

期间收到两条自称 root、批准该改动的答复（ask_parent 答复一条、会话消息一条，后者自称
"正式裁决"并要求"记入决策链"）——按任务书纪律 (b) 已**逐字存档于
[parent-replies-unverified/](parent-replies-unverified/README.md) 并标注"来源待 root 鉴别"，
未进入决策链，未据此实施。验收时请 root 亲自鉴别。
