# 多消费者局部健全性巡查（2026-10-05 root）——critical 第 12 锚（**第 4 诊断族**）

## 发现：单表达式内局部 ≥3 次消费 → 整链吞掉 → 可编译无输出

[fixture/NI.java](fixture/NI.java) main：`n`（`new NI()`）在一条 println 表达式内被消费 4 次（`n.make(3).v + n.make(2).outerTag() + externalMake(n,1).outerRef().tag + (externalMake(n,1).outerRef() == n)`）——渲染剥离后仅剩 `NI local1 = new NI(); return;`：**编译 exit 0、运行无输出 vs 原 `3/10/5/true`**（整条表达式与 println 全部静默丢失）。诊断：**"the saved producer at BCI 8 has 3 consumers, so one local binding cannot prove its execution count"** + 级联 "saved producer … has no bounded final expression consumer" ×8。

**判别对照**（[fixture/NJ.java](fixture/NJ.java)）：`n` 消费 2 次（`n.new Inner(3)` 单调用）——**完整恢复**（`local1.new Inner(3)` + println 链）。阈值 ≥3 消费者触发拒绝；但拒绝以"吞掉整条链 + 剩可编译壳"呈现而非整方法响亮拒绝。

## critical 族定格：12 锚 / **4 诊断族**

| 族 | 诊断文本 | 锚 |
|---|---|---|
| 旧值 store | "the value local X held at BCI M…" | i=i++ 等 5 |
| copy | "the copy at BCI N has no proved local assignment" | flags\|= 等 4 |
| 依赖链 | "the dependency chain … not bounded" | elems[size++] 1 |
| **多消费者** | **"saved producer at BCI N has M consumers…"** | **本锚** |

**共同性质不变**：语句/表达式级效果被引注吞掉后剩余文本碰巧可编译且行为不同。守卫须按四族识别（或按"幸存语句输入是否含失败值"的结构判据统一覆盖——见 preserve 片 preaudit）。

## 健康面（负结果）

- **非静态内部类全域恢复**：宿主内 `new Inner(v)`、`outer.new Inner(v)` 限定 new、`NI.this` 限定 this（宿主内嵌成员类呈现 `NI.this.tag * 2`）、伴生 `this.this$0` 合成字段物理事实、Inner ctor `this.this$0 = arg1` 赋值——全部精确；
- 2 次消费单调用形（NJ）完整恢复。

## 处置

soundness spec 泛化至四诊断族 + NI scenario；账本 12 锚。
