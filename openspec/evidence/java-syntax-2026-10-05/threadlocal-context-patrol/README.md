# ThreadLocal 上下文传递族巡查（2026-10-05 root）——域主体健康；withCtx = local-scope 第 11 锚

## 探针

[fixture/TL.java](fixture/TL.java)（`--release 8`）：匿名子类 `initialValue` 覆写（`CTX`）、`ThreadLocal.withInitial(() -> 0)` lambda 工厂、`InheritableThreadLocal`、**save→try→finally-restore 上下文管理器形**（withCtx）、get+intValue+set+valueOf 装箱链（bump）、匿名 Thread 捕获继承值（main）。

## 结果

- **字段/工厂/装箱链健康**：`CTX = new TL$1()`、`SEQ = ThreadLocal.withInitial((Supplier)(() -> 0))`（lambda 工厂 cast 呈现）、`PARENT = new InheritableThreadLocal()`；bump 的 get→intValue→+1→set(valueOf) 装拆箱链完整；
- **withCtx = 整方法 crosses 拒（SAFE）**："local 2 crosses a quoted fallback region"——`old` 局部的 def-use（声明→try 前读→finally 写回→return 读）穿 try-finally 恢复区；空 body + 非 void 缺 return → 编译失败 = SAFE；jadx 完整解（try/finally 形）。**上下文管理器惯用法**（save/restore across finally）——local-scope 异常区族的经典新形状；
- main 3 quotes = 匿名 Thread/Runnable 实参位（family 6 域）+ 捕获写回级联。

## 处置

**local-scope 片第 11 锚**（save-old→finally-restore→return：上下文管理器形——连接池/事务/租户上下文真实骨架）；ThreadLocal 域（withInitial/继承/装箱链）不立项。
