# 二分查找/双指针/记忆化巡查（2026-10-05 root）——critical 第 16 锚 + 新诊断族

## critical 第 16 锚（旧值族）：`xs[w++] = xs[r]` 存储丢弃、自增保留

twoPtr（**双指针原地去零——高频算法形状**）：`if(xs[r] != 0){ xs[w++] = xs[r]; }`——渲染保留 `local1 = local1 + 1;`（w++）但**丢弃数组存储** `arg0[w_old] = arg0[r]`，诊断旧值族（"value local 1 held at BCI 17"）。**隔离编译 exit 0、`[0,1,0]` vs 原 `[1,2,3]`**。jadx 以 `int t = w; w++; xs[t] = xs[r];` 解。族计数 **16 锚 / 4 诊断族**。

## 新诊断族（第 5 族候选，可恢复性非 critical）：canonical block multi-owner

bsearch（**二分查找——闭区间收缩 + else-if 链 + 早退 return**，最高频算法）整方法拒：**"canonical block at BCI N on jsr path [] has more than one owner in the completed Region tree; the whole method is quoted"**——[诊断普查](../diagnosis-census/README.md) 29 模板外的新文本（region 树 canonical 块多 owner）。body 空=编译失败（缺 return）=第一不变量不违反（**安全**）；jadx 完整解（嵌套 if-else）。归档为**可恢复性锚**：region 树几何（与 irreducible 同层），锚计数不改 critical 族。

## 健康面（负结果）

- **递归+记忆化**（static HashMap + get/put + 递归双调用）完整恢复（装箱/条件/递归全过）；twoPtr 的 copyOf 收缩与循环条件精确。

## canonical 族判别补完（同日晚，CB/CB2/BX 三连探针）

- **CB 矩阵四形全恢复**（while+else-if+早退 / while+else-if 无早退 / while+单 if-else+早退 / 无循环 else-if+早退）——else-if 链/早退/循环任一维度都不单独触发；
- **CB2 出口消费也恢复**（`return -(lo+1)` 循环出口变量消费不触发）；
- **BX 决定性实验**：bsearch 方法**逐字节相同**（javap 逐条 diff 仅方法边界不同）——BS 类（含 memo 静态字段+clinit+twoPtr/fib）拒、BX 单独类**恢复**；
- **结论：canonical multi-owner 的真实触发条件是类级上下文**（同类其它成员的存在影响本方法的 Region 树构建）——**恢复确定性缺陷**（同一方法在不同类环境中结果不同），非方法形状本身。这是比 else-if 链更深的架构层线索（Region 树构建的类级共享状态/顺序依赖），窄片候选取证就绪。

## 处置

第 16 锚入四族 Requirement（旧值族新位点——写侧下标后缀）；canonical-block 族记 census+账本（第 5 可恢复性族）；bsearch 候选窄片（region 层）。

## root 静态调查补充（2026-10-06，无构建窗口内的读码排查）

对"类级上下文影响单方法 Region 树"的通道假设做了静态排除，收敛到需插桩/二分的候选：

- **budget 残量通道：基本排除**。`region.rs` 全部 budget 用途为 `poll`/`charge`（纯 stop 语义，无压力下形状降级）；且渲染账本显示 bsearch 是 BS 类**第二个**分析的成员（methods.1，ctor 之后），请求级累计消耗差极小。budget stop 的诊断码是 `StopReason::Budget`，与实测 `jre_region_ownership_overlap` 不符。
- **region::recover 的每方法输入：语义相同**。bsearch 无字段访问/无拼接/无分配/无异常表——`field::plan`/`chains`/`sites`（`init::sites` 的类级输入 `member_inner_targets` 两腿皆空）对本方法产出空计划；`class_fields`（BS 多 memo 字段）不被 bsearch 的计划触碰。
- **剩余候选通道**（下一构建窗口按序实验）：
  1. **类准备/binding 阶段的状态**（`analyze_method_ir` 之上的 class-source 管线，含 demand resolver 的 facts 缓存复用）；
  2. **上游 IR 构建的类上下文敏感性**（同指令字节、不同池布局/类名下 canonical 或 view 的构建差异）；
  3. **池布局敏感性**（bsearch 引用的池索引在两腿不同；若有按裸索引键控/比较的通道）。
- **判别实验矩阵**（MVP 二分，全部用 `--release 8`）：BS 去掉 main / 去掉 fib / 去掉 memo+clinit / 去掉 twoPtr 各一腿，观察 bsearch 翻转点；翻转后用 `--jarde` 诊断逐字比对定位产生点；必要时加 P3LOST 式插桩对比两腿的 regions 几何。**任何"已定位"结论须有最小门控实验支撑**（handoff 纪律）。
