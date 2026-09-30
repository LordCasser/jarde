## ADDED Requirements

### Requirement: TWR 的包围具名 catch 恢复

系统 SHALL 在具名类型异常行完整覆盖 try-with-resources 降低的 claim 跨度（含资源初始化到清理链尾）、且其 handler 块在 claim 之外时，将资源声明与 `catch (E e) { handler 体 }` 一起呈现：try 头携带资源语义，catch 子句呈现 handler 体，全部行与块（含 handler）进入覆盖账本。catch-all 包围行、部分覆盖行、交叠 handler 或不可呈现 handler 体 SHALL 保持既有拒绝语义。

#### Scenario: 包围 catch 完整恢复

- **WHEN** 固定形态类（`try (T r = …) { … } catch (E e) { return "caught"; }`，含正文调用语句形态）三方 Java 8 重编运行
- **THEN** `try (…)` 与 catch 子句都完整呈现，正常路径续行、注入具名异常走 catch、清理异常传播、suppression 语义保留，`java -Xverify:all` 逐路径与原 class 一致

#### Scenario: 非包围形状保持拒绝

- **WHEN** 包围行为 catch-all、只覆盖部分跨度、handler 与 claim 交叠，或 handler 体含分支/循环与双 catch
- **THEN** 维持 `jre_guard_unexplained_row` 或正文拒绝的既有诊断，不得呈现半个结构

#### Scenario: 无 catch 的 TWR 不变

- **WHEN** 输入为既有 TWR 家族（单/多资源、可空资源）
- **THEN** 输出与本变更前逐字一致
