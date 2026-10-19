## Context

动机和范围见 proposal。原始 debug/no-debug 的 80B Code 一致：slot3 写入 BCI1=0、37=1，读取 BCI68→69 append(Z)、78→79 ireturn/Z。已接受完整基线有 94 实际命令，原件 24 用例观测；Jarde 16 次完整编译均成功但 128/192 用例 stdout 不同；JADX 两次完整 JDK23 编译后 24 观测相符，JDK8 框架版本失败原样保存。这分别证明输入有效性、工具链边界、编译性和所选运行等价，不是整个特性的保证。

现有 Declarations::decide_types、accumulates_boolean 与 BooleanConsumption 已具全写/全读检查，literal 0/1 不是独立 seed，旧规则要求位运算 seed；调用参数不被接受。single-store 条件证明不能用于本跨循环双写。现有 concat append(Z) gate 只接受已证布尔，不应放松。root真实探针已通过（debug/no-debug同图）：whole slot3写V20/V52，退出Phi V11只合并二者，读取68/78；循环Phi为[V20,Itself]且无用户，回边41→15/47→5。既有消费证明在append(Z)为false、return/Z为true。见 results/same-run-ir-probe-root-v1；这只确认原始实例，不替代生产准入的通用闭合证明。

## Goals / Non-Goals

**Goals:** 在现有声明规划中增加一个有完整生产/消费证据的窄准入；debug/no-debug 同样有效；保留预算、取消、来源及拒绝。

**Non-Goals:** 不新增全局类型求解器、pass、公共探针协议或 CFG/Region 结构；不改整数 return 转换、任意 `(Z)` 方法参数、异常/cleanup 恢复或未知 map BCI。不因为全类编译成功宣称语义正确。

## Decisions

1. **先验真实 IR。** root 运行最小真实 class 探针，核精确 LocalVariable 的全物理访问与 reuse 归属、全部 store/load SSA、local Phi 输入及 loop edges。不得写手工 SSA 摘要替代真实分析。未知输入、非所属值、未计入写入均拒绝；必须明确检查现有证明是否已提供这些事实，只有缺失时才补最小闭合检查。
2. **复用声明决策。** 优先在现有 accumulates_boolean 路径增加 literal-only 准入，保留原位运算 seeded 规则和 fixpoint 语义。新分支排除 exception handlers 和 resource/catch slots，保留旧 seeded 路径；非参数且有实际写/读、所有 path 有证。slot_uses会跳过elided_cleanup和reuse归属None，故必须对真实SSA候选slot访问与全access表/精确owner作物理闭合核对，不能只信其枚举完整。全部写值须为直接 0/1（本片不泛化任意生产）；所有访问绑定同一确切复用变量。首片只检查实际读取值：候选已证 store 结果，或同owner/local slot且每个输入均为候选已证 store 结果的一层 Phi。live Itself、嵌套 Phi、Undefined/未知/stack escape 均拒绝；原件无用户的循环 Phi 不参与这条读取证明。只做按需受预算查找，不增加跨候选 Phi 索引、parents/grounded传播或通用solver。更深/活循环图只有出现真实缺口才另立证明。
3. **独立消费证据。** 至少一个真实 return/Z 或精确 StringBuilder/StringBuffer append(Z) 参数作为独立 seed。不能让候选自身、局部 copy 环或 ifeq 构成 seed；本片也不扩展ifeq/ifne消费准入。复用 BooleanConsumption 的全部读取证明，并仅为精确 append(Z) 增补正确 invoke kind/owner/name/descriptor/receiver/参数位置和 SSA value 身份匹配；旧拼接 gate 不变。需要兼顾旧调用者，不能把泛化的“存在任意布尔消费”替代“所有消费安全”。
4. **反例与停止。** 原始 fixture 及 debug/no-debug、不同位置的合法控制走真实 IR；至少验证写2、混合 append(I)/算术、仅分支、参数或复用歧义/未知 Phi 中实际适用的拒绝。预算/取消从真实准入路径触发，保留 Stop/usage/发布事务。root 对失败路径及原位运算/条件布尔回归进行对抗审查，不镜像实现编无意义测试。
5. **完整对照。** 原始全 outer/child/check 源码不裁剪；class/plainJAR×default/all×debug/no-debug 八 API。最终完整源码在 JDK8/23 编译并运行相同六个有限用例，包括 `{{7},null}` 命中后跳出控制，核输出/返回/异常及隔离加载。保留所有失败、原始与 JADX 证据；源码相同且精确绑定时复用已有证据。未映射 BCI 逐项记录，本片不顺带补来源。
6. **依赖与串行守卫。** Rust 标准库和现有 Runtime 足够，无需新库；复用原始 JADX fixture 时保留 LICENSE/NOTICE，不复制工具依赖进产品。root 独占共享写/Git/网络/Cargo/JDK/CLI；Luna 只写私有候选。上片 2684b4ee7 的 CI 源码采集关闭前不集成本片生产/fixture/指纹。Cargo 使用 5GiB 余量、target1GiB 上限、default stack、每秒 guard/finally exact clean，期间不共享写或混跑工具。

## Risks / Trade-offs

- [循环 Phi 或 slot reuse 被候选自身循环论证掩盖] → 先真实图核查，再证明全部生产根/所有读取，不接受未知来源。
- [旧seeded accumulator的SlotUse省略边界属于另项架构债务] → 本片只对新literal分支补闭合，记录后独立审计，不顺带改变旧路径。
- [扩展共享消费器使旧拒绝过宽] → 精确消费位置、独立 seed 与全读闭合分开核验，保留原规则回归。
- [恢复打印后仍有其它 fallback 或映射缺口] → 公共报告和完整源码运行分别验收，另记债务，不用编译成功遮蔽差异。

## Migration Plan

root 接受真实基线与 IR 后审 Luna 私有最小差异，集成并定向验证，再执行 fresh workspace、最终产品自身 CI 双 seed 和具名门。保留实际失败与修复历史；完成 spec 同步、strict、持久证据、账本和 handoff 后提交推送干净主线。尚未执行的步骤不提前勾选。
