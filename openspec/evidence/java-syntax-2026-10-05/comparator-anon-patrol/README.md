# Comparator 家族巡查（2026-10-05 root）——critical 第 18 锚 + **第 6 诊断族**

## critical 第 18 锚（新族：匿名类实参引用转换）：`Collections.sort(c, new Comparator<User>(){...})` 整条语句吞掉

byAnon（**匿名 Comparator 实参——pre-lambda 排序最常见形**）：sort 调用语句**被完全吞掉**，幸存 `new ArrayList(copy); return c;`——**隔离编译 exit 0、渲染 `[30, 10, 20]` vs 原 `[10, 20, 30]`**（可编译错：排序静默丢失，第一不变量违反）。

诊断文本：**"the parameter 1 of the invocation at BCI 17 is declared \`java.util.Comparator\` presents \`CP$1\` but the invocation requires \`java.util.Comparator\` and this layer has no safe reference conversion evidence"**——**第 6 诊断族（匿名类实参的池形呈现 vs 声明参数类型的引用转换证据缺失）**。伴生 `generic_source_shape_unproved`（类名无可拼源形）为类级注记（池形名债务域）。

## 健康面（负结果）

- **byNameAge**：`Comparator.comparing(User::getName).thenComparing(User::getAge).reversed()` **全链恢复**（方法引用 companion 内联+cast 链+默认方法链式调用）；
- **byLen**：`Arrays.sort(String[], lambda)` 双参 lambda 内联恢复；
- **CP$1 伴生类**本身 quotes=0（compare 体恢复）——失败只在**宿主侧实参转换**；
- 行为 `[bo:30, al:40, al:20]/[al:20, bo:30, al:40]/[a, bb, ccc]` 一致（byNameAge/byLen）。

## 处置

soundness spec 补第 6 族锚场景；census 记第 7 条；账本 **18 锚/6 族**。
