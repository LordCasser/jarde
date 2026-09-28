## ADDED Requirements

### Requirement: 透明 catch-all 的空 finally 降为具名 catch

系统 SHALL 仅在 Java 8 方法的 catch-all handler 对所有进入的 Throwable 不产生可观察效果并原样重抛、且其异常行与具名 catch 的保护范围和出口均有完整物理证据时，省去空 `finally` 并输出可重编的普通 `try/catch`。恢复 MUST 保留受保护调用、具名 catch 的吞吐行为、其他异常的身份与传播，以及每个物理 BCI 的来源；证据不足、停止或取消时 MUST 安全拒绝，不能把 handler 消失解释成异常边也消失。

#### Scenario: 正常关闭与受检异常

- **WHEN** 固定 TestEmptyFinally 两行 Java 8 方法的 `close()` 正常完成或抛出 `IOException`
- **THEN** 恢复的完整类 SHALL 在两条路径上各调用一次 `close()`，分别正常返回或吞掉该受检异常；原 class、固定 JADX 和恢复类在 `java -Xverify:all` 下 SHALL 行为一致，输出只有具名 `catch(IOException)` 而无空 `finally`

#### Scenario: 其他异常传播

- **WHEN** 同一 `close()` 抛出非 `IOException` 的运行时异常
- **THEN** 恢复类 MUST 传播同一个异常对象，不增加清理调用、不更换异常、不吞掉异常

#### Scenario: 非透明或不闭合的异常形态

- **WHEN** catch-all handler 新增副作用、改写重抛对象，异常表范围/优先级改变，或者存在额外进入 handler 的正常边、异常边或未归属块
- **THEN** 系统 MUST 拒绝省略该 handler 的源码投影，保留物理指令与异常边的可追溯拒绝信息，不输出伪等价的 `try/catch`

#### Scenario: 来源与停止

- **WHEN** 固定目标证明成功、证明预算耗尽或请求取消
- **THEN** 成功产物 SHALL 覆盖目标全部 BCI，包括被证明透明的 handler；预算耗尽或取消 MUST 不交付半个 `try/catch` 或遗漏 `close()` 的源码
