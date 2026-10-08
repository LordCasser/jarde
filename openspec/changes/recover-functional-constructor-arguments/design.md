## Context

见 proposal。root 门控只在 init 参数扫描中允许 SSA 依赖链里的 Operation::InvokeDynamic，未修改 renderer，两个编译腿的 FunctionalConstructors 与 IntBox 整类已重编译/运行并与原类相同。它确认缺口在结构组合，而不是 lambda 算法缺失。

## Goals / Non-Goals

**Goals:** 复用同一运行的 new/lambda 证明，补齐参数 effect 与 handler 覆盖条件，按完整类验收。

**Non-Goals:** 不迁移 lambda planner 到 init，不做 owner 对照表，不解决更广泛的 generic Signature 投影或可空 bound receiver。主线类输出有保守 generic 返回投影记录，即使剥离文本编译通过也不能称泛型已恢复。

## Decisions

1. **结构与语义各自证明。** new@1 的 verify 只允许实际构造参数 value_dependency_bcis 到达的动态调用，与普通 Invoke 相同；后续 new_expr→render_value→lambda_expr 仍负责 bootstrap/SAM/handle/capture 与适配。无须提前复制 lambda plan 或增加第二条识别流程。未知动态站点结构即使闭合，也不能获得猜测函数文本，必须由既有拒绝与 producer-retention 机制保留完整依赖区间。
2. **动态实参复用 handler 覆盖检查。** 为本构造实际接纳的动态参数设置局部事实，与 embedded_concat/array/nested 检查合流，读取 allocation 至 constructor 的每条指令及唯一 consumer 的 handler ordinal 向量。任一不一致拒绝；不用生产位置代替最终表达式位置。拒绝诊断明确动态参数边界。
3. **类型与类级投影保持单一来源。** 构造实参继续用物理 ctor descriptor 的 invocation_argument；它已为函数工厂与 required 参数添加 cast、固定重载。class_source 继续决定 helper 是否可内联、改名或保留。不把 signature 拒绝消失作为本片通过条件。
4. **JADX 提供经验而非新依赖。** 本地 CustomLambdaCall/InvokeCustomBuilder/InsnGen 分离工厂 descriptor、impl handle、SAM 与 surrounding invoke 的方式与本架构一致；采用通用组合而非 PriorityQueue 特判。没有复制代码，无新库、许可或维护负担；现有 Rust proof 与 writer 足够。
5. **各层边界不变。** reader/decode 读取原始 class facts，runtime profile 决定 dialect，frame/SSA 验证值，init 验证结构，lambda 验证函数适配，builder/class_source 恢复文本；不修改 raw refs 或其他层。

## Risks / Trade-offs

- 动态工厂可能有创建时 effect → 保留 SSA 参数依赖、闭合区间、唯一消费、handler 覆盖与 renderer 的 capture replayability 检查；无关 InvokeDynamic 仍拒。
- nullable bound ref 有创建时 nullcheck → 本片不声称支持，冻结反例；不得为了通例越过 copy/check 证明。
- 仅看 text 或 quality 会漏掉副作用 → 全类重编译，-Xverify:all 外部 Driver 对照；原始与两种反编译结果独立比较。
- planner 的结构站点和实际文本呈现不是同一含义 → 反例以最终 bytecode/origin 与无伪造函数文本判定，不用单个 new record 掩盖下游拒绝。

## Migration Plan

无 API 与依赖迁移。小范围实现/测试后由 root 独立验收、主线提交推送；回滚撤销本片即可。
