# finally 重抛与精确 rethrow 巡查（2026-10-05 root）

## 健康面（负结果——重抛族全过）

[fixture/RT.java](fixture/RT.java)（`--release 8`）：
- **catch 内重抛** `throw e`（同异常直抛）恢复；
- **multi-catch 精确 rethrow**（`catch(MyEx | IllegalStateException e){ throw e; }`——javac 精确 rethrow 分析的产物）恢复为 multi-catch 形 + 重抛；
- main 四段 try/catch 链（含分型捕获）全恢复；
- 行为（stub rethrow 后）`c/f/2/nr:neg/p1:1/p2:2` 逐行 IDENTICAL——重抛链语义精确。

## 发现：finally 复制中的异常 slot 跨区（local-scope 域，锚家族四形）

`rethrow`（try 抛出 + catch 打印 + finally 打印 + 尾 return）拒——"local 1 crosses a quoted fallback region"（finally 三出口复制中异常 slot 跨引注区）——与 NA-b（平铺 t/c/f）、EX.nestedFin（嵌 finally）**同因同族**，[preserve-local-scope](../../../changes/preserve-local-scope-across-exception-regions/) 锚家族第四形。

## 处置

不新立；锚已补入 local-scope 片 proposal。
