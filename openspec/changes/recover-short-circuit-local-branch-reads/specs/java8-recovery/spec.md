## ADDED Requirements

### Requirement: 短路布尔局部的分支条件读取

当一个已证明的短路布尔局部（单写、声明区、同词法 region）被 `ifeq`/`ifne` 分支测试读取——三元条件位（`b ? x : y`）、语句条件位（`if (b)`）或链中段测试位（`a && b && c` 的 `b`）——系统 SHALL 按源码形态呈现该读取，方法行为完整。加载位置不动（分支即其消费者），故求值序不变；分支两臂由既有条件值/语句呈现承接。

该准入是消费者白名单的**分支臂**：消费者 opcode 为 `0x99`/`0x9a` 且解码语义为 `JumpIfZero`/`JumpIfNotZero`。数值分支（`iflt`/`ifgt` 等）读取同一加载值不构成布尔位，保持拒绝。

#### Scenario: 三元读主锚
- **WHEN** 输入为固定 `OP2`（`boolean b = …; return b ? x : -1;`，`javac --release 8`）的 class 并恢复
- **THEN** 方法 SHALL 完整呈现且整类剥离编译后 `-Xverify:all` 输出与原一致

#### Scenario: 语句条件位与反向零测试
- **WHEN** 已存布尔局部作 `if` 语句的条件，或以 `ifne` 读取（源码 `if (!b)` 形）
- **THEN** 两者 SHALL 按源码形态呈现（同一分支臂准入；`ifne` 形按实际分支语义拼写，两臂互换）

#### Scenario: 链中段测试位
- **WHEN** 已存布尔局部是另一条短路链的中段测试（`(x > 0) && b && (x < 100)`）
- **THEN** 该读取 SHALL 被同一分支臂准入并按源码形态呈现（链的区域合成可拼写为嵌套形，求值序不变）

#### Scenario: 循环条件位保持拒绝（跨词法 region）
- **WHEN** 已存布尔局部作 `while (b)` 循环的条件
- **THEN** 该读取 SHALL 保持拒绝（gate 的跨词法 region 判据先于消费者白名单生效：循环头是其自身的 canonical block，读取路径与声明路径不同）；循环本身仍按既有 Loop 条件呈现，局部类型保持 `int`，链区域整体引用

#### Scenario: 既有消费位与跨 region 零回退
- **WHEN** 输入为 putstatic-Z / ireturn-Z / append-Z 既有锚，或跨词法 region 读形（catch 内写 try 外读、循环头读）
- **THEN** 渲染/拒绝 SHALL 逐字不变
