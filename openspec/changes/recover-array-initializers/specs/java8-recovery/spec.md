## ADDED Requirements

### Requirement: Proven array initialization chains recover as one Java expression

当单一数组分配和后续存储能被现有字节码、SSA 身份与类型事实完整证明为 Java 一维数组初始化器时，系统 SHALL 用一个数组创建表达式呈现该链，并保持真实的元素值与结果数组身份。

#### Scenario: Primitive and reference literals

- **WHEN** 常量长度为 N，N 条存储按 `0..N-1` 恰好写入同一新分配的 `int[]` 或已证明组件赋值合法的引用数组，且数组在此期间不逃逸
- **THEN** 系统 SHALL 写出 `new T[]{e0, …, eN-1}`，完整类可编译，返回或消费的数组长度、顺序和值与原 class 一致

#### Scenario: Empty allocation remains stable

- **WHEN** 长度为零且没有初始化 store
- **THEN** 已有 `new T[0]` 恢复 SHALL 保持可编译与行为一致，不要求改变源码风格

### Requirement: Array initialization preserves effects, boundaries and evidence

已证明的初始化表达式 SHALL 沿既有受限执行、求值位置与来源合同提交；证明不足 SHALL 保守拒绝，不产生可编译但执行不同的正文。

#### Scenario: Effectful elements evaluate once and in order

- **WHEN** 每个元素来自会记录调用或抛出可识别异常的表达式
- **THEN** 分配 SHALL 在元素之前，元素 SHALL 从左到右各求值一次；原 class 与重编译 jarde 的 trace、返回值、异常类别和身份 SHALL 逐项一致

#### Scenario: Non-initializer stores are not folded

- **WHEN** 长度是动态值、索引重复/跳跃/倒序、数组逃逸或别名可见、链跨块/异常边界，或引用组件赋值不能证明等价
- **THEN** 系统 SHALL 保留现有普通数组语句或来源完整的拒绝，不把写入猜成 `new T[]{…}`

#### Scenario: All owned bytecode remains traceable

- **WHEN** 初始化链成功提交或预算/取消/证明使其拒绝
- **THEN** 分配、copy、索引、元素生产者及各 store 的真实 BCI SHALL 分别可追踪且不重复执行；默认与完整证据 SHALL 使用同一正文
