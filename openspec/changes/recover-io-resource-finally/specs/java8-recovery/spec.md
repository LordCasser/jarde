## ADDED Requirements

### Requirement: 资源句柄跨 finally 的 IO 读取呈现

当方法的形状是资源卫（资源局部在保护体内被循环读取、在 finally 中被 close、行集覆盖同一 finally、close 接收者 SSA 同一、body 含循环与会抛调用）时，系统 SHALL 按 `try { … } finally { r.close(); }` 源码形态呈现，方法行为完整。

#### Scenario: 三层包装链主锚
- **WHEN** 输入为固定 `IO`（BufferedReader/InputStreamReader/FileInputStream 链 + readLine 循环 + finally close，`javac --release 8`）的 class 并恢复
- **THEN** 两方法 SHALL 完整呈现且整类剥离编译后 `-Xverify:all` 输出与原一致

#### Scenario: 平台 ctor 实参宽化
- **WHEN** `FileInputStream` 作 `InputStreamReader` ctor 实参或 `InputStreamReader` 作 `BufferedReader` ctor 实参
- **THEN** 按 `platform_reference_argument_widens` 既有渲染呈现（表行封闭，javap 转录）

#### Scenario: 单行锁卫与多资源仍拒/不变
- **WHEN** 输入为 LK 三法（渲染逐字节不变）或多资源嵌套 try/close 带返回值形
- **THEN** 渲染/拒绝 SHALL 逐字保持
