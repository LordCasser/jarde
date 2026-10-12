## ADDED Requirements

### Requirement: A protected prefix and loop retain one exception scope

普通单range named catch保护的前置语句和已证明循环，当完整body都位于该保护范围内、内部continuation及唯一正常出口均已证明时，恢复 MUST 把它们完整置于同一try body。前置iterator构造、循环条件、元素读取/检查转换和循环body的异常覆盖 MUST 与物理异常表一致；MUST NOT 只恢复前缀、把循环移到try外或以未owned块代替已证明的body。

范围末尾的跳转只有在它是无效果、无异常保护/handler入口、无竞争进入、仅通向try join的已证明纯transfer时，恢复 MUST 允许用try后的词法continuation呈现并保留来源。未证明出口、重复所有权、跨范围效果、fallback或未支持组合 MUST 沿既有保守路径处理，MUST NOT 通过正常流投影隐藏异常/调用边或猜测闭合。

新增续接的header、body和前缀的每条可触发异常边 MUST 精确属于当前Catches的range/types/handler及实际ordinal；MUST NOT 仅凭own_try起点节点或另一named row覆盖该块放行。此归属仅在已证明的ordinary候选传播，MUST NOT 全局扩大loop frame或替代finally等已有证书。nested Try/Guard、额外活动row及未支持scope组合 MUST 保持拒绝。

#### Scenario: Iterator initialization and loop stay protected

- **WHEN** 一个try中先初始化iterator、再执行已识别循环，整个前缀和循环受同一个RuntimeException handler保护
- **THEN** 恢复 MUST 把初始化和循环都写入try中；完整生成类运行时iterator构造、hasNext、next及body抛出的RuntimeException按原handler处理，Error仍按原行为逃逸

#### Scenario: The certified range end has one lexical continuation

- **WHEN** 循环唯一正常出口是保护范围独占末端的单一纯跳转，所有物理进入都来自已owned body且唯一后继为try join
- **THEN** 恢复 MUST 把transfer作为try后的continuation，并保留其物理来源；循环body、catch和末尾块各自只owned一次

#### Scenario: A physical named row cannot borrow another try scope

- **WHEN** loop header/body的可触发边指向不属于当前sites的named row、同handler不同range、catch-all，或普通loop没有该候选的scope事实
- **THEN** 恢复 MUST 保留原拒绝；MUST NOT 因真实存在一个handler就声称当前Try已覆盖该异常

#### Scenario: A competing bridge cannot borrow the certificate

- **WHEN** 范围末尾块有外部或异常进入、handler角色、效果指令、多后继、非join后继、额外保护范围或loop/control target角色
- **THEN** 恢复 MUST 拒绝新的闭合决定并保留可靠refusal/bytecode/source-map，MUST NOT 把这个块的效果移入try或删去竞争边

#### Scenario: Incomplete child recovery cannot become a complete try

- **WHEN** 内部continuation返回fallback、重复已owned块、跨出保护范围或无法证明完整循环和正常出口
- **THEN** 恢复 MUST 保持既有保守分类，MUST NOT 用一个Sequence包装把不完整恢复标成完整

#### Scenario: Stops and request selection preserve atomic ownership

- **WHEN** 新的续接/所有权/出口验证发生预算耗尽或取消，或同一输入由default/all请求恢复
- **THEN** 中止 MUST 发布既有Stopped，无部分try、来源或owned状态；完整请求 MUST 得到相同正文和真实来源，邻近已有switch/fragmented/retry/finally行为不回退
