# 单例双惯用法巡查（2026-10-05 root）——DCL 可恢复性缺口（SAFE）；lazy holder 恢复

## 发现

- **惯用法 1（lazy holder 按需类初始化）恢复**：`holder()` → `return SG$Holder.INSTANCE;`，伴生 Holder 静态嵌套类 + `static final` 字段初始化完整；`volatile` 修饰符字段声明逐字；
- **惯用法 2（DCL 双重检查锁）整方法拒**：`dcl()` → "local 0 crosses a quoted fallback region"，body 空 + **非 void 返回类型缺 return → 编译失败 = SAFE**；
- **机制**：静态字段 `dcl` 的值 def-use 切片（getstatic 读 + dup 空检 + sync 后重读 + return）**跨 monitor（synchronized 块）引注区**——crosses 呈现族新形状：静态字段惰性初始化穿 monitor 区（#75 guarded wait 恢复的是**局部条件**循环，本形是**静态字段双读**穿区）；
- jadx 完整解出教科书 DCL 形（if-null → synchronized(X.class) → re-check → 赋值 → return）。

## 处置

**local-scope 片第 10 锚**（crosses 族：monitor 区 × 静态字段 def-use——真实并发代码最高频单例形）。
