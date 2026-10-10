## ADDED Requirements

### Requirement: A proved return-arm loop retains its hidden latch source

系统 SHALL 为已经恢复且循环体末尾由一个条件分支分别返回和回到准确循环头的已证明唯一物理回边保留完整循环语句的派生来源。来源 MUST 绑定原 class 字节、物理方法 name/descriptor 和准确 BCI，保持已有正文与来源。

#### Scenario: The terminal conditional returns or iterates

- **WHEN** lastIndexOf([IIII)I 的循环末尾条件一侧返回，另一侧减小索引并由物理 goto@25→5 回到该循环头，返回边界、唯一自然回边及完整出边均已证明
- **THEN** @25 的 derived 来源覆盖完整 while 语句，既有 condition、If、更新和返回来源保持，全方法物理 BCI 可准确查询

#### Scenario: A transfer belongs to another destination or loop

- **WHEN** 候选不在该末尾分支持有范围、指向其他目标或内层循环、不是末尾无条件跳转、含额外或异常出边、自然环非唯一或不可归约，或者另一侧不能证明为返回
- **THEN** 系统不得增加该循环的回边来源，既有可靠来源与拒绝边界保持

#### Scenario: Complete source remains executable and stable

- **WHEN** 双 JDK 原 class 以 default 和 all 证据配置生成完整 CF07 类，并使用新候选工具重放已接受循环控制
- **THEN** 配置间正文与映射一致，所有旧正文和来源保持；完整生成源码原样重编运行的 exit/stdout/stderr 与同 JDK 原 class 完全一致

#### Scenario: Source coverage does not imply a different loop spelling

- **WHEN** 唯一隐藏回边来源已补齐，生成循环仍以 while 表达原 for
- **THEN** 报告 SHALL 保持准确当前呈现，不因物理 BCI 全覆盖宣称原 for 写法或整个语法单元已追平

#### Scenario: A stopped proof publishes no partial output

- **WHEN** 新回边来源证明遇到预算不足或取消
- **THEN** 系统传播既有 Stop，MUST 不发布部分正文或部分 source map
