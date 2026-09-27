## ADDED Requirements

### Requirement: 有身份保证的直接返回字面量装箱简写

对于完整 Java 8 方法中准确的标准包装类静态 `valueOf` 调用，当其唯一实参和唯一直接返回消费者证明 primitive 字面量的类型、值及返回转换，且语言规范与标准类库共同保证改写前后的包装对象身份时，Java 源码视图 SHALL 把显式调用呈现为 primitive 字面量。完整类源码 MUST 通过 Java 8 重编，并保持原 class 的包装类型、值、可观察引用身份及物理调用/返回来源。不满足任一证明时 MUST 保留显式调用或既有保守回退，不得仅凭固定编译器的一次运行结果扩大规则。

#### Scenario: 有保证的布尔、整数与字符装箱

- **WHEN** 准确 `Boolean.valueOf(Z)` 的布尔字面量、`Integer.valueOf(I)` 的 `-128..127` int 字面量或 `Character.valueOf(C)` 的 ASCII 字符字面量直接成为对应包装类或 `Object` 方法返回值，全部物理证据闭合
- **THEN** 源码可直接输出 `return true;`、`return 1;` 或 `return 'c';` 一类 primitive 形式；原 class 与重编后的类对类型、值和重复调用身份的观察一致

#### Scenario: 身份保证之外的 valueOf

- **WHEN** 调用是 `Byte`、`Short`、`Long` 的 `valueOf`，或 int/char 值超过上述保证范围，或 owner、descriptor、字面量类型、消费者/返回目标不完整
- **THEN** 保留原有显式 `valueOf` 或保守拒绝，不以源码简短为由制造跨编译器未证明的引用身份

#### Scenario: 预算或取消中断

- **WHEN** 证据读取、证明或输出在预算耗尽或取消时停止
- **THEN** 不发布半个装箱改写，报告保留停止原因及调用/返回的物理来源
