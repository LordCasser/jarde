## ADDED Requirements

### Requirement: 分支抛异常与保存返回值穿落穿 finally

系统 SHALL 对已证明的单catch-all Java8方法恢复try内的分支抛异常/单值return及落穿副作用finally，保持原返回快照、原异常及清理行为。清理只执行一次，清理抛错 MUST 替代待完成的返回或原异常；未知拓扑或不能闭合的值/范围/所有权 MUST 保留可定位拒绝与全部物理来源。

#### Scenario: 正常返回与负分支抛异常
- **WHEN** 固定finThrow形分别执行返回分支与创建并抛出异常的分支，finally正常完成
- **THEN** 原样重编完整生成类 SHALL 返回与原class相同的String或抛出同一逻辑来源异常，type/message与清理顺序/次数一致，原物理throw和保存返回值的身份来源保留

#### Scenario: 清理异常覆盖待完成结果
- **WHEN** 相同方法的finally调用抛出已知RuntimeException、Error或空receiver异常
- **THEN** 输出 SHALL 与原class保持异常选择和执行次数，已知清理异常对象须原样逃逸，不得再次执行清理或恢复被覆盖的返回值/异常

#### Scenario: 证明边界与原子停止
- **WHEN** 真输入的清理副本、异常范围、保存/重抛链或区域所有权不匹配，或证明/发射中实际预算耗尽、取消或期限停止
- **THEN** 系统 MUST 拒绝不闭合形或原样返回停止原因，不交付部分声明/正文/来源，不丢弃可靠的原始证据

#### Scenario: 完整来源与证据模式一致
- **WHEN** 对双真实编译器的完整独立类以default/all及CLI恢复
- **THEN** 声明、正文与物理方法身份 SHALL 一致，所有语义指令来源完整，生成源码必须原样通过双编译器重编及隔离rebuilt运行；原EX其它未证明方法与JADX实际编译失败必须保留，不视为本片或父家族完成
