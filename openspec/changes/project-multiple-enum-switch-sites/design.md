# Design

class-source 枚举候选扫描已按物理方法分组。当前 `candidates.len() != 1` 的提前分支会在后续逐站点证明前记录拒绝。后续代码已经按方法暂存证明结果，但 `emit_class_source_enum_switch` 每次都会克隆并发射原 AST 的单个站点；之后用另一份完整方法文本覆盖前一份暂存文本，因此无法保留多处改动。

双枚举同方法正例暴露了现有 map proof 的前置限制：javac 将相关 enum 的 values/newarray/table-store 初始化串接在同一 synthetic helper `<clinit>` 中，旧证明器按单表扫描并拒绝其余 Invoke/field。新增严格联合证明路径：收集当前方法候选所引用的全部不同 helper table；仅当它们恰好封闭地覆盖该 helper `<clinit>` 中的所有表时，才对整个指令流扫描一次。每张表按真实 owner/name/descriptor 分组；根据实测 BCI 顺序验证 `values()->length->newarray int->putstatic`、每个 enum 常量字段/`ordinal()`/`iastore`、独立 `NoSuchFieldError` handler、CFG/SSA 路径及唯一最终 return。每条物理指令和 handler 必须恰好归属某一选中表或最终 return；handler 可以转移到下一组 values 调用，但不得跳过、重叠或留空。每张表仍用其物理 enum 定义及常量证明映射，不按字段名或声明顺序推断 key。候选未覆盖的额外表、其它效果、未知指令、重复/缺失写入或不连续结构均拒绝。联合扫描只读既有候选需要的定义，不展开依赖。

候选身份仍由物理方法 identity、switch/read BCI 和所选 table identity 组成。增加 grouped emitter：只克隆一次该方法本轮 AST，按每个候选的 switch BCI 定位目标，验证相同的 `ordinal()` selector 形状及该站点的全部标签，再一次发射完整方法体。只有所有候选都成功时，才暂存这份方法文本和全部证明标记。匹配失败、方法不完整、任一联合 map proof 拒绝、预算停止或取消时，方法内所有 switch 均保留整数路径，并保留逐站点结果。不同 helper 的单站点候选继续走原单表证明路径与 emission，以维持单站点输出稳定。

候选身份仍由物理方法 identity、switch/read BCI 和所选 table identity 组成。不得根据 table 字段名、参数顺序、enum 声明顺序或候选顺序关联标签。回归 fixture 使用两个 enum 类型和两张 helper table，并比较原始、固定 JADX 与 Jarde 的完整源码编译及 `-Xverify:all` 执行。部分证明负例改变一个站点的证明前提，确认兄弟站点不会被部分投影。
