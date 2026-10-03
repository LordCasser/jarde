# 单层折叠下声明/引用拼写不一致（2026-10-04，root FP probe 发现的既有缺陷）

在核实 `recover-nested-class-literal-values` 的验收阻塞点时，root 用一个**不含任何类字面量**的最小 fixture（FP）意外发现一个既有呈现缺陷。因它与 ncl 正交、且属**响亮失败**（不违反核心不变量），单独登记为独立债务，不混入 ncl 验收。

## 现象（root 实测，主仓 HEAD 二进制，无 ncl 改动）

fixture [fixture/FP.java](fixture/FP.java)：`FP` 含 `static class Mid { static class Leaf {} }`，两个方法经实例做结构反射：

```java
static class Mid { static class Leaf { } }
public static String viaInstance()      { return new Mid.Leaf().getClass().getSimpleName(); }
public static String viaInstanceChain() { return new Mid.Leaf().getClass().getEnclosingClass().getSimpleName(); }
```

原 class 运行 `java -Xverify:all`：`Leaf` / `Mid`（[fixture/orig.out](fixture/orig.out)）。

Jarde 家族口径（`class-source --input fam.jar --class FP`）呈现 [results/FP-fam.txt](results/FP-fam.txt)：

- **声明侧**：`static class Mid extends java.lang.Object { … }`（源码拼写，深度 1 被折叠）；**但 `Mid` 体内无 `Leaf` 声明**（折叠是单层，未递归到 `Leaf`）。
- **引用侧**：`return new FP$Mid$Leaf().getClass().getSimpleName();`（**池形**，因 `FP$Mid$Leaf` 是自嵌套深层名，折叠覆盖不到）。
- **0 处 `@bytecode` 引注、0 处 "not recovered"**（自称完整恢复），头部却带 "not a compilable project" 免责。

`javac --release 8 -cp .`（物理 `FP$Mid$Leaf.class` 在 classpath）重编 → **exit 1，"找不到符号 类 FP$Mid$Leaf"**。即声明侧拼源码形、引用侧拼池形，两侧不一致 → 不可编译。

## 性质判定

- **响亮失败，非静默偏离**：不可编译（javac exit 1）+ 头部免责，故**不违反** `recover-return-in-do-while-false` / handoff "不得可编译且行为不同" 的核心不变量。优先级低于 ncl 首版的静默偏离/NPE（那个已退回修正）。
- **根因**：成员折叠是**单层**的（深度 1 的成员折叠为源码形声明，深度 ≥2 的成员名在引用位保持池形）。同一机制也是 ncl 首版 N2 偏离的根因（见 handoff.md "池形类型名的结构反射陷阱"）——但 ncl 的偏离是**可编译**的（因 N2 的呈现路径使文本可编，而结构反射静默返回池名/null），FP 的偏离是**不可编译**的（`new FP$Mid$Leaf()` 引用了一个源码里不存在的顶层名）。两者的共同根因是"单层折叠 + 深层自嵌套名保持池形"。
- **既有、非本片引入**：FP 不含类字面量，主仓 HEAD 二进制即复现，与 ncl 的 decode 准入放宽无关。

## 处置方向（未立 spec，登记待议）

候选 `recover-deep-member-fold-spelling`（或并入既有折叠域）：让折叠**递归**到深层成员（`Mid` 体内声明 `Leaf`），使声明侧与引用侧的深层名一致（都源码形）；或在折叠不覆盖深层名时，令引用侧也退回可解析形（如全限定 `FP.Mid.Leaf` 若声明链完整）。**但这是折叠深度域的实质改动**（涉及 `member_inner.rs` 折叠判据、`class_source.rs` 成员装配、以及全部既有折叠片 zero-regression），且当前是响亮失败（非正确性 bug），故：

- **不立即立 spec**——优先级排在 ncl 守卫、interface-headers（Impl 桥隐藏的接入点）、5.3 混合匿名形（里程碑）之后。
- 待折叠深度域有其它驱动（如 corpus 频率证据显示深层成员类高频）时，与 ncl 的池形守卫一并纳入同一片分析（二者根因同源）。
- 与 ncl 的关系：ncl 的守卫（池形 ∧ 结构反射消费 → 拒绝）会**顺带**让 FP 这类深层名在结构反射位退回拒绝（响亮失败），部分缓解本缺陷的表现；但 `new FP$Mid$Leaf()` 这种非结构反射的引用位仍不一致，需折叠递归才能根治。

原 class 为行为基准（`Leaf` / `Mid`）。
