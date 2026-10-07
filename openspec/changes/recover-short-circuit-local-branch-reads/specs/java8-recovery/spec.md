## ADDED Requirements

### Requirement: 短路布尔局部的分支条件读取

当一个已证明的短路布尔局部（单写、声明区、同词法 region）被 `ifeq`/`ifne` 分支测试读取——三元条件位（`b ? x : y`）、语句条件位（`if (b)`）或循环条件位（`while (b)`）——系统 SHALL 按源码形态呈现该读取，方法行为完整。

#### Scenario: 三元读主锚
- **WHEN** 输入为固定 `OP2`（`boolean b = …; return b ? x : -1;`，`javac --release 8`）的 class 并恢复
- **THEN** 方法 SHALL 完整呈现且整类剥离编译后 `-Xverify:all` 输出与原一致

#### Scenario: 语句与循环条件位
- **WHEN** 已存布尔局部作 `if` 语句或 `while` 循环的条件
- **THEN** 两者 SHALL 按源码形态呈现（同一分支臂准入）

#### Scenario: 既有消费位与跨 region 零回退
- **WHEN** 输入为 putstatic-Z / ireturn-Z / append-Z 既有锚，或跨词法 region 读形
- **THEN** 渲染/拒绝 SHALL 逐字不变
