## ADDED Requirements

### Requirement: 分配点双括号初始化呈现

当一个匿名子类（池名 `X$N`）的体是纯实例初始化块（无方法/字段声明）、其分配点是该值的唯一使用、且超类可拼时，系统 SHALL 在使用点呈现双括号源级形（`new Super(args…){ { body… } }`）并从文本消隐伴生类。

#### Scenario: 集合构建主锚
- **WHEN** 输入为固定 `DB`（无捕获与捕获双形，`javac --release 8`）的 class 并恢复
- **THEN** 分配点 SHALL 呈现双括号形且整类剥离编译后 `-Xverify:all` 输出与原一致（`2/z`）

#### Scenario: A 路径负例文本不变
- **WHEN** 伴生声明方法、分配多点使用或超类不可拼
- **THEN** 渲染 SHALL 保持 A 路径（重排 ctor）文本逐字不变

#### Scenario: 顺序敏感零回退
- **WHEN** 输入为构造期虚分派与 CST 序控制件
- **THEN** 既有行为 SHALL 逐字不变
