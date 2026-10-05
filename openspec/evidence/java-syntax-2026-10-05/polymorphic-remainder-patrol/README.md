# 多态分派残留巡查（2026-10-05 root）

## 健康面（负结果）

[fixture/CL.java](fixture/CL.java)（`--release 8`）：`crossCall(P)` 参数多态分派恢复；`poly(List<P>)` 泛型直传恢复（raw + Signature 拒绝注释=既有域）；`main` 中 `(CL$P) new CL$Q()` 上转型调用点如实。

## 发现：三元异型汇合的子类上转型变体（已立项域，锚+1）

`choose(boolean){ return c ? new Q() : new R(); }`（Q/R 同父 P）被拒——诊断文本与 [三元巡查](../ternary-merge-type-patrol/README.md) 的 `Integer vs String` 形**不同**（"the copy at BCI 7 has no proved local assignment" + "the value at BCI 21 is the entry state of stack depth 0"——两 new 值各自落到栈汇合点，无 "two values joined" 文本），但**根因同族**：汇合点无可证条件类型。`main` 里的 BCI 49-57 引注同因级联（asList(new P()) 数组初始化 + choose 调用）。

## 处置

不新立（三元片已立项）；**变体锚已补入该片 proposal**——实现时该形是否与 `Integer/String` 形同落点待 task 1.1 插桩确认（若不同路径，先报告）。
