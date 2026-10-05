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
