## ADDED Requirements

### Requirement: 体内配对的第二对 monitor 按嵌套 synchronized 呈现

系统 SHALL 在外层 monitor 体内存在恰好一对配对的内层 monitorenter/monitorexit（对象值 SSA 同一、均在体内路径）时，将内层对按嵌套 `synchronized` 块呈现，外层配对证明不变。单层 monitor、monitor 与 try 复合等既有形态 SHALL 逐字不变；不配对、跨出外层或三层以上 SHALL 保持既有拒绝。

#### Scenario: 嵌套双锁恢复

- **WHEN** `synchronized (A.class) { int s = 0; synchronized (log) { for (…) … } return s; }` 三方 Java 8 重编运行
- **THEN** 嵌套块完整呈现，`java -Xverify:all` 逐路径与原 class 一致

#### Scenario: 单层与负例不变

- **WHEN** 输入为单层 synchronized 或内层不配对/跨出
- **THEN** 输出与本变更前逐字一致
