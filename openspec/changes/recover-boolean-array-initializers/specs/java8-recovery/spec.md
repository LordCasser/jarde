## ADDED Requirements

### Requirement: Proven boolean array initializers preserve integer low bits

对已证明可折叠的新建 `boolean[]` 初始化器，系统 SHALL 将真实 `bastore` 所消费的可呈现整数元素表达为可重编译 Java，并保持原 JVM 逐元素最低位结果、初始化次序及返回数组内容。该要求 MUST NOT 授权尚未证明闭合的数组写入链。

#### Scenario: Multiple raw integer elements

- **WHEN** 合法 Java 8 class 用实际 `newarray boolean` 与逐元素 `bastore` 构成已证明初始化器，元素为 0、1、2、3、-1 等整数
- **THEN** 完整恢复类 SHALL 用 Java 8 编译并严格验证执行，按原 JVM 顺序返回 false、true、false、true、true，不把非零偶数当 true

#### Scenario: Existing proven boolean elements

- **WHEN** 每个写入元素已有 boolean 证明或可直接拼写的 0/1 常量
- **THEN** 恢复 SHALL 保留合法且等价的 boolean 初始化器，不加入多余整数转换

#### Scenario: Effectful elements and exceptions

- **WHEN** 闭合初始化器中的元素生产者有可观察副作用或抛错
- **THEN** 恢复 SHALL 每个生产者至多执行一次并保持原数组分配后、逐元素写入的先后、异常及已发生的可观察副作用；不能因折叠而提前、复制或漏掉生产者

### Requirement: Initializer conversion is bound to each proven store

系统 SHALL 仅在数组组件、元素值和其对应真实写入指令均有证明时作最低位转换，且每个转换的来源 SHALL 指向对应的 store，而非仅指向整体数组使用位置。

#### Scenario: Distinct element and store origins

- **WHEN** 一个初始化器含多个元素和不同真实 `bastore` BCI
- **THEN** 完整来源 SHALL 分别保留每个元素生产者、对应 store BCI 与整体数组/消费 BCI；任何两个元素不得误共用同一 store 来源

#### Scenario: Unproved or mismatched chain

- **WHEN** 数组组件未知/非 boolean、对应 opcode 非 `bastore`、元素不能呈现为 B/C/S/I 或已有 boolean 证据，或闭包/异常处理器/顺序证明不足
- **THEN** 本项 MUST NOT 投影低位 boolean 初始化器；系统 SHALL 沿现有安全路径保留原写入或可定位拒绝，不把普通 `bastore` 的准入推给未证明折叠

#### Scenario: Replay and bounded stop

- **WHEN** 对同一初始化器请求默认证据、完整证据、重放，或预算不足/请求取消
- **THEN** 成功正文 SHALL 相同；停止 SHALL 沿现有契约不发布部分初始化器、半张来源表或重复生产者
