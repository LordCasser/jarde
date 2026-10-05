# 正则 Pattern/Matcher 惯用法巡查（2026-10-05 root）——第 6 族第三形态（平台类型间引用转换）+ 高频可恢复性缺口

## 发现：`String → CharSequence` 引用转换证据缺失——正则三方法全吞

- **countWords**（静态 Pattern 字段 + matcher + find 循环）：`WORD.matcher(s)` 调用被吞（local1 无初始化），幸存 `n=0; while(local1.find()){n++;} return n;`——**隔离编译 exit 1（local1 未初始化）= 安全拒形**（非 compilable-wrong）；
- **firstMatch / groups**：整方法拒（explanation only）；
- 诊断文本与 #112/#113 第 6 族**逐字同族**："the parameter 0 of the invocation at BCI N is declared \\`java.lang.CharSequence\\` presents \\`java.lang.String\\` but the invocation requires \\`java.lang.CharSequence\\` and this layer has no safe reference conversion evidence"——**第 6 族第三形态：JDK 平台类型间 widening 引用转换**（前两形态：匿名类实参 CP$1→Comparator【critical #18】/ Thread ctor Runnable）；
- **cleaned 恢复**（replaceAll(String,String) 无 CharSequence 形参）；
- jadx 完整解（`WORD.matcher(str).find()` 链/group 提取）。

## 处置

- **可恢复性缺口登记**：正则匹配是 java8 文本处理最高频惯用法之一，三方法全拒——第 6 族可恢复性窄片候选（三形态合并：匿名类实参/String→CharSequence/Thread ctor，同一转换证据机制）；
- soundness spec 第 6 族场景补 String→CharSequence 数据点（SAFE 方向：幸存编译失败）；
- census 第 6 族条目扩形态 3。
