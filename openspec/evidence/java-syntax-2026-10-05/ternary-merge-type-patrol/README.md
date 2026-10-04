# 三元汇合类型与短路呈现巡查（2026-10-05 root）

## 发现一：三元分支异型汇合（响亮拒绝，已证缺口）

`static Object poly(boolean c){ return c ? Integer.valueOf(1) : "s"; }` 整方法拒绝：**"the two values joined at BCI 13 do not have a conditional Java type this run can prove"**——两分支值（`Integer`/`String`）汇合到 `Object`，呈现层无法证明条件表达式的静态类型。

- **行为安全**：响亮拒绝（带引注），非静默偏离；
- **jadx 有解**：拆为 `if (z) { return 1; } return "s";` 的语句形（[results/jadx-IF.java](results/jadx-IF.java)）——语句化是回避汇合类型证明的既有合法路径；
- 判别：同型分支（`dense`/`chain` 的 int 三元、嵌套链）**全部恢复**——缺口仅在**引用型异型分支汇合**（LUB 到非公共具体父类形）。

**处置**：登记为独立窄缺口（呈现域——把汇合证明不了的引用型三元拆成 if/return 语句形即可，与 jadx 同构）。不阻塞其他域。

## 发现二：短路布尔组合的位技巧呈现（行为等价、呈现退化）

`(a && b) || (!a && !b)` 渲染为嵌套数值三元 + `% 2 != 0`（字节码的 boolean-int 翻译被如实展开）。**行为验证通过**（`true` 双向一致，前 4 值 `4/9/true/2` 逐行相同）；jadx 呈现为布尔短路式（更优）。属**呈现质量债**（非行为缺口），与已登记的 simplify-proved-boolean-conditional-returns 同族——不立项，数据点并入该域。

## 发现三：else-if 链呈现为嵌套 if + 平铺 if（行为等价）

`guard` 的三段 else-if 渲染为两组嵌套 if + 尾 return（CF-02/CF-03 的 else-if 首片已覆盖此形态语义；jadx 用条件取反 + 三元混合）。行为一致，呈现取舍，不立项。

## 处置汇总

新登记一个窄缺口（发现一，三元异型汇合→语句形拆分）；发现二/三为呈现债数据点。全部冻结于 [results/](results/)。
