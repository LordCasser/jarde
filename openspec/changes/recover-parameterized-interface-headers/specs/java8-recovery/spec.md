## ADDED Requirements

### Requirement: 类头接口子句按 Signature 投影类型实参

系统 SHALL 在类 `Signature` 属性可解析、接口定义在选定环境中可解析（或为 Java 8 class-path 形态下已转录的
`java.lang.Comparable` 平台事实：一个类型参数，本机 rt.jar 转录见证据目录）、擦除与物理 `interfaces`
一致且唯一选定时，把类头 `implements` 子句投影为带类型实参的形（`implements Comparable<Impl>`），复用既有
类头投影通道（与 `extends Parent<T>` 同一环境、同一擦除核对、同一原子性）。任一接口不满足判据时 SHALL 保留
该接口的裸类型与物理来源，不得从调用或局部变量猜类型。类自身的头位置不可忠实发布时（类种类、名字、类型注解
等既有自有头门），SHALL 保留物理类头，不得新增拒绝。既有父类参数化投影与成员声明参数化 SHALL 逐字不变。

#### Scenario: 泛型接口在类头带类型实参

- **WHEN** `class Impl implements Comparable<Impl>`（类 `Signature` 记有该实参、擦除与物理 `interfaces` 一致）三方 Java 8 重编
- **THEN** 类头呈现 `implements java.lang.Comparable<BR$Impl>`，`javac --release 8` 通过且 `java -Xverify:all` 逐路径与原 class 一致（含经接口引用调用 `compareTo`）

#### Scenario: 接口不可解析时保留裸类型

- **WHEN** 接口定义不在选定环境（无 JRE image 或非快照内）
- **THEN** 类头保留该接口的裸类型、物理来源可查，且既有父类参数化投影与成员声明参数化输出逐字不变

#### Scenario: 接口定义与 Signature 自相矛盾时拒绝类头投影

- **WHEN** 接口定义在选定环境中可解析，但其类型参数个数与类 `Signature` 声明的实参个数不符，或类 `Signature`
  的擦除与物理 `interfaces` 不一致
- **THEN** 类头投影拒绝并如实诊断，不得以自相矛盾的声明拼写类头；物理类头文本保持不变

### Requirement: 擦除桥的隐藏以类头类型实参为前置

系统 SHALL 仅在桥的擦除契约所属父类型在类头文本中已带类型实参时隐藏该桥成员；类头为裸类型时 SHALL 保持桥可见并如实诊断，不得产出"桥已隐藏且类头裸类型"的文本（该形 javac 报未覆盖抽象方法）。

#### Scenario: 类头参数化后桥可安全隐藏

- **WHEN** `class ParamI implements Comparable<ParamI> { public int compareTo(ParamI o) { … } }`（类头已带类型实参）三方重编
- **THEN** 擦除桥被隐藏、源级覆写保留，`javac --release 8` 通过（javac 自行重建桥）且运行与原 class 一致

#### Scenario: 类头裸类型时桥保持可见

- **WHEN** 同一擦除契约所属接口在类头只能保留裸类型（`implements java.lang.Comparable`）
- **THEN** 桥成员保持可见（不出现"契约丢失 + 桥隐藏"的双重损失），输出带诊断说明；该文本的编译状态如实记录，不声称可编译

