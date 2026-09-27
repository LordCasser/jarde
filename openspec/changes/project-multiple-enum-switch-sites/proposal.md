# Proposal

当前 class-source facade 在一个方法包含多个 remap-array 枚举 switch 时会拒绝所有枚举标签投影。固定审计确认完整源码仍可编译且行为正确，但两个 switch 都保留整数标签；审计当时被 facade 的 blanket refusal 截断，没有执行 map proof。新最小双 enum fixture 又证明：移除 blanket refusal 后，现有逐表 helper proof 会拒绝共享 `<clinit>` 的第二张表初始化。这个变更同时收敛处理这两个阻塞点。

扩展范围包含 grouped AST 发布和满足双枚举正例所必需的严格联合 map proof：javac 会把同一方法不同 enum 的 remap-array 初始化放进一个 synthetic helper `<clinit>`，现有单表证明器将其它表初始化视为额外效果而拒绝。联合证明只接受候选引用的全部不同表构成封闭集合，并对完整 `<clinit>` 一次证明所有选中表；任何未选表、未知指令、额外副作用或结构不符仍拒绝。随后把同一方法内的全部候选应用到同一份本轮 AST 副本，并一次发射。保持整方法原子性：任一站点证明或发射失败时，发布原恢复方法，保留其中所有整数 switch。不得为联合证明扩展候选发现或依赖读取。

## Scope

- 支持同一完整 class-source 方法内两个或更多已独立证明的 Java 8 enum remap-array switch 站点。
- 支持同一 helper `<clinit>` 中候选所选全部表的严格联合证明；不得容忍候选之外的表或未证明操作。
- 保留整方法全成或全拒以及现有停止/预算行为。
- 增加聚焦的双站点正例和部分证明拒绝例。

## Out of scope

- 放宽跨类 helper 或 enum 的既有物理身份、常量与控制流证明规则；不扩大依赖读取。
- switch expression、直接 `ordinal()` Smali/Java 21 lowering、单个 switch 多 selector 或任意 AST 重建。
- 改变已经通过 DT-31 replay 的单站点投影行为。
