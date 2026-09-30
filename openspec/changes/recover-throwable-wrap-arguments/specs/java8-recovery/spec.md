## ADDED Requirements

### Requirement: java.lang 异常子类的调用实参上转型可呈现

系统 SHALL 在调用实参的被呈现类型为 java.lang 核心 Throwable 子类（固定闭集）且要求类型为其 java.lang 祖先（Throwable/Exception/RuntimeException）时呈现该实参并保留要求类型拼写，使异常包装重抛（`new RuntimeException(msg, e)` 等）与转发调用（用户方法收 `Throwable` 形参）完整恢复、整类可重编。闭集之外的引用上转型 SHALL 继续拒绝并保留既有诊断，直到以 resolution 环境证据建立的一般证明通道启用。

#### Scenario: 包装重抛完整恢复

- **WHEN** catch 体构造带 cause 的包装异常并重抛，以原 class/固定 JADX/Jarde 三方 Java 8 重编运行
- **THEN** catch 体三语句（消息拼接、构造、throw）全部呈现，`java -Xverify:all` 下包装消息、cause 身份与异常传播逐路径与原 class 一致

#### Scenario: 闭集外保持拒绝

- **WHEN** 呈现类型为用户自定义异常类或非祖先 java.lang 类型
- **THEN** 维持 `no safe reference conversion evidence` 拒绝与级联行为，不得隐式放行

#### Scenario: 既有转换回答不变

- **WHEN** 实参走 Object 目标、同名、数组闭集、overload 证明或 List→Iterable 平台回答
- **THEN** 各自呈现与拒绝行为与本变更前逐字一致
