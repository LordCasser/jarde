# try/catch/finally slot 复用巡查（2026-10-05 root，已立项域的锚数据点）

## 判别实验（[fixture/NA.java](fixture/NA.java)，`--release 8`）

| 方法 | 形 | 结果 |
| --- | --- | --- |
| `a` | catch 内嵌 try（无 finally） | **恢复**——嵌套本身不是缺口 |
| `b` | 平铺 `try{return 100/n}catch(Exception){return 1}finally{println}` | **拒**："local 1 crosses a quoted fallback region" |
| `c` | 嵌套 + finally | 拒（同因） |

**机制**（[results/javap-NA.txt](results/javap-NA.txt)）：javac 把 finally 复制进三个出口（println ×3），并把 **slot 1 复用**为 try 结果 int（`istore_1@4`）与 catch 异常对象（`astore_1@15`）——恢复器无法把 slot 1 的赋值/消费呈现为一个词法有界局部，整方法拒绝（响亮、非静默）。

## 归属

该形**已立项**：`preserve-local-scope-across-exception-regions`（1.2/1.3/2.1-2.5 待做）的 Why 正是"跨 try/catch 局部作用域"。本巡查价值：(1) 提供其待做任务的**最小常见形锚**（`b` 是 Java 最普通的 try/catch/finally 返回值形，比该 change 既有 fixture 更基础）；(2) 证实**嵌套 try-in-catch 本身已恢复**（`a`），缩小该 change 的实际缺口面到 slot 复用/词法作用域。KX 探针的 `nestedTry`（组合形）拒绝同因。

## 处置

不新立 change；`NA-b` 建议并入上述 change 的 2.3 验收锚（其 fixture SHA 见 fixture/）。CF-16/CF-18 行不受影响（它们验收的是既有 fixture 形）。
