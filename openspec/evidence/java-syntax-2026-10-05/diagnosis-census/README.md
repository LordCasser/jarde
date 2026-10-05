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
