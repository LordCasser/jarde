## ADDED Requirements

### Requirement: 非 ASCII（JLS 合法）标识符在声明层如实呈现

当类/字段/方法/参数名为 JLS Character.isJavaIdentifierStart/Part 合法的非 ASCII 标识符（如 CJK）时，系统 SHALL 在声明层呈现该名字本身，不得替换为 `__` 或其他占位符。

#### Scenario: CJK 字段与方法（unicode-identifier 巡查锚）
- **WHEN** 输入为固定 `UT`（UTF-8 源：中文字段 `变量`/`描述`、方法 `方法`、嵌套类 `内部类`，`javac -encoding UTF-8 --release 8`）的 class 并恢复
- **THEN** 声明层 SHALL 呈现 `变量`/`描述`/`方法`（无 `__` 替换与 "not a Java identifier" 注释）；去注释呈现文本经 `javac -encoding UTF-8 --release 8` 编译若成功，运行输出 SHALL 与原 class 一致（`变量=1/42/中文`）；方法体引用（现状已正确）SHALL 保持

#### Scenario: ASCII 路径零回退
- **WHEN** 任一现有 corpus class（ASCII 标识符）
- **THEN** 渲染与 corpus fingerprint 逐字节一致
