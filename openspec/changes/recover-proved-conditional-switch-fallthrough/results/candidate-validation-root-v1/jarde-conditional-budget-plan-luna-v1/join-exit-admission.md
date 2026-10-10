# 最小准入修正说明

补丁只处理 prove_switch_fallthroughs 逐 arm DAG walk 中的“非 Transfer/Comparison 节点直接流向 switch join”情况。

改动为：
- 在每个 arm 的现有局部状态里增加一个 bool。
- 遇到不可呈现的 join 出口时置位，继续执行原有 successor 入栈、worklist 分类与后续 ownership 验证。
- 仅在该 arm 完整 walk 后，同时存在 case-entry exit 和不可呈现的 join exit 时拒绝。

这允许没有 case fallthrough 的普通 arm 自然落到 switch join。该路径由既有 switch group 末尾的 break 表示，因此 store@169 -> join@171 不需要虚构 SwitchBreak。若同一 arm 还有通往另一 case entry 的 fallthrough exit，则它也有一个不能由组尾 break 单独表达的 switch join 出口；仍然拒绝，避免把混合路径误写成已经证明的 break。

其余路径保持原样：重复/环路仍由原来的 color/work 检查拒绝；多 case exit 仍由 exits.len() 检查拒绝；所有节点仍进入 closures/owned 并接受相同 incoming/ownership 校验；canonical 全边、parallel-row、path 与终端约束没有改变。没有新增结构、API 或常驻观测代码。

补丁上下文取自本次只读检查时的 live crates/jarde-java/src/region.rs。Root 应在 live helper 上按上下文审阅应用；未修改仓库，也未编译或实测。
