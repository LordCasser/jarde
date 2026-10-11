# CF-11 后续最小诊断准备

本目录只有只读架构审阅和未应用、未编译、未运行的候选。`archive-pins.json` 绑定原字节。不能用它们关闭实现或验收任务。

引用类型审阅 v1 的“唯一读点直接返回”说法超出了旧 raw 的 SSA 覆盖；v2 已明确更正。旧日志只闭合 Phi2→Load29，尚未展开 Stack Value14→Return30 的所有消费者。后续诊断先补这条读链；写入端可以考虑复用现有 LocalVariable 和逐写遍历形成 Object 候选，但不得据此预定实现或无条件 Object 化。合法 CharSequence/StringBuilder receiver 和数组读写反证已列出；无需为该窄型引入通用 LUB 或新的 identity 服务。

补读链的 private observation-v3 已准备并由 root 静态核对：复用原日志函数，只增加 Load29 每个写出值及其完整消费者 instruction/operation 的36行观测。runner 要求当前固定 Value14→Return30 完整事实，原4744→4745→4744、原字节恢复与5/1GiB守卫不变。patch SHA256 `26d9b02565f8ea5e8793776f3a5a0c69d9adf9c75b0e8502e01a309dbf1a090c`、runner `81aff04094719ca2323fc174d8b0c6e4bae330cd028d4169870080aeb3020a3e`。候选未应用/编译/运行，不替代旧失败日志，不闭合读取证明。

方法泛型诊断只观测固定 TestNestedLoops。root 核对原 `class_source::ordinary_parameterized_declaration`：Signature 解析/擦除证明之后，缺少 same-run body proof 和 candidate 时，`candidate.ok_or_else` 产生当前准确拒绝文案。候选将记录同次 Program/SSA 的首个拒绝源行及 Signature 投影终态；沿用既有测试的 default/all/no-rule-details 三请求，原 fixture 已在4744闭包，没有新增测试输入。

方法泛型 patch SHA256 `c736eb2b70475d2a00f10d8eebb5e92c990910a5c4a413ed4cf32f65e9c3ac11`；runner SHA256 `6c4ad2598a898b104560bb4aed07f547584107d1656d2f4d8a9696eb5ee6c6ed`。root 静态核对补丁、日志字段接口和相关 Debug 实现，尚无 Rust 编译证据。运行时仍须5GiB最低余量、target1GiB、一秒守卫、默认Rust栈、finally固定cargo clean与两文件原字节恢复。当前余量不足，不启动本候选。

当前 foreach 产品73adae72的自身CI和最终主线审计保持独立；该准备不改变6/8状态，也不增加71/612完成数。
