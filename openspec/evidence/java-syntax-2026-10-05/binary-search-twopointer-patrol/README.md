# 二分查找/双指针/记忆化巡查（2026-10-05 root）——critical 第 16 锚 + 新诊断族

## critical 第 16 锚（旧值族）：`xs[w++] = xs[r]` 存储丢弃、自增保留

twoPtr（**双指针原地去零——高频算法形状**）：`if(xs[r] != 0){ xs[w++] = xs[r]; }`——渲染保留 `local1 = local1 + 1;`（w++）但**丢弃数组存储** `arg0[w_old] = arg0[r]`，诊断旧值族（"value local 1 held at BCI 17"）。**隔离编译 exit 0、`[0,1,0]` vs 原 `[1,2,3]`**。jadx 以 `int t = w; w++; xs[t] = xs[r];` 解。族计数 **16 锚 / 4 诊断族**。

## 新诊断族（第 5 族候选，可恢复性非 critical）：canonical block multi-owner

bsearch（**二分查找——闭区间收缩 + else-if 链 + 早退 return**，最高频算法）整方法拒：**"canonical block at BCI N on jsr path [] has more than one owner in the completed Region tree; the whole method is quoted"**——[诊断普查](../diagnosis-census/README.md) 29 模板外的新文本（region 树 canonical 块多 owner）。body 空=编译失败（缺 return）=第一不变量不违反（**安全**）；jadx 完整解（嵌套 if-else）。归档为**可恢复性锚**：region 树几何（与 irreducible 同层），锚计数不改 critical 族。

## 健康面（负结果）

- **递归+记忆化**（static HashMap + get/put + 递归双调用）完整恢复（装箱/条件/递归全过）；twoPtr 的 copyOf 收缩与循环条件精确。

## 处置

第 16 锚入四族 Requirement（旧值族新位点——写侧下标后缀）；canonical-block 族记 census+账本（第 5 可恢复性族）；bsearch 候选窄片（region 层）。
