# 防御性拷贝惯用法巡查（2026-10-05 root，负结果）

## 探针

[fixture/DF.java](fixture/DF.java)（`--release 8`）：ctor `src.clone()` 入 final 字段、getter `field.clone()` 出（不可变对象惯用法）、for-each 原生数组累积、`Arrays.copyOf` 扩容链、`System.arraycopy` 行拷贝（矩阵库惯用法）、main 改拷贝验证隔离性。

## 结果：**健康，无缺口**（成员 quotes=0；5 quotes=main 级联）

- ctor/getter 防御性拷贝完整（`(int[]) arg1.clone()` / `return (int[]) this.data.clone();`——covariant cast 如实）；
- for-each 原生数组（`for (int local5 : local2)` 增强 for 形保留）+ 内部数组快照局部（local2 = this.data）；
- `Arrays.copyOf` 扩容链 + 嵌套 ctor 调用完整；arraycopy 五实参（含两次 `(Object)` 擦除 cast——物理事实）；
- 行为往返 `6/1/6/456` 逐行 IDENTICAL（隔离性精确：改 out[0] 不影响 d.sum()=6、d.data()[0]=1）。

## 处置

负结果归档，不立 spec。防御性拷贝域（clone 双向/copyOf/arraycopy 行拷贝/for-each 数组）确认覆盖。
