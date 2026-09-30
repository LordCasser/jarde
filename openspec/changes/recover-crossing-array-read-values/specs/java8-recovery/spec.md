## ADDED Requirements

### Requirement: 跨保护区域局部的数组元素写值可呈现

系统 SHALL 在跨保护区域局部的内联写值树含同块数组元素读（数组操作数与下标树均在既有白名单内、值单用途、求值区间封闭）时，将该写值判为可呈现，使循环累计数组元素且体内含 try/catch 的方法完整恢复。共享元素值、跨块读或区间插副作用指令的写值 SHALL 保持既有拒绝；无保护区域的对照 SHALL 逐字不变。

#### Scenario: 循环累计 + try/catch 恢复

- **WHEN** `for (int i = 0; i < a.length; i += 2) { sum += a[i]; try { sum += risky(a[i]); } catch (E e) { sum -= 1; } }` 三方 Java 8 重编运行
- **THEN** 方法完整恢复，正常与注入异常路径 `java -Xverify:all` 与原 class 一致

#### Scenario: 对照与负例不变

- **WHEN** 输入为无保护区域的同表达式对照，或共享/跨块/插副作用的写值形态
- **THEN** 前者与本变更前逐字一致；后者保持既有拒绝与诊断
