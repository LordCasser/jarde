## ADDED Requirements

### Requirement: 循环测试位 copy-and-store 的呈现

当 dup 值的存储满足双消费舞蹈身份（store 与测试各一消费）、其槽由方法体声明、且呈现位置是该循环测试的原位（其余位置保持既有移动规则）时，系统 SHALL 将测试表达式按源码复合赋值-比较形呈现（`while ((c = read()) != -1)`），方法行为完整。（root 2026-10-08 验收时更正：初版"读者全部是测试"与 readAll 场景矛盾——其 store 目标被体读，实现以双消费身份+体声明槽+原位表达式为准。）

#### Scenario: FileReader char 循环主锚
- **WHEN** 输入为固定 `IO`（readAll 形，`javac --release 8`）的 class 并恢复
- **THEN** readAll SHALL 完整呈现（IO 类 0 引注）且剥离编译后双腿文件读取驱动（含 EOF）输出与原类一致

#### Scenario: 五族零回退
- **WHEN** 输入为 copy/快照/dup-store/guard 各族锚
- **THEN** 渲染 SHALL 逐字节不变

#### Scenario: 多读者与违反形仍拒
- **WHEN** store 后读者不止测试，或不可观察性违反
- **THEN** 拒绝 SHALL 逐字保持
