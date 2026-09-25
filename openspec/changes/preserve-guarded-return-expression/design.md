## Context

见 `proposal.md`。`guard::monitor` 已证明正常路径唯一的 `monitorexit`、随后紧邻的 `return`、返回值在受保护体内产生以及异常处理器的释放路径；`Region::Guard` 把返回写入 `StmtKind::Synchronized` 的 body。随后 `Builder::prepare_deferred_bindings` 在生产者到最终 `return` 的同块区间调用 `has_independent_boundary`，目前把 `monitorexit` 视作独立效果，给 `getfield` 安排 `saved0`。`p3_sync_return` 的精确文本断言因此失败。这个规划器正确阻止其它独立效果导致的延期求值，不应全局把 monitor 指令设为透明。

## Goals / Non-Goals

**Goals:** 只对已由同一个同步 Guard 认领的正常退出及其内部返回，避免无必要的保存局部；保持其它独立效果、异常和来源顺序。

**Non-Goals:** 不改 TWR、普通 try/catch、跨 region 值提升或不完整 monitor 的准入；不把所有 `monitorexit` 设为纯指令，不重构 SSA 或一般值绑定规划。

## Decisions

1. **从既有 Guard 证明传递窄所有权。** 在 Builder 准备绑定前，沿已恢复 Region 收集返回形状的 `Shape::Monitor`，记录该正常退出 BCI、对应返回 BCI 与 body 范围。正常退出 BCI 应由 `guard::monitor` 的已证结果显式交接；不要从相邻 BCI、指令名称或 `facts()` 中的多个 monitor 退出猜一个。可在现有 `Shape::Monitor` 增加该字段，不建立第二套监视器分析或通用效果豁免表。收集仍按已有遍历预算/取消计费。
2. **只在对应值的放置判断中略过这一次退出。** `has_independent_boundary` 检查的最终 consumer 必须是该 Guard 的返回，生产者必须在其 body 范围，退出 BCI 必须是该 Guard 已证明的正常退出且落在生产者与 consumer 之间。只有这一个 BCI 可视为结构自身实现细节；body 中其余调用、存储、读取、检查或未知操作仍是边界。如此既不把生产者跨独立效果移位，也不因已由 Java 同步块表达的退出再次引入局部。
3. **原子性与来源不变。** 不删除实际 monitor 指令或改 SSA。返回表达式仍由原 `return_expr` 在同步块内渲染，Guard 的 `OriginSet` 继续携带进入、正常/异常退出与返回 BCI；若监视器证明、值来源、作用域或预算任何一项失败，保持原有拒绝/保存路径，不发布半个同步语句。现有 `preserve-deferred-value-order` 对普通独立效果的保护继续有效。
4. **以输入行为区分结构效果和独立效果。** 冻结 `Locked.locked` 的 Java 8 class 与 runner，要求直接 `return this.n`，并对照原/JADX/Jarde 完整类重编执行。另以 verifier-valid 反例把有副作用调用插在值产生后、正常退出前，要求该调用的次数、返回旧值和异常优先级不变；证明不了时允许完整引用，不能为了去掉 `saved0` 输出错序文本。不得仅用当前字符串断言代替执行比较。

## Risks / Trade-offs

- [Risk] 全局将 `monitorexit` 透明化会重算值或改变异常顺序 → 只豁免同一 Guard 的唯一正常退出和对应返回；其它效果仍由既有绑定判断处理。
- [Risk] 只看 BCI 区间可能把另一监视器的退出误归入当前语句 → 通过 Region Guard 的所有权与精确正常退出身份交接，不靠邻近或名称。
- [Risk] 额外预处理突破预算或引用不完整 → 复用有界遍历，失败沿已有停止/拒绝路径，正文与来源一起提交。
