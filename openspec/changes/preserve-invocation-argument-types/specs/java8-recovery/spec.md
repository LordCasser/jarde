## ADDED Requirements

### Requirement: Presented invocation arguments retain the symbolic parameter types safely

对于参数可通过已知同型、数值加宽、范围内窄常量、null、Object 上溯或已证明函数式目标呈现的调用，恢复文本 SHALL 保留池目标所要求的参数静态类型，使重编译仍选择对应参数签名。系统 MUST NOT 仅因赋值/返回可隐式转换而丢掉调用约束，也 MUST NOT 为匹配参数描述符合成无证据的可失败引用检查。无足够证据时 SHALL 明示该调用的缺口并保留参数生产者来源。

#### Scenario: Object arguments retain the Object overload

- **WHEN** Object/String 或 Object/String[] 重载并存，而池目标的参数是 Object，实际值为字符串、null、引用数组或装箱结果
- **THEN** 恢复正文重编译执行 SHALL 与原 class 选择同一 Object 目标，装箱或其它参数生产者 SHALL 只执行一次

#### Scenario: A poly invocation is constrained even when its erased type matches

- **WHEN** 外部泛型方法擦除后返回 Object，其结果传给池目标 Object 参数，同时存在 String 重载
- **THEN** 恢复文本 SHALL 固定实参的 Object 类型，MUST NOT 因擦除类型已相等而让重新推断改选 String

#### Scenario: Primitive invocation conversions retain narrow and wide targets

- **WHEN** char/byte/short 值供给 int 目标，或范围内常量供给 byte/short/char 目标，且存在其它数值重载或只有窄参数声明
- **THEN** 恢复文本 SHALL 可编译并选择原参数类型；赋值和返回的隐式加宽规则 SHALL 不因此改变

#### Scenario: Functional arguments keep the factory target

- **WHEN** Runnable 与 Supplier 重载并存，原工厂目标为 Runnable，实参是兼容两者的方法引用或 lambda
- **THEN** 外层调用 SHALL 保留 Runnable 选择；若该函数式对象又作为 Object 参数传入，文本 SHALL 同时保留可编译的函数式目标和 Object 参数类型

#### Scenario: Constructors and multiple arguments preserve selection and evaluation

- **WHEN** new、this/super 构造器调用或普通多参数调用的静态参数类型会影响重载选择
- **THEN** 可恢复文本 SHALL 保持原目标、从左到右的求值顺序、次数及异常，不得只修普通单参数调用

#### Scenario: Unknown reference relations do not invent runtime checks

- **WHEN** 参数值仅声明为 Object，调用需要 Runnable，原字节码没有对应检查也没有足够安全转换证据
- **THEN** 恢复 SHALL 拒绝该调用并保留来源，MUST NOT 合成 Runnable cast 将可执行原 class 的返回值改成 ClassCastException

#### Scenario: Contextual type spelling retains provenance and bounded work

- **WHEN** 调用点补充静态类型呈现或因转换证据不足拒绝，且请求来源证据或触发预算终止
- **THEN** 参数和调用的原始来源 SHALL 保留，补充类型 MUST NOT 冒充不存在的原 cast 指令；工作 SHALL 沿用现有预算与取消传播，MUST NOT 增加外部方法体读取
