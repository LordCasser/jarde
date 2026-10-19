## ADDED Requirements

### Requirement: Proved named catch loop return tails preserve protected expression evaluation

系统 SHALL 在证据完整的普通命名 catch 中恢复初始化前缀、完整循环和单块返回表达式，保持表达式求值的原顺序及其异常处理。系统 MUST 保留原物理异常范围，不能因为范围右边界恰为不能抛异常的 return 指令，就丢弃整个返回表达式或把未受保护的效果移入 catch。

#### Scenario: Original Loop5 keeps parsing and early multiplier selection

- **WHEN** 对完整 TestBreakInLoop5 debug/no-debug 原件执行 class/JAR、default/all 反编译
- **THEN** 构造器、test、check SHALL 全部报告 structured；完整 test SHALL 保留数字缓冲追加、首个非数字非点字符处的 m/M、g/G 倍率选择和 break，以及随后转换、乘法、ceil、long 返回和 Exception 返回 -1。完整源码原样重编并隔离运行时，1.2、12am、13m、1G4、空串 SHALL 分别返回 2、12、13312、1048576、-1，与同 profile/runtime 原件逐项一致，check 结果与框架边界单独记录

#### Scenario: A sole terminal return can sit at the exclusive protected boundary

- **WHEN** 返回表达式所有可能抛异常和有副作用的指令都在同一已证明命名保护范围内，唯一范围外指令为恰位于右边界的末尾 return
- **THEN** 输出 SHALL 保持完整 try 中的 return 表达式与独立 catch body；原范围 [0,123)、handler124 和 return123 等物理事实 MUST 保持不变，不得宣称 return123 是受保护指令

#### Scenario: A proved catch binding does not retype the ordinary local sharing its slot

- **WHEN** 已恢复catch在处理器入口把该处理器收到的异常绑定到参数，而同一物理局部槽在正常body中存储double等普通局部值
- **THEN** catch头部绑定 SHALL 保持自己的词法声明，不得作为普通局部变量的跨类别赋值造成类型冲突；绑定来自错误处理器、错误路径或槽、非入口写入或不能证明为该处理器收到的异常时，MUST 保持原拒绝及逃逸检查，不得把无关值错误认作catch参数或用强制转换掩盖证据缺失

#### Scenario: Foreign exception scope and incomplete owners remain refused

- **WHEN** 出现额外入口/出口、未闭合循环、错误或额外异常行、handler 混入正常 body、越界非 return 指令、重复或遗失 owner，或返回尾不满足唯一末尾条件
- **THEN** 系统 MUST 拒绝发布该完整组合，保留全部未恢复事实和原原因；已被接受的旧 goto 路径及其它异常语义 SHALL 保持原契约

#### Scenario: Candidate interruption never publishes partial recovery

- **WHEN** 在前缀、循环、返回尾证明及发布期间遇到取消或预算不足
- **THEN** 系统 SHALL 传播真实停止原因与用量，不提交局部 owner 或泄漏试探状态；成功场景下 default/all 的正文、identity、quality、映射和有效物理引用 MUST 一致
