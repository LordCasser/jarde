# synchronized 返回时序巡查（2026-10-06 root）——**critical 第 20 锚 / 第 7 族：monitor 释放时序漂移**

## 探针

- [fixture/SR.java](fixture/SR.java)：单层 return-in-monitor（`retInside`）、跨 monitor 局部写（`localAcross`）、void 体（对照）、嵌套双锁 return（`nestedLock`）；
- [fixture/NL.java](fixture/NL.java)：**判别探针**——`Box.toString()` 返回 `Thread.holdsLock(NL.class) ? "Y" : "N"`，`probe` 在内层 `synchronized(NL.class){ return "n" + o; }` 求值。

## 发现

- **单层形健康**：`retInside`/`localAcross` 完整恢复（return/局部均在锁区内）；`voidBody`（void+throw）整方法 crosses 拒——**又一个 void 空体活例**（同 LK.put，归 soundness 片 void 修复，不另立）；
- **嵌套内层 return = 可编译错码**：`nestedLock`/`probe` 渲染为内层 `synchronized(NL.class){ }` **空块** + `return "n" + arg0;` **外移到内层锁释放之后**；
  - 字节码（javap 实证）：求值 BCI 11–27 在内层 `monitorexit`（BCI 31）**之前**；
  - 渲染剥离编译 exit 0、`-Xverify:all` 运行 **`nN`** vs 原 **`nY`**——`Thread.holdsLock` 判别直接证明求值被移出了持锁窗口（第一不变量违反，compilable-wrong）；
  - jadx **有解**：`String str; synchronized{ synchronized{ str = "n"+obj; } } return str;`——临时局部赋值留在内层锁内、return 外移，时序保真。

## 归因（初步）

build.rs 的 synchronized return 形（~14525 注释/`returns` 分支）要求 return 语句在**该层** braces 内；嵌套 pair 下内层形状渲染为空、return 归属外层 body——外移越过内层 `monitorexit`。**不是诊断族 1–6 的语句吞形**：本例全恢复、无引注，是结构性呈现错序（第 7 族）。

## 处置

**窄片立项 `preserve-monitor-exit-evaluation-order`**（呈现不变量：return 消费的表达式求值必须发生在配对 `monitorexit` 之前——return 留在锁内或 temp 赋值锁内+return 锁外；jadx 先例）。
