# recover-proved-for-condition-expressions — 只读对抗审查

## 范围与结论

只读检查 `/Users/lordcasser/workspace/projects/jarde` 当前 checkout 的 `region.rs` 与 `build.rs`。不运行 Git、Cargo、JDK、JADX 或 Atlas，也未改仓库文件。依据调用者提供的真实同 IR 诊断：TestNestedLoops2 已形成 1 test / 10 nodes / 12 edges / 28 SSA / 19 trace；外层 header4、init3、update41、phi 门通过，首拒为 `Invoke List.size@6`；内层因 preheader 不是 induction store 被拒，继续保留 while。

**建议方向成立，但应将结论限于现有证明链。** 外层候选的首拒位于 `for_header_candidate` 条件指令扫描：它只准 `Load`、`Push`、`Arithmetic`、终端 `Comparison`，因此拒掉已由 `test_is_pure` 认可的 `Invoke` 等条件 producer。生产调用链先执行 `test_is_pure(header, test_bci, true)`，再调用 `prove_for_header`；扩大这段已有扫描以接纳同一 predicate 已认可且属于 branch value dependency 的 producer，是最小闭环。保留 init/update/phi/CFG/predecessor/invariant-write 证明和 Builder 的 clause/update 检查，不增加第二套 purity walker、证书或 IR。

## 相关源码

- `crates/jarde-java/src/region.rs:9481-9516`：`test_is_pure` 构造 `condition_value_bcis` 和块内 value read 计数，并逐条调用 `test_expression_instruction`。
- `region.rs:9530-9597`：producer 白名单。`Invoke`、Field Read 只有在 `condition_bcis` 中才可过（9593）；ArrayLength 还要求 caller 允许且唯一读者（9594）；ArrayLoad / ArrayElementLoad 要求唯一读者（9595）。Push/Load/Arithmetic/Negate/NumericComparison 先于 dependency 检查直接放行（9540-9549）。Store、Increment、Copy 不在普通 producer allowlist；只有既有 store-dance / postfix 特例可过（9552-9576）。
- `region.rs:9890-9927`（以 `nl -ba` 查看精确区间）：`condition_value_bcis` 从 branch 所读 SSA 值递归回溯到本 block producer，故 Invoke/Field/Array 的普通放行受限于条件表达式依赖，而不是任意同块调用。
- `region.rs:12629-12783`：实际 header-tested while/for 路径；对单测试分支在 `prove_for_header` 前先调用 `test_is_pure(..., true)`（12762-12763），随后才求 ForHeader（`prove_for_header` 调用点约 12783）。生产唯一 caller 的次序满足本方案前提。
- `region.rs:13409-13433`：`prove_for_header` 做既有预算 charge 后委托 candidate。
- `region.rs:13435-13710`：`for_header_candidate` 保留唯一自然 latch、更新形状与 induction input、所有 latch incoming edge / body-effect 特例、唯一 preheader、最后一条 init store、stack producer 单 use、两输入 header phi 与 init/update 单 use；13668-13700 是需要扩展的条件指令匹配。13689-13694 当前只接受 Push/Arithmetic/Comparison；load 同时识别 induction slot 与 invariant slots。后续 13713 起的全方法扫描拒绝循环内额外 induction slot 写、invariant slot 写，并限制循环外读写；不要动这些门。
- `crates/jarde-java/src/build.rs:18781-18835`：Builder 通过 `test_expr` 从原 test BCI 构造条件，并把条件放在循环头，每次测试按原循环频率执行；Continuation 决定极性。
- `build.rs:18845-18875,18895-18962`：ForHeader 仅接受一个测试及可生成 Java init clause 的既有 init；更新必须生成单个 assignment，且局部变量名与 init 一致，随后组装 `StmtKind::For`。

行号为当前 checkout 精确位置；上述第二处行号 9890-9927 可直接用 `nl -ba crates/jarde-java/src/region.rs | sed -n '9890,9930p'`复核。

## 对抗风险核对

