# Map 缓存惯用法巡查（2026-10-05 root，负结果——memoization 家族全过）

## 探针

[fixture/MC.java](fixture/MMC.java)（`--release 8`）：**`get` + null 守卫 + `put`**（memoization 惯用法——最常见的手写缓存）、`containsKey` 消歧（null 值合法形）、`computeIfAbsent(k, lambda)`、`getOrDefault`。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- memoization 完整还原：`(Integer) cache.get((Object) k)` + null 守卫 + `cache.put(...)` + `intValue()` 拆箱返回——cast 链、装箱/拆箱、静态字段限定全部精确；
- containsKey/get 消歧、computeIfAbsent 谓词 lambda 内联、getOrDefault 全恢复；
- 行为 `3/3/hit/absent/4/dflt` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。Map 缓存族（get-guard-put/containsKey/computeIfAbsent/getOrDefault）确认覆盖。
