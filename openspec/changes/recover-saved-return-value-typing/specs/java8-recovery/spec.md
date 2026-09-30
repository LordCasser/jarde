## ADDED Requirements

### Requirement: TWR saved-return 声明按保存值类型拼写

系统 SHALL 在 try-with-resources 的保存返回局部声明类型可由保存值的生产者（字面量、调用结果、构造点、数组）经既有值-类型通道细化时，以细化类型拼写该声明，使 try 内 return 形态的完整类可重编。null 保存值 SHALL 保持既有拼写；细化不可得时 SHALL 回退既有拼写而不拒绝。

#### Scenario: 非 null 保存值可重编

- **WHEN** `try (T r = …) { …; return "in"; }` 的完整类以三方 Java 8 重编运行
- **THEN** 声明拼为保存值类型（如 `String local1 = "in";`），`javac --release 8` 通过，正常与异常路径 `java -Xverify:all` 与原 class 一致

#### Scenario: null 与既有输出不变

- **WHEN** 保存值为 null 或细化不可得（跨块/phi）
- **THEN** 声明拼写与本变更前逐字一致，既有钉死期望全绿

#### Scenario: saved-return 家族零回退

- **WHEN** 输入覆盖 TWR 家族与 finally 各证书的 saved-return 形态
- **THEN** 除本切片目标外输出逐字不变
