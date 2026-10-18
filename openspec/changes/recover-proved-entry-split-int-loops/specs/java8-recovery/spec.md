## MODIFIED Requirements

### Requirement: Semantics-preserving fallback

恢复 MUST 保留异常优先级、资源关闭/抑制异常、monitor、类初始化和副作用顺序；未获本规范明确的完整源码变换证明的不可约 CFG、非法 Java 名称或不满足模式时 MUST 降级，不生成伪等价空代码。明确受证的条件初始int前缀循环仅按其独立要求恢复，不将该例外推广至其它不可约图。

#### Scenario: TWR and exceptional close

- **WHEN** 方法包含 try-with-resources 的正常和异常 close 路径
- **THEN** 输出保留 close 顺序、primary/suppressed exception 关系；无法证明时返回 representation=Mixed/Bytecode、quality=Conservative/Fallback 和诊断

#### Scenario: Unstructured control flow

- **WHEN** 方法的区域恢复遇到没有明确完整变换证明的不可约 CFG 或交叉异常区域
- **THEN** 结果保留可靠低级结构和 origin，不能输出空 body 或成功 Structured 标志；若扫描完整，coverage 仍可为 CompleteWithinSchema，只有扫描失败才为 Partial（验收 A13）

## ADDED Requirements

### Requirement: A proved entry-split integer prefix loop preserves initial and recurring updates

系统 SHALL 恢复有完整物理拓扑、SSA及精确局部类型证明的条件初始int前缀循环：入口分支选择先更新值或直接检查计数；计数非零时先更新计数、再执行同一值更新并重新检查；退出返回实际值。首片只覆盖三个不同局部绑定、无异常/调用/内存效果、单前缀更新/单计数更新/单条件/单返回闭合的形状。输出 MUST 保留初始路径、每轮次序、int溢出、退出值和物理来源，缺证保持原整方法拒绝。

#### Scenario: Skipping the first prefix does not skip recurring prefixes

- **WHEN** 值初始17、前缀加3、计数每轮减1，入口选择跳过或不跳过首次前缀，计数为0、1、2
- **THEN** 完整重建类 SHALL 分别得到跳过时17/20/23、不跳过时20/23/26，与原class相同；输出包含对应条件初始更新、循环及最终return，不能丢掉首次或重复某轮更新

#### Scenario: Integer wrapping and structural variants remain faithful

- **WHEN** 已证明相同图形的值靠近int上下界，或只改变有符号更新常量、合法局部槽/参数位置及入口分支极性
- **THEN** 恢复 SHALL 依据实际操作、槽绑定和分支sense呈现，与原class的32bit环绕结果及原始选择一致，不匹配固定类名、descriptor、BCI、槽号或+3/-1常量

#### Scenario: One physical prefix retains both displayed origins

- **WHEN** 受证前缀在条件初始位置和循环尾各展示一次
- **THEN** 两个源码span MUST 指向同一物理方法与真实前缀BCI，入口/条件/尾更新/goto/return的全部物理BCI可追；不得伪造第二物理节点或丢掉重复展示中的来源

#### Scenario: Additional edges effects or ambiguous locals preserve refusal

- **WHEN** 图存在额外入口/出口/live块、异常或调用/monitor/内存效果、别名更新、非Int或Unknown更新类型、SSA/绑定缺证、栈逃逸、jsr上下文或未accounted指令
- **THEN** 本片 SHALL 不发布该结构化候选，保留完整拒绝、原始物理事实与来源，不放开一般不可约图或输出只有一半效果的源码

#### Scenario: Budget and cancellation do not publish half a duplicated prefix

- **WHEN** 候选拓扑/SSA证明、复制前缀、任一AST输出或来源物化遭遇真实预算拒付或取消
- **THEN** 系统 MUST 原样传播停止原因和实际用量，未完成正文/映射不发布，不能回退成成功空代码、只留初始更新或仅留循环尾更新；重复展示的真实输出分别计费

#### Scenario: Complete comparison does not imply whole-family coverage

- **WHEN** 交付本片的公共库/CLI与原始/JADX/Jarde完整类对照
- **THEN** 成功源码 MUST 原样经两编译器与隔离JVM对照，保留全部失败与负向边界；原Smali/DEX、其它不可约图及整个CF08/71单元账本不得由该构造JVM正例自动标为完成
