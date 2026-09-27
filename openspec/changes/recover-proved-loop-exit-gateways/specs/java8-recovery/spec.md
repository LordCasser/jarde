## ADDED Requirements

### Requirement: 同目标的循环退出网关不得丢失可达出口

当循环条件的正常失败路径和循环体内的另一条条件分支分别经过纯转接到达同一个循环后继，系统 SHALL 保留两条路径在各自原条件下离开循环的行为，MUST 保留各物理出口的来源，MUST NOT 发布遗漏任一可达出口的完整 Java 方法。

#### Scenario: 纯整数循环有两个 break 出口

- **WHEN** Java 8 `EndlessInts.find(int)` 的 `i >= limit` 与 `i == 3` 分别从同一个 `while (true)` 离开，并在循环后返回 `i`
- **THEN** Jarde SHALL 输出无 `@bytecode` 的完整可重编类；原 class、固定 JADX、Jarde 的完整源码在 `java -Xverify:all` 下对 `limit=0,1,2,3,4,8` MUST 逐行同为 `0,1,2,3,3,3`，第二条出口与正常出口的指令来源 MUST 均可追溯

#### Scenario: 网关归属无法证明

- **WHEN** 任一候选转接块有额外正常入口、异常或子例程边、除纯直接转移外的指令，或两个出口实际指向不同后继
- **THEN** 系统 MUST 保持覆盖该路径的物理引用和诊断，MUST NOT 将未证明的转接隐藏为已恢复的循环出口，也 MUST NOT 发布会改变终止条件的完整方法

#### Scenario: 请求在证明期间停止

- **WHEN** 扫描转接、核对入边或恢复区域期间达到预算或收到取消
- **THEN** 系统 MUST 原子停止或拒绝该候选，保留已知物理来源，MUST NOT 发布只包含其中一个出口的结构化循环

#### Scenario: 已证明的单出口循环保持行为

- **WHEN** 输入属于原有已恢复的单 `break`、计数 `for`、或 CF-07 的循环内唯一返回叶形态
- **THEN** 系统 MUST 保持它们原有的出口目标、更新次数和完整来源，不得因本次网关证明产生重复或遗漏转移
