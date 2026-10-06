# 循环累积/数组操作 + CharSequence 实参扩宽巡查（2026-10-05 root）

## 健康面（负结果）

[fixture/SB.java](fixture/SB.java)（`--release 8`）——**反编译器经典难点全部通过**：
- **循环携带 StringBuilder 累积**（`b.append(x).append(',')` 链式）完整恢复；
- **String 循环重赋值**（`r = r + x`，每轮新 builder）呈现为字符串拼接 `local1 = local1 + local3`——忠实；
- `System.arraycopy`（含 Object cast 实参）、`Arrays.copyOf`（扩容）、**varargs 双向**（`spread(int...)` 收集 + `spread(1,2,3)` 展开）全部恢复；
- `main` 的多段拼接呈现为 saved0/saved1… 链——既有域。

## 发现：String→CharSequence 平台接口实参扩宽缺失（响亮拒绝，已证缺口）

`static String join(String[] xs){ return String.join("-", xs); }` 整方法拒绝：**"the parameter 0 of the invocation at BCI 3 is declared `java.lang.CharSequence` presents `java.lang.String`"**——`String.join(CharSequence, CharSequence…)` 的首参要求 String→CharSequence 上转型，而 `platform_reference_argument_widens` 的 `DIRECT_EDGES` **只覆盖 java.util 集合树**（build.rs ~24948，注释自述"its java.util counterpart"）；`java.lang` 侧只有 Throwable 一族（`java_lang_throwable_widens`）。**jadx 完整恢复**（`String.join("-", strArr)`）——有解。

**判别**：这不是集合/Throwable 域；是 `java.lang.CharSequence` 接口族（String/StringBuilder/CharBuffer 实现）。同类 API 高频：`String.join`、`CharSequence.subSequence` 参数、`Appendable.append(CharSequence)` 等。

## 处置

登记第 4 个新证窄缺口（呈现域：平台实参扩宽表的 java.lang CharSequence 族补充——与既有 java.util 表/Throwable 通道**同构的第三张小表**，零新机制）。窄片立项 `recover-charsequence-argument-widening`（root 随后提交）。

## 处置（2026-10-06，change `recover-charsequence-argument-widening` 落地后重渲染）

`SB` 的 `join` 现在整方法恢复（`refusals = 0`，整类 0 引注）：[`results/jarde-SB-after-charsequence-argument-widening.txt`](results/jarde-SB-after-charsequence-argument-widening.txt)。
写出 `java.lang.String.join((java.lang.CharSequence) "-", (java.lang.CharSequence[]) arg0)`——**两个位点**：首参的
类行（String→CharSequence）与第 2 参的**数组位**（String[]→CharSequence[]，同一事实在数组位置的投影；实测只放行类行时
第 2 参仍在 BCI 3 被拒，故数组位是主锚恢复的必需伴随，见 change 的 verification）。同批落地的 Serializable
表覆盖多重界锚（[generics-edge](../generics-edge-patrol/README.md)），`stream-chain` 的第 5 位点见
[该巡查](../stream-chain-patrol/README.md)。负例（封闭表外的实现者）保持拒绝。
