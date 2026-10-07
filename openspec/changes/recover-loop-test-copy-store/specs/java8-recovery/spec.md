## ADDED Requirements

### Requirement: 循环测试位 copy-and-store 的呈现

当 dup 值的 store 目标的读者全部是该循环测试（及已证提升位）时，系统 SHALL 将测试表达式按源码复合赋值-比较形呈现（`while ((c = read()) != -1)`），方法行为完整。

#### Scenario: FileReader char 循环主锚
- **WHEN** 输入为固定 `IO`（readAll 形，`javac --release 8`）的 class 并恢复
- **THEN** readAll SHALL 完整呈现（IO 类 0 引注）且剥离编译后双腿文件读取驱动（含 EOF）输出与原类一致

#### Scenario: 五族零回退
- **WHEN** 输入为 copy/快照/dup-store/guard 各族锚
- **THEN** 渲染 SHALL 逐字节不变

#### Scenario: 多读者与违反形仍拒
- **WHEN** store 后读者不止测试，或不可观察性违反
- **THEN** 拒绝 SHALL 逐字保持
