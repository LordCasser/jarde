# 布尔复合/SafeVarargs/char switch 巡查（2026-10-05 root，负结果）

## 探针

[fixture/BV.java](fixture/BV.java)（`--release 8`）：布尔复合赋值三形（`b &= x`/`b |= y`/`b ^= true`——javac 发 iand/ior/ixor，**非短路**语义）、`@SafeVarargs` 注解 + 泛型变参方法体、char switch 标签（含合并 case）。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- 布尔 `&=`/`|=` 呈现为 `local = local & x`（**非短路 `&`**——iand 物理事实如实，与源 `&=` 语义一致）；`^= true` → `local = !local`（等价规范化）；
- `@java.lang.SafeVarargs` 注解逐字保留、varargs 泛型方法体（for-each + null 检查计数）完整恢复（泛型 Signature 拒=raw 退化既有域）；
- char switch 标签精确（`case 'b': case 'c':` 合并、default 兼 return）；
- 行为 `false/true/true/false/2/ABC?` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。布尔复合（非短路保真）/SafeVarargs/char switch 确认覆盖。
