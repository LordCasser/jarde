## ADDED Requirements

### Requirement: Exact List parameters retain their types in certified ordinary try bodies

系统 SHALL 在普通单range named catch 的前缀与foreach结构、整个void方法及全部参数用途均已证明时，恢复方法Signature中的精确 `List<E>` 参数，并生成在这些类型下可编译的完整Java正文。支持范围 SHALL 包含已认证foreach内的分支、null写入、`remove(Object)`前缀与嵌套foreach，统一适用于已有有限List赋值证明支持的精确引用元素类型，至少验证 `String` 和 `Integer`。MUST NOT 按fixture、成员名称、变量名称或debug元数据判断准入，也不得把已读取参数称为未读参数。

#### Scenario: String and Integer branches inside an ordinary catch

- **WHEN** 方法分别以精确 `List<String>` 或 `List<Integer>` 参数执行已认证普通try前缀+foreach，循环含已证明分支、元素写入与null写入
- **THEN** 两种方法均SHALL恢复参数泛型并保持完整正文；成功、空输入、null元素和两个分支的值、集合、异常及副作用与原class相同，缺少debug信息时准入结果不变

#### Scenario: Object removal and nested foreach keep their types

- **WHEN** 同一已认证范围在foreach前调用 `remove(Object)`，或foreach正文包含已支持嵌套foreach和条件，且所有List参数用途闭合
- **THEN** 系统SHALL恢复每个已证明精确参数，保留实际调用重载、嵌套结构和求值顺序；不得为通过验收删除前缀、内层循环或完整类的其他方法

### Requirement: Try generic projection preserves complete physical and source use closure

普通try泛型准入 SHALL 同时绑定真实异常表行的ordinal、catch type、handler和protected范围，以及同次恢复的body、handler与continuation唯一归属。MUST 核验整个物理方法的Code、操作数、SSA、effects和操作库存的一一对应，包含handler-owned指令；每个参数读写与每个受保护effect都须有闭合解释。仅有非空或Structured正文、相同handler地址、相同数量、文本try或Signature与descriptor擦除一致，MUST NOT 代替这些证明。

#### Scenario: A handler reads or overwrites a List parameter

- **WHEN** catch内存在List参数读取、写入或别名赋值
- **THEN** 这些用途SHALL进入整个方法的同次证明；已支持直接用途可以按同一契约验收，未知用途或任何参数重赋值必须保留raw声明与明确拒绝，不能只扫描protected body而漏掉catch

#### Scenario: Physical range or ownership evidence is changed

- **WHEN** 异常行缺失、重复、range不同、catch-all，或body/handler/continuation有缺失、重复、跨范围或竞争owner，或Code/SSA/effect库存不完整
- **THEN** 普通try泛型投影MUST拒绝，保留已有可靠正文和物理身份；不得借用同handler地址或canonical start匹配错误的行，也不得新认领未证明区域

#### Scenario: Unsupported exception topology stays outside this admission

- **WHEN** 泛型候选具有本项未证明的多range、多catch、nested try、finally、TWR或monitor拓扑
- **THEN** 本项SHALL继续拒绝该投影，不因其文本包含foreach或已有其他结构规则成功就扩大此普通try准入范围

### Requirement: List projection retains alias cast and overload semantics

泛型List声明与调用投影 SHALL 继续以实际物理receiver、参数读取及源级消费者闭合为前提。只在精确 `add(E)` 的实际赋值类型得到证明后，SHALL移除其合成Object呈现包装。系统 MUST 保留真实 `Iterator.next()` 后的checkcast、raw alias/heap pollution的异常时序，以及 `remove(Object)` 与 `remove(int)` 的重载区别；未知调用、wildcard写入、raw别名逃逸和不闭合实参必须拒绝相关泛型投影。

#### Scenario: An unknown call consumes a List parameter

- **WHEN** try循环将List参数作为未知调用实参，或参数经raw局部别名写入不兼容元素
- **THEN** 系统MUST保持raw声明与拒绝说明，保留别名、调用、真实cast和行为；不得因为foreach的元素cast与Signature相同就假定所有用途安全

#### Scenario: Object and index overloads coexist

- **WHEN** 方法含 `List<Integer>.remove(Object)`、`remove(int)`或明确的Object实参cast
- **THEN** 输出MUST保持每次调用的descriptor与实际重载；未经本项有限用途证明的候选继续拒绝，不能全局删除Objectcast或把索引操作改成值操作

### Requirement: Ordinary try generic projection publishes atomically and is compared as a whole class

泛型声明、调用改写与来源映射 SHALL 在全部证明及重发射成功后一次发布。预算耗尽或取消 MUST 返回原停止状态且不发布部分候选、部分声明、部分正文或部分来源；证明拒绝须保留可靠raw输出与拒绝原因。完整类验收 SHALL 比较未手修的原class/JADX/Jarde产物，包含双JDK、debug/none与default/all；Jarde与原类的值、异常类型/消息、集合、调用事件及副作用必须相同。

#### Scenario: Stop after ordinary try admission

- **WHEN** 扫描已进入普通try证明或调用overlay阶段后预算耗尽或取消
- **THEN** 系统MUST原样传播Stop，不提交半个投影；相邻no-handler方法的既有准入与停止契约保持

#### Scenario: Iterator and output calls throw inside the catch range

- **WHEN** 受控完整类分别在 `iterator()`、`hasNext()`、`next()`、输出 `add()` 注入RuntimeException或Error，包含String与Integer路径
- **THEN** Jarde输出SHALL与原类保持相同catch/escape行为、消息、调用顺序和effects；异常不能因header或前缀移动到try外而改变

#### Scenario: A comparison provider fails to compile

- **WHEN** JADX完整生成类在真实编译中失败
- **THEN** 验收SHALL保存其原始源码和失败raw并记为no-runtime；不得修补、删除方法、改写harness或把未执行结果视为行为等价，也不得由局部通过宣称整个CF11或71单元账本完成

#### Scenario: Evidence selection and origins agree

- **WHEN** 同一物理方法在default与all证据选择下恢复并投影
- **THEN** 输出声明与正文SHALL一致，实际来源映射绑定该物理方法和真实指令边界，语义操作来源不得因wrapper移除丢失；已有纯transfer映射缺口须如实单列，不能由非空映射或coverage分类宣称全指令覆盖
