## ADDED Requirements

### Requirement: 同包直接父类的非公开实例字段写入保留物理 owner

系统 SHALL 在已选中的 `B extends A`、二者同包、物理 `putfield` 精确指向 `A` 中唯一同名同描述符的 `protected` 或包可见实例字段、SSA 接收者确为 `B` 时，恢复一次写入 `A` 所拥有字段的 Java 8 源语句。子类隐藏同名字段时，源码 MUST 使用能选中 `A` 字段的接收者表达式；证据缺失时 MUST 保留原拒绝和 BCI 来源，不得把写入改绑到 `B`。

#### Scenario: 同包 protected 和包可见字段被子类隐藏
- **WHEN** `B extends A` 且两类同包，`B.set` 的两条 `putfield` 分别精确写 `A.protectedField` 与 `A.packagePrivateField`，`B` 也声明两个同名字段
- **THEN** Jarde 完整源码经 Java 8 重编和验证运行后，反射读取 `A` 的两个字段均为 true、`B` 的两个字段均为 false，且与原 class 和固定 JADX 的运行结果一致

#### Scenario: 字段身份或可访问性不成立
- **WHEN** CP owner 不是选中直接父类、name/descriptor 不匹配或声明不唯一，字段为 private/static/final，父子类不同包，或实际 SSA 接收者不是当前子类类型
- **THEN** 系统 MUST 不发出本项父字段写入证书，保留该字段指令的拒绝与 BCI 来源

#### Scenario: 既有公开字段和 private accessor 路径不回归
- **WHEN** 输入分别为已证明的直接父类 public 实例字段和唯一 Java 8 private setter accessor
- **THEN** 既有 owner cast、物理 helper 调用和完整源码的编译运行行为 SHALL 保持不变
