## ADDED Requirements

### Requirement: 标注循环尾段完整覆盖与源码式标签

系统 SHALL 在标注循环（含 `continue label`）的尾随语句段接受"非 void 调用后紧随丢弃其结果"的语句，使尾段全部物理 BCI 进入覆盖、输出无 `@bytecode` 引用残留；标签 SHALL 以源码式简名拼写（无内部合成痕迹）。17a 的负例与 TWR 路径 SHALL 零变化；行为 SHALL 与原 class 逐字一致。

#### Scenario: 尾段残留消除

- **WHEN** `outer: for (…) { inner; …; chainAppend(); tail(); }` 含 `continue outer`，三方 Java 8 重编运行
- **THEN** 输出零引用残留，标签为源码式拼写，`java -Xverify:all` 逐路径与原 class 一致

#### Scenario: 对照与既有路径不变

- **WHEN** 输入为无标注循环、直线体调用语句或 17a 的 TWR 形态与负例
- **THEN** 输出（除标签拼写本片有意变化外）与本变更前逐字一致
