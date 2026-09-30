## ADDED Requirements

### Requirement: 具名行前的完成 store 交回普通 catch 呈现

系统 SHALL 在具名类型异常行（`catch_type_index` 非空）的保护范围起点位于语句边界、且紧邻前置指令是完成一条语句的 `Store` 时，将该行交由普通 `catch` 呈现而不做资源头检查；该 store 呈现为它自己的语句，受保护正文与 handler 体完整恢复。catch-all 行、行起点不在语句边界、或范围劈开/吞掉语句的形状 SHALL 保持既有资源检查与拒绝语义不变。

#### Scenario: 构造或调用 store 后接具名 catch

- **WHEN** 方法形如 `Acceptor a = new Acceptor(); try { a.foo(); } catch (NamedE e) { … }` 或 `r = open(); try { … } catch (NamedE e) { … }` 且以 Java 8 重编三方对照
- **THEN** 全部语句与唯一 `try/catch` 恢复，跨该 region 的局部以声明-初始化器呈现，正常与异常路径 `java -Xverify:all` 运行与原 class 一致

#### Scenario: catch-all 与非边界保持拒绝

- **WHEN** 行为 catch-all 且前置完成 store，或行起点落在构造表达式/旧值更新中间、吞掉初始化
- **THEN** 维持既有降级拒绝码与语义，不得改判为用户 catch

#### Scenario: 真 TWR 的具名包装行不受影响

- **WHEN** 输入是 `try (R r = …) { … } catch (NamedE e) { … }` 的 TWR 降低（具名行覆盖资源初始化）
- **THEN** 该行不被此判别放行，TWR 家族既有恢复与拒绝行为不变

#### Scenario: 停止与来源

- **WHEN** 证明、区域构造或输出被取消或耗尽预算
- **THEN** 原子停止；成功时 lead store、正文、handler 与跨 region 局部读写的全部 BCI SHALL 在 source map 中可查
