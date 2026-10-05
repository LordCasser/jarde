# 异常翻译/cause 链巡查（2026-10-05 root，负结果——真实代码最常见异常模式全过）

## 探针

[fixture/ET.java](fixture/ET.java)（`--release 8`）：**catch 包装重抛**（`throw new IllegalStateException(msg, e)` 带因构造——异常翻译模式）、**因链检查**（`e.getCause().getClass().getSimpleName()` 两级链式）、跨方法异常传播、三元内方法调用。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- **带因构造**：`new IllegalStateException("bad input: " + k, (Throwable) local1)`——cause 实参的 `(Throwable)` 上转型 cast 如实（物理 invoke 次序）；
- **因链检查**：`local1.getCause().getClass().getSimpleName() + "/" + local1.getMessage()` 多级链恢复；
- 行为 `NumberFormatException/bad input: x`、`7`、`0` 逐行 IDENTICAL——**cause 链穿透精确**（错误丢因构造会改变 getCause 结果）。

## 处置

负结果归档，不立 spec。异常翻译族（包装重抛/带因/因链检查/传播）确认覆盖。
