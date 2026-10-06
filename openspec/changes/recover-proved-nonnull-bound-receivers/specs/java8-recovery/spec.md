## ADDED Requirements

### Requirement: 可证非空绑定接收者的 lambda 适配

当方法引用站点的绑定接收者捕获值满足：其 SSA 定义是分配指令（可证非空），且捕获点之后该局部槽无进一步 store（值捕获与槽捕获等价）时，系统 SHALL 按 lambda 适配呈现该站点并保持方法行为完整。

#### Scenario: ifPresent 绑定接收者主锚
- **WHEN** 输入为固定锚（`StringBuilder sb = new StringBuilder(); o.ifPresent(sb::append)` 形，`javac --release 8`）的 class 并恢复
- **THEN** 调用语句 SHALL 完整呈现且剥离编译后 `-Xverify:all` 输出与原 class 一致（`[S]` 形）

#### Scenario: 可空接收者仍拒
- **WHEN** 绑定接收者是参数读或字段读（定义非分配）
- **THEN** 拒绝文本 SHALL 逐字保持（"adapting this bound receiver would move its null failure from functional-value creation to invocation"）

#### Scenario: 捕获后重写仍拒
- **WHEN** 捕获点之后该局部槽被重写
- **THEN** 拒绝 SHALL 保持，不得产出与源捕获值语义不同的文本

#### Scenario: 既有绑定拒绝形零回退
- **WHEN** 输入为 `recover-typed-functional-method-references` 冻结的绑定实例拒绝锚
- **THEN** 渲染 SHALL 逐字节不变
