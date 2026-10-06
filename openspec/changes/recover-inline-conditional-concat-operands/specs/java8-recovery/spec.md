## ADDED Requirements

### Requirement: 内联条件值作拼接链操作数

当一条 `+` 拼接链的中间块恰好构成一个已证条件值物化（两臂各常量、唯一 join、无其它副作用），且该值由链的 append 实参位消费时，系统 SHALL 呈现整条链（比较操作数按源码形态内联）。

#### Scenario: 引用等值内联主锚
- **WHEN** 输入为固定 `NI`（`… + (a == b) + …` 形，`javac --release 8`）的 class 并恢复
- **THEN** 表达式 SHALL 完整呈现且剥离编译后 `-Xverify:all` 输出与原一致（`3/10/5/true`）

#### Scenario: 无分支与存储后形零回退
- **WHEN** 输入为 `NMA`/`NI2`（无分支四读）或既有 scv-concat 存储后消费锚
- **THEN** 渲染 SHALL 逐字节不变

#### Scenario: 真跨块链仍拒
- **WHEN** 分支臂含副作用、双比较嵌套或链跨异常表
- **THEN** `jre_concat_split` 拒绝 SHALL 逐字保持
