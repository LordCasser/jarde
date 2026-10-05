# 手动 Iterator 循环巡查（2026-10-05 root，负结果——前增强 for 惯用法全过）

## 探针

[fixture/IR.java](fixture/IR.java)（`--release 8`）：**手动 Iterator 循环 + `it.remove()`**（老代码无处不在）、`ListIterator.set()`（双向迭代写）、`removeIf(lambda)`（现代形）。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- 手动 Iterator 循环完整还原：`local1 = arg0.iterator(); while(local1.hasNext()){ String s = (String) local1.next(); if(s.length() < 3){ local1.remove(); } }`——接口调用链、`next()` 的 `(String)` checkcast、**void 方法调用作语句**（remove）全部精确；
- ListIterator.set 同构恢复；removeIf 谓词 lambda 内联（`(Predicate)((Object p0) -> ((String) p0).isEmpty())`）；
- 泛型 Signature 拒（raw 退化=既有已登记域，非本巡查对象）；行为 `[ccc]/[EX, y]/[z]` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。迭代器族（手动循环/remove/set/removeIf）确认覆盖。
