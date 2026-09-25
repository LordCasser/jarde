## ADDED Requirements

### Requirement: Instanceof preserves boolean results and operand evaluation

系统 SHALL 在已有可恢复区域内呈现真实类型测试，正确处理null、类、接口和数组目标及其boolean用途，保留操作数的单次求值、旧值、原有检查与异常。恢复的Java静态类型上下文 MUST 允许该类型测试，不能因源码加宽在字节码中消失而生成不相容的类型比较；MUST NOT 为此引入目标类型的收窄检查、折叠测试为常量或丢失生产者。无法可靠呈现的消费者 SHALL 保留类型测试与必要生产者来源。

#### Scenario: Object inputs flow through existing boolean contexts

- **WHEN** Object参数或调用结果被测试为普通类、接口、原始数组、引用数组或多维数组，并直接返回、写入boolean局部或由已有分支使用
- **THEN** 恢复完整正面类 SHALL 能按Java8重编译，null和各兼容/不兼容输入的值及调用次数与原class一致，不把boolean写成int或错误的零比较

#### Scenario: Widening disappeared from bytecode

- **WHEN** 合法原源码先把String、String数组或返回String的调用安全上溯Object，再测试为不相容的Integer、int数组或Runnable，而字节码没有记录该加宽
- **THEN** 恢复 SHALL 保留可编译的引用静态上下文，实际行为与原class一致；不能输出String instanceof Integer等被javac拒绝的文本，也不能改为会抛CCE的目标类型cast

#### Scenario: Original checks and abrupt completion are retained

- **WHEN** 操作数包括显式checkcast、一次性调用、null或生产者自身抛错
- **THEN** 原检查与调用 SHALL 按原顺序只求值一次；null测试为false，生产者错误或原checkcast的CCE保持其位置，不因输出已知boolean值而省掉效果

#### Scenario: A recovered functional value retains its target type

- **WHEN** 已由现有规则恢复的lambda或命名方法引用直接供给类型测试，字节码中的函数式工厂类型可证明
- **THEN** 输出 SHALL 保留可编译的函数式目标及必要安全加宽，不能把Object作为lambda的函数式目标；原工厂及类型测试不能被常量结果替换

#### Scenario: Unsupported consumers remain explicit

- **WHEN** 类型测试结果被丢弃、重复消费、用于不合法的Java整数上下文，或经过本项未覆盖的0/1汇合
- **THEN** 未证明部分 SHALL 明确引用相关bytecode、测试与必要生产者BCI/物理成员，不能生成非法表达式语句、boolean算术或漏掉延期调用

#### Scenario: Evidence and budget follow committed output

- **WHEN** 请求默认或全部来源，或正文/来源预算在新表达式处耗尽
- **THEN** 默认请求 SHALL 不构造未请求来源，完整映射 SHALL 覆盖真实测试及操作数；正文停止不发布半个表达式，来源停止不改变已提交正文或发布未支付映射
