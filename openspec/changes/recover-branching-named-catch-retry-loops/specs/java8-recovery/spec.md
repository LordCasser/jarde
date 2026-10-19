## ADDED Requirements

### Requirement: Recover proved conditional rethrow and retry catches

Java 恢复 SHALL 支持一个普通物理保护块的具名 catch 重试循环：保护区正常完成后退出，handler 的一次条件分支分别到终止 throw 与效果语句后回到精确保护区入口。恢复 SHALL 保留条件求值、被抛值、效果顺序、异常作用范围和精确循环转移；每个真实块 SHALL 仅由一个结构位置拥有，物理转移的来源 SHALL 保留。该能力 MUST NOT 将结构证据升级为 JVM verification 或普遍语义等价声明。

#### Scenario: Complete original with conditional rethrow
- **WHEN** 对完整 `TestBreakInLoop2` 原件的 debug/no-debug class 请求 Java 恢复，且单保护块、唯一具名 handler、条件重抛分支及回跳分支均有闭合证据
- **THEN** 目标方法 SHALL 形成结构化的 try/catch 重试循环，正常完成退出、重抛终结、clear 与 sleep 仅在重试路径按原序出现；生成的完整类 SHALL 能重编，已声明可达输入的行为 SHALL 与原 class 对照一致

#### Scenario: Retry and terminal branch remain separate
- **WHEN** 独立可执行控制例实际触发一次失败后重试成功，或触发条件重抛，且两种条件极性分别有完整输入证据
- **THEN** 重编结果 SHALL 保留事件顺序、异常类型和值依赖、正常退出及输入变化，throw 后 SHALL 不发生重试效果，保护区外效果抛出的异常 SHALL 不被该 catch 接管

#### Scenario: Incomplete branch ownership is refused
- **WHEN** handler 或分支叶有未证明的外部入口、共享物理 handler、额外转移或异常边、错误保护范围、错误回跳目标或未闭合的终结
- **THEN** 该新增恢复能力 SHALL 拒绝该候选，保留既有有界降级及所有权检查，MUST NOT 靠去重、吞掉真实块或改变 CFG 把候选发布为完整结构

#### Scenario: Proof work stops without publishing a partial loop
- **WHEN** 候选证明或结构组装期间达到预算限制或取消
- **THEN** 恢复 SHALL 保留原停止原因、诊断、真实用量和已允许的可靠前缀，MUST NOT 发布未证明的循环、重复 owner 或在取消后重试收费
