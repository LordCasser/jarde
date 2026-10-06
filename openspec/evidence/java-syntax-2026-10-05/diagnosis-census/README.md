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
6. **普查后新族（2026-10-05 晚，[optional-chain 巡查](../optional-chain-patrol/README.md)）**：**绑定接收者适配**（"adapting this bound receiver would move its null failure from functional-value creation to invocation"）——`o.ifPresent(sb::append)` 绑定接收者=捕获局部；此前 DT-27 域只见整方法拒绝，本锚为**语句吞掉+方法幸存的 compilable-wrong 形**（第 5 主族；绑定到参数的方法引用恢复——失败面收窄到捕获局部接收者）。
7. **普查后新族（2026-10-05 晚，[comparator-anon 巡查](../comparator-anon-patrol/README.md)）**：**匿名类实参引用转换**（"the parameter 1 of the invocation at BCI N is declared `T` presents `CP$1` but the invocation requires `T` and this layer has no safe reference conversion evidence"）——匿名类作方法实参时池形呈现与声明参数类型的转换证据缺失，整条调用语句被吞（第 6 主族；伴生类本身恢复，失败只在宿主侧实参转换）。
8. **第 6 族形态 3（2026-10-05 晚，[regex-matcher 巡查](../regex-matcher-patrol/README.md)）**：**平台类型间 widening 引用转换**（String→CharSequence，`Pattern.matcher(s)`）——同诊断文本；正则三方法全吞（一幸存编译失败+两整方法拒，均安全）；jadx 全解——第 6 族可恢复性窄片候选（三形态合并）。
9. **第 6 族形态 4（2026-10-05 晚，[legacy-collections 巡查](../legacy-collections-patrol/README.md)）**：**lambda→JDK ctor**（`new PriorityQueue<>((a,b)->b-a)`，诊断异径 "allocation belongs to no shape this run verified"——同落点不同渲染路径）——ctor 位两形态（匿名/lambda）幸存文本均含未初始化局部=结构性安全；**四形态族簇集齐**，仅匿名→方法参数位为 critical。
10. **普查后新族（2026-10-06，[sync-return-timing 巡查](../sync-return-timing-patrol/README.md)）**：**monitor 释放时序漂移**——嵌套 synchronized 内层 return 的表达式求值被呈现到内层 `monitorexit` 之后（内层空块+return 外移）；全恢复无引注（非语句吞形），判别探针 `nN` vs `nY` 可编译错码；jadx temp 形有解——第 7 主族，窄片 preserve-monitor-exit-evaluation-order。
11. **普查后新族（2026-10-06，[array-element-field-receiver 巡查](../array-element-field-receiver-patrol/README.md)）**：**数组元素接收者类型丢失**——"the field access at BCI N is not one this run proved names the member its own receiver's type declares" + "the value at BCI M comes from the field access…"：aaload 来源接收者的字段读循环体/方法体被吞；普通类静态查找表幸存文本编译后容器静默为空（第 8 主族，可编译错码）；直接参数/局部别名字段读恢复（判别钉死），当前/伴生数组组件同败；jadx 有解。
12. **第 6 族最小机制面拆解（2026-10-06，[platform-interface-widening 巡查](../platform-interface-widening-patrol/README.md)）**：顶层具名类 `implements Comparator` 复刻 CP.byAnon 同败——排除 `$` 伴生变量；既有 snapshot-hierarchy 扩宽 design 明确排除"用户类→平台目标"（实现对目标也先取 header，平台接口不在快照即停）。最小单边片 recover-platform-interface-argument-widening：源类 header 逐字列出目标即证。java.io 平台→平台位点另被丢弃分配掩蔽，不混片。
13. **第 6 族 platform→platform 新面（2026-10-06，[java-time-temporal 巡查](../java-time-temporal-patrol/README.md)）**：`LocalDateTime→TemporalAccessor`（format 实参）、`→Temporal`（Duration.between）、parse 位——源与目标均平台且均不在快照，单边机制与既有表都不覆盖；java.time 为 Java 8 最高频新 API；jadx 全解。窄片 recover-temporal-argument-widening（表机制直接后继，零新机制）。