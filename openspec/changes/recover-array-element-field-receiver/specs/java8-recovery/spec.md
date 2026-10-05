## ADDED Requirements

### Requirement: 数组元素接收者的字段读取按组件类型证明

当 `aaload` 的数组操作数具有可证引用数组类型时，系统 SHALL 让元素值携带该数组的组件类型，并以此证明元素接收者上的字段访问身份，使 `xs[i].field` 与 `for(T x : xs) x.field` 与同类型直接参数的字段读取同判。

#### Scenario: 静态查找表主锚（array-element-field-receiver 巡查）
- **WHEN** 输入为固定 `RG`（`static{ Item[] all={A,B,C}; for(Item c:all) BY_LABEL.put(c.label,c); }`，`javac --release 8`）的 class 并恢复
- **THEN** 循环体 SHALL 完整呈现 `BY_LABEL.put(c.label, c)`（无 "not one this run proved names the member" 引注）；去注释呈现（伴生 Item 并入）编译若成功，运行 `of("beta")`/`of("alpha")` SHALL 非 null 且 `of("?")` SHALL 为 null（与原一致）

#### Scenario: 直接元素与 for-each 双形
- **WHEN** 输入为固定 `RK`（`xs[0].label` 与 `Item y=xs[0]; y.label`）或 `RH`（for-each 字段累积）
- **THEN** 方法体 SHALL 完整恢复且行为与原一致（`q/q`、`5`）

#### Scenario: 不可证数组类型仍拒
- **WHEN** 数组操作数的组件类型无可证来源
- **THEN** 系统 SHALL 保持现行字段身份拒绝（不得猜测组件类型）

#### Scenario: 直接参数对照零回退
- **WHEN** 输入为固定 `RJ`（`direct(Item)`/`viaLocal`）或既有字段读取 corpus
- **THEN** 渲染与本变更前逐字节一致
