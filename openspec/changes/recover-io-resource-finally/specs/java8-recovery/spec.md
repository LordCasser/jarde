## ADDED Requirements

### Requirement: 资源句柄跨 finally 的 IO 读取呈现

当方法的形状是资源卫（资源局部在保护体内被循环读取、在 finally 中被 close、行集覆盖同一 finally、
close 接收者 SSA 同一、body 含循环与会抛调用、完成形为保存返回值）时，系统 SHALL 按
`try { … } finally { r.close(); }` 源码形态呈现，方法行为完整。

#### Scenario: 三层包装链主锚
- **WHEN** 输入为固定 `IO`（BufferedReader/InputStreamReader/FileInputStream 链 + readLine 循环 + finally close，`javac --release 8`）的 class 并恢复
- **THEN** `countLines` SHALL 完整呈现（0 族拒绝、一个 `finally`、一次 `close()`），三腿（巡查 jar + 双腿）文本逐字节相同；同一证书形在 caller-owned 流上（`IOMidRead.countRemaining`）剥离重编译后 SHALL 在双腿 `javac` 下编译、`-Xverify:all` 下由同一驱动跑出与原生 class 逐字相同的输出（正常完成 + 中途读异常时 finally 的 close 均被观测）

#### Scenario: 平台 ctor 实参宽化
- **WHEN** `FileInputStream` 作 `InputStreamReader` ctor 实参或 `InputStreamReader` 作 `BufferedReader` ctor 实参
- **THEN** 按 `platform_reference_argument_widens` 既有渲染呈现（表行封闭，javap 转录）

#### Scenario: 三层构造链的深度边界
- **WHEN** 一条三层构造链（如三层包装链自身）或四层链
- **THEN** 三层 SHALL 呈现为单个 `new` 表达式；四层 SHALL 保持 `new@1` 最外层拒绝

#### Scenario: 单行锁卫、固定 void-loop 与多资源仍拒/不变
- **WHEN** 输入为 LK 三法（渲染逐字节不变）、固定 CF-16 void-loop 形（渲染逐字节不变）或多资源嵌套 try/close 带返回值形
- **THEN** 渲染/拒绝 SHALL 逐字保持

#### Scenario: 登记的 copy 族边界
- **WHEN** 输入为 `IO.readAll`（`FileReader` 手动 char 循环：其 loop test 的 copy-and-store dance 目标可观察）
- **THEN** SHALL 保持 copy 族循环测试纯度判据的拒绝逐字（登记边界；其后续片自带负例设计）
