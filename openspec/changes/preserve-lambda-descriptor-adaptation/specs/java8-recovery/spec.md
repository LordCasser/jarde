## ADDED Requirements

### Requirement: Functional invocation preserves dynamic parameter constraints

系统 SHALL 在恢复已验证函数式工厂时，保留擦除接口参数、动态参数限制和实现成员参数之间的区别。对于已支持的 Object 到引用/数组的动态限制及同型或 Object 上溯的实现参数，输出 SHALL 可在其实际呈现的接口目标下编译，调用结果、异常和实现目标 MUST 与原 class 一致。MUST NOT 仅因参数同为引用类型就丢掉检查。

#### Scenario: Erased function still selects the String overload
- **WHEN** 原 class 的函数式接口参数擦除为 Object，动态参数为 String，实现指向 String 重载，完整类输出使用擦除接口类型
- **THEN** 字符串与 null 的调用 SHALL 选择 String 重载，非 String 对象 SHALL 在实现执行前抛 ClassCastException，MUST NOT 改选 Object 重载

#### Scenario: Wider implementation does not erase the dynamic check
- **WHEN** 同一函数式工厂的动态参数为 String，而实现参数为 Object
- **THEN** 正常调用 SHALL 选择 Object 实现，错误类型仍 SHALL 按动态 String 限制失败，MUST NOT 因实现可接收 Object 而放过该输入

#### Scenario: Reference arrays preserve their component restriction
- **WHEN** 动态参数为 String[] 而接口参数擦除为 Object
- **THEN** String[] 和 null SHALL 保持原行为，Object[] 与其它类型 SHALL 保持原 ClassCastException；MUST NOT 把数组检查改成元素检查或扩大为任意数组

#### Scenario: Unbound receiver and constructor use the stated target
- **WHEN** 无捕获实例方法或构造器的接口参数需要本项支持的动态检查
- **THEN** 完整恢复类 SHALL 编译并调用原实现，接收者与参数检查 SHALL 按原顺序发生，创建/方法调用的次数及异常优先级 SHALL 不变

### Requirement: Functional adaptation stays within proven conversion and evaluation boundaries

系统 MUST 明确区分函数对象创建时的捕获行为与函数调用时的参数检查。缺少受支持的参数/返回转换或创建阶段证明时 SHALL 明确说明拒绝原因并保留该站点与捕获生产者来源，MUST NOT 默默加入或删除可失败检查。

#### Scenario: Bound receiver failure is not postponed
- **WHEN** bound 方法引用在创建时对 null 接收者失败，而候选 lambda 写法只会在调用时失败
- **THEN** 系统 SHALL 保留已证明的创建阶段检查，或明确拒绝该转换；MUST NOT 发布推迟异常的正常 Java

#### Scenario: Unsupported adaptation remains explicit
- **WHEN** implementation 参数适配需要尚未证明的引用类型关系，或 implementation→擦除 SAM 的真实转换需要本项未覆盖的 primitive 适配、boxing/unboxing、返回窄化或附加 bootstrap 协议
- **THEN** 系统 SHALL 明确拒绝不受支持部分，MUST NOT 以同为引用、同为槽形状或输出能编译代替等价证明

#### Scenario: Identity adaptation retains successful recovery
- **WHEN** 已有合法工厂的 SAM、动态和实现参数无需转换，返回类型同型或仅确定上溯到 Object
- **THEN** 恢复 SHALL 保持可读 Java，MUST NOT 为修复 String 输入案例而把全部 lambda/method reference 改成引用

#### Scenario: Generic return does not invent a function-body check
- **WHEN** instantiated SAM 返回为 String，但原函数的 implementation 与擦除 SAM descriptor 均返回 Object；raw Supplier 调用可返回 Integer，而 typed String 调用仅在调用者自身 checkcast 时失败
- **THEN** 恢复后的 raw 与 typed 调用 SHALL 分别保持这些行为，MUST NOT 把 instantiated 返回类型误作 implementation→擦除 SAM 的新转换或在函数体额外检查该值

### Requirement: Functional adaptation retains provenance and bounded execution

新增适配工作 SHALL 遵守既有工作、IR、来源预算与取消契约；已发布正文与其来源 SHALL 对应真实站点/捕获输入。证据选择 MUST NOT 改变恢复正文。

#### Scenario: Same body under evidence selection
- **WHEN** 同一输入分别请求 essential 和完整证据
- **THEN** 输出正文 SHALL 相同；完整证据 SHALL 保留真实 invokedynamic、bootstrap/CP 与捕获来源，MUST NOT 为源码 cast 虚构 bytecode 检查位置

#### Scenario: Planning stops before publishing an incomplete adaptation
- **WHEN** 新适配计划或表达式构造触发工作/IR预算或取消
- **THEN** 系统 SHALL 按既有停止契约返回，MUST NOT 将只完成部分检查的 lambda 标记为正常恢复
