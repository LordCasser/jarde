## ADDED Requirements

### Requirement: 协变数组异型存入的宽化呈现

当数组元素存入的接收者是引用数组、其呈现组件型与存入值类型不兼容（协变存入形）时，系统 SHALL 将该存入点的访问接收者宽化为 `Object[]` 呈现（`((Object[]) local)[idx] = value`），保持编译性与运行时数组存入检查语义。

#### Scenario: 协变存入主锚
- **WHEN** 输入为固定 `AS`（`Object[] a = new String[2]; a[0] = Integer.valueOf(1)` 形，`javac --release 8`）的 class 并恢复
- **THEN** 存入 SHALL 可编译（剥离编译 exit 0）且 `-Xverify:all` 运行时 ASE 触发与原类一致

#### Scenario: 同型与兼容存入零回退
- **WHEN** 接收者组件型与值类型兼容
- **THEN** 渲染 SHALL 逐字节不变

#### Scenario: 非引用数组不适用
- **WHEN** 接收者是基本型数组或非数组
- **THEN** 现有呈现/拒绝 SHALL 保持