- **单次求值与调用副作用：** `test_is_pure` 的 Invoke 是 effectful，但限制在条件 SSA 依赖里；Builder 在 loop condition 原位通过 `test_expr` / `render_value` 写出，不把它提升到 init，也不移入 body，所以一次条件求值仍只调用一次，调用对字段/数组的修改仍发生在相同迭代边界。`List.size()` 属于典型每次检查都重新执行的条件调用；不应把它缓存到循环外。对一般 Invoke，条件表达式内的 Java 操作数顺序由 SSA operand 树表达；本次调用者给定的 IR 首拒信息未提供更细的 operand-use 统计，适合实施时继续以同次 trace 确认 `List.size@6` 的栈结果为单一 branch dependency。
- **多 use：** test predicate 自身不要求 Invoke/Field read 的输出唯一使用，这是它相对 ArrayLength/ArrayLoad gate 的明确差异（9593 vs. 9594-9595）。但本方案不是放开所有块内 Invoke；`condition_value_bcis` 要求它位于条件依赖中，测试块中非表达式操作仍需过 predicate；普通字节码复制需经过 `Duplicate`，而 Duplicate 不在普通白名单，只有已证明的 store-dance 特例能吸收它。对提供的 `List.size@6`，标准 `invoke; push; if_icmp*` 结果是单 branch operand，不会复制调用。若真实 trace 显示该值有重复表达式消费，单靠当前 caller 不足以保证调用不被 Builder 展开多次，应拒绝该形状；这不需要为目标场景新建通用分析机制。
- **Field Read 与 body mutation：** 对象/静态字段可在循环体中变化。把 Field Read 原样留在头部条件可正确观察每次 body 后的值；若读取值仅用于当前比较，现有语义足够。不要把该 read 移至初始化。volatile 并发读取以及 receiver 求值的异常也受益于保留原条件位置；重复展开仍按上条边界处理。
- **ArrayLength / element loads：** NullPointerException、越界异常及读取发生顺序是语义的一部分。现有 predicate 已将 `ArrayLength` 限为 `allow_array_length` 且恰好一个 block reader，将两类 array load 限为恰好一个 reader；这里已有 caller 用 `true`，因此可依赖这一单用门并保持内联位置。`ArrayLength` 单消费者也避免同一个可能抛异常的读取被 AST 展开两次。
- **Negate / NumericComparison：** 已是 `test_expression_instruction` 的无额外写入 value producer（9540-9549），但 `for_header_candidate` 目前遗漏它们；加入候选扫描须服从 test predicate，而不是另行猜测它们的纯度。注意 branch terminal 的 `Operation::Comparison` 与栈上的 `NumericComparison` 是不同 Operation；当前 candidate 已需接受前者作 branch（13689-13694），应同时允许后者作依赖 producer。
- **continue 与 update：** Builder 的 Java `for` 会在 continue 时执行 update。Region 先将已证明的 `update_block` 作为 body frame 的 continue target（约 12787-12804），candidate 要求唯一 latch、incoming edge 与更新前 body-effect 的形状；`build.rs` 再要求 update 是单一 assignment。只改 test producer scan不会改变这条控制流映射。
- **内层反例：** 内层因 preheader 最后一条不是 induction slot 的 Store，既有 `initial` 检查拒绝（约 13590-13600），与条件 Invoke 的白名单无关；扩大 test op match 不会绕过该 return-None。保持此门即可继续输出 while。
- **局部写与已有表达式 assignment：** caller 中 `test_is_pure` 对普通 Store/Increment/Copy 仍拒绝，存储 dance / postfix snapshot 是明确成对的既有例外；ForHeader 不应把这些 op 添加到 scan。`while` condition 的赋值表达式是已有独立路径，Builder 走 `for_header == None` 时保留其 while 表达，不要求本改动改写它。候选仅在 ForHeader 形状成立时变成 for；若 ForHeader 候选失败，已有 LoopForm::While 和 condition Builder 仍可消费同一条件。

## 最小设计边界

改动应限 `for_header_candidate` 的 header-test op match：让 `test_is_pure` 当前允许、且条件 producer 类型可由 `condition_value_bcis` 命中的 Invoke、Field Read、ArrayLength、ArrayLoad、ArrayElementLoad、Negate、NumericComparison 通过；不扩 Store/Increment/Copy，不改其它 initializer/update/phi/predecessor/invariant-write 和 Builder clause/update 验收。Predicate 已由生产 caller 先执行，且用同一 caller 提供与 while 相同的数组异常单用约束。

需要在实际实现验收中把真实 trace 的 `List.size@6` 栈输出及 branch 消费关系写清楚。若确认为单 branch consumption，上述已有 caller 对此目标充分，不需要引入新的通用 purity 设施；若不是，保留该形状为 refusal，避免因调用复制改变调用次数。

Root复核：v1正文开头init/update数字误写已在此副本纠正；本次IR实际ValueId22定义在block4/BCI6，SSA值表仅一个use为同block branch@11，与instruction输出一致。根验收不依赖v1的典型字节码推测。
