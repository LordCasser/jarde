# 候选实现记录

本文记录待 root 验收的实现候选，不是验收结论。change tasks 仍为 2/8；本片没有勾选任务。

组合入口借用 `ChildArrayFacts`，针对一条已配对的 `aastore` 调用受限构造器证明。数组证明仍负责数组结构：构造站点先在候选范围内暂存，只有完整 initializer 链和联合 ownership 检查成功后，才逐个 move 到普通 `Sites` census。递归构造站点保存在父站点的有界树中，并随完整数组候选一起展平移交。

数组元素的存储值必须是构造器调用写出的确切完成引用。候选路径从该 ValueId 的构造器定义读取 receiver 的 `Uninitialized { NewSite }`，再回到当前元素物理区间中的分配点；不会把完成引用的定义 BCI 当作 `new`。身份检查覆盖 `new`、`0x59`、唯一未初始化 token、`dup` 的两个不同输出、确切 constructor receiver、保留的绝对 stack slot、该位置写出的完成类引用，以及唯一配对 store use。

候选扫描和 SSA use 在实际遍历处计费，包括 member qualifier、依赖链、handler coverage、构造器/store 身份验证和 `ChildArrayFacts` 的 source/member 读取。预算停止作为 `StopReason` 向上传递；证明不成立作为 refusal 处理。既有 array-only helper 保留原计费路径。组合表达式只允许已闭合的窄原子通过通用 single-use 检查：构造器身份脚手架、已递归证明的构造站点，以及完整 inline child-array 链；普通参数 producer 仍须满足唯一 use 和完整 containment。

当前测试代码包含 `sequence()` 的构造器/store 正例、消费 inline `Arrays.asList` 数组的 collection 正例、低 `AnalysisSteps` 预算和预取消断言。focused 控制还读取冻结的 `NestedControls.class` 与 `BoundaryControls.class`：嵌套元素要求 outer/inner 两个 Site 各自移交，outer 的 `finished_value` 必须等于真实 `aastore` 值，两个 `owned` 与 `instance` 身份分离；同一 nested SSA 上要求错误完成值（分别注入真实 array、index operand）、错误 allocation、错误 block position、非 `dup` BCI 均拒绝。递归预算断言直接调用受限 `verify_array_store`，不经过 initializer candidate 的发现/paired-store 扫描；outer 在 BCI 10 charge 一步并进入 inner verifier，limit=1 时 inner 首条扫描 BCI 14 的 charge 应停止。边界控制确认第二元素实际含 `i2l` 且 Long 构造器实参 SSA 定义就是该转换，再检查 array initializer 与 pending Sites 都为空。

本轮又加入两个 `sequence` 两腿 classfile Code 变体：重复 index 只改 BCI 20 的 `iconst_1` 为 `iconst_0`；descending 交换 BCI 5 与 20 的两个常量字节。两者都要求完整数组候选与 pending Sites 零提交。重复 index 变体另独立证明首个 new/store；descending 首个 index 已是 1，因此测试刻意不声称完整候选进入首元素构造验证，只用未变更原 class 的首元素证明作为 counterproof。额外 stack reader 变体按 `code_span` 将 `aastore@18` 替换为 `dup_x2@18; aastore@19; pop@20`，同步 Code 长度与 Code attribute 长度；从两腿真实 SSA 断言 constructor 完成值由 dup_x2 读取、两个同类 alias 分别流入 store/pop、array retained alias 到下一 dup@21。完整 candidate 与 pending Site 必须为空；定向 `array_store_consumes_site_value` 对 stored alias 与原 constructor completed ValueId 分别拒绝，而未变更源的同一 guard 通过。

另加入 HandlerControls 两腿受保护范围变体，只将 Code exception-table 的 `start_pc` 从 0 改成 BCI 12。测试在原 count=1 与完整 tuple 锚定后断言仅此 u16 改变；从 SSA 验证 array allocation 的 handler 集由 `[0]` 变空、mark/init/store 仍为 `[0]`，相关 source BCI 保持同一 block。直接 `verify_array_store` 仍应成功，只有组合 proof 因 array effect closure 不同而不提交 initializer/Site。以上新增控制都还没有运行。

额外 reader 与索引控制只在测试内基于各自 javac8/javac23 原 class bytes 派生，不改冻结 fixtures。额外 reader patch 按 `MethodCodeFacts.code_span` 精确定位，确认无 branch、handler、Code 子属性，检查 `max_stack >= 5` 后将 `aastore` 替换为三条指令并同步 Code 长度与 Code attribute 长度；没有按错误草稿提升 max_stack。HandlerControls 由另一轮双 javac 生成并冻结，测试只改 exception-table entry 的 `start_pc` 两字节，另三项原 tuple 保持不变。

这些新增测试还没有运行。focused-v2 曾有两处编译错误和一个 unused-mut；root 后续报告 focused-v3 与正向完整 family 已通过，但本次新增负控尚待 root 串行验收。为消除生产编译 dead-code 警告，未被生产调用的测试 wrapper 改为 `cfg(test)`，无调用的 null-check 非计费 wrapper 删除。本文不声称本轮负控已通过，也不声称 task 2.2 已完成或实现已验收。

仍未覆盖的负族包括独立的 source-block split、旧数组写入组合，以及 `closedNumberBoundary` 的类型拒绝呈现闭环；后者由 integration 范围负责。错误 stored-value 控制直接把真实 store 的 array 与 index operand ValueId 分别作为错误预期值传入 verifier，没有扩展接口。也没有把普通旧数组写入的证据扩大为构造元素组合控制。新增索引、extra-reader 与 handler controls 都还待 root 执行验收；本文不据测试代码宣称这些控制通过，也不宣称 task 2.2 已完成。
