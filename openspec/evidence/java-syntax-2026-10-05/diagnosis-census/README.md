# 诊断族系统普查（2026-10-05 晚，root，第 100 前沿）

对 java-syntax-2026-10-05 全部 60+ 巡查渲染的 `// the …` 诊断行做模板聚类（BCI/数字归一化）：**29 个模板**。

## 四主族频次（critical 守卫对象）

| 族 | 主诊断 | 级联伴随行 |
|---|---|---|
| copy | "the copy at BCI N has no proved local assignment"（**55**，最高频） | — |
| 旧值 store | "the value at BCI N is the value local N held at BCI N…"（10） | — |
| 依赖链 | "the dependency chain from BCI N to final consumer N is not bounded"（9） | "comes from an Other"（16）/"produced by a saved declaration…could not commit"（6）/"not part of the provable subset"（12） |
| 多消费者 | "saved producer at BCI N has N consumers…"（6） | **"saved producer at BCI N has no bounded final expression consumer"（40，第二高频）** |

## 普查结论

1. **spec 枚举缺口已补**：第 4 族在 spec 只枚举了主行，未枚列级联行（40 次）——preserve-postfix-fallback-soundness 诊断不改 scenario 现包含两行文本；
2. **伴随行均为级联**（Other/saved-declaration/not-provable/entry-state-of-stack 在全部已档案例中与四主族同现，未见独立触发留可编译错文本）——四族封闭性维持；
3. **第 5 扩宽表候选线索**：`java.util.Collection` 位（OB.flags 的 `retainAll(EnumSet)` 级联行，Enum 锚同方法）——**已否定（[collection-widening-probe](../collection-widening-probe/README.md)）**：retainAll/containsAll/disjoint/泛型擦除四形独立恢复——该行为 Enum 锚级联伴随，不升表；
4. 已知独立域确认在案：irreducible（3）、conditional-type join（5）、short-circuit 共享消费（1）、bitwise 类型混合（2）、widening 四表族（2+2+1+1）、array-init 异构（2）。
5. **后续新增（2026-10-05 晚，[binary-search 巡查](../binary-search-twopointer-patrol/README.md)）**：**canonical block multi-owner**（"canonical block at BCI N on jsr path [] has more than one owner in the completed Region tree"——二分查找 else-if+早退形触发；body 空=安全，可恢复性缺口，region 树几何层）——第 5 个可恢复性族（非 compilable-wrong）。**判别补完**：方法形状维度（else-if 链/早退/循环/出口消费）均不触发；逐字节相同的方法在单独类恢复、在含 memo 字段+clint+多方法的类中被拒——**真实触发条件是类级上下文**（恢复确定性缺陷：Region 树构建存在类级共享状态/顺序依赖）。
