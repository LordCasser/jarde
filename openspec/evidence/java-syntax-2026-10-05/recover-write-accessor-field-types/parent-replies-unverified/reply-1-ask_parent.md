# 答复一（ask_parent，来源待 root 鉴别）

> 元数据：id=`01a107c1-12fc-7ab1-80f6-db5f703b2758:call_14cbb39a7910490293a8950b`，status=`answered`，
> subagent_task_name=`实现写访问器字段类型泛化`，target_session_id=`01a0f0b9-8484-7f92-88de-d9cc133415cb`。
> 原文逐字保留如下（未改写）：

---

**裁决：允许直接实施。这处改动属 design 决策 1 的字面范围，不是停手条件 (b) 的对象——我的预审计在这一点上有遗漏，你的定位是对的。**

逐点确认：

**1. 归属判定：这行门是硬编码 boolean 类型事实，不是"结构判据"。** 停手条件 (b) 保护的是"非类型结构判据"（单 SSA 块、无异常表、无 clone 块、BCI 布局、Operation 序列、`fields.claim(3)`、`field.owner == owner`、`!field.is_static`、预算——design 决策 2 列举的那些）。而 `!= if boolean_accessor { "Z" } else { "J" }` 是**描述符常量**，与 prove 里的 `field.descriptor != "Z"` 同类：它就是决策 1"把每处判据都硬编码 boolean……全部改为按描述符查表"应覆盖的最后一处，只是藏在消费侧而非 prove 侧。我的预审计只核了**渲染语义**（render_value 按 SSA 值、field_value 描述符驱动槽宽——那半结论仍成立），漏扫了这行描述符门。预审计遗漏不应反过来把决策 1 的字面对象变成禁区。

**2. 你的最小提案恰好是正确的形状。** `boolean_accessor: bool` → `expected_descriptor: &str`、门改 `!= expected_descriptor`，两个调用点各传 `&str`：J 姊妹路径传 `"J"`（与今天 `false` 臂**逐字等价**、行为逐位不变）；wrapper 传 prove 记录的表键。这正是"泛化一个已验证处理器"而非"改消费链语义"——参数从布尔开关变为它本就想表达的那个事实（期望描述符），没有引入新判据、没有平行结构、没有放宽任何东西。比我预审计的"消费链零改动"结论**更符合**决策 1 的本意。

**3. 边界照旧：** `LongAssignmentResult::prove`、`field_value`、全部结构判据零改动（你已列明）；boolean 先例（`PrivateFieldFamily`）与 J 姊妹路径渲染逐字节不变——这两条仍是你 3.2 零回退锚，**不可因这个裁决放松**。J 臂的逐位等价性请在证据里用双渲染 diff 实证一次（改动前后各渲一个 J 形 fixture，逐字节相同），不只是论证。

**4. 流程说明：** 这条裁决由 root 会话直接答复（你按既有纪律存档、注明"root 裁决：预审计遗漏的描述符门，归决策 1"即可，不必再标"来源待鉴别"——这是对你主动停手上报的正常响应，你的停手处置是对的：拿不准就问，比自行扩权好）。同时请在 design.md 决策 3 或 Open Questions 处追记一行：`assignment_result_statement` 的描述符门经停手上报确认为决策 1 范围（root 预审计遗漏，2026-10-05 裁决），保持 spec 与实现同步。

继续。
