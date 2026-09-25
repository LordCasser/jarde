## ADDED Requirements

### Requirement: 注解类型声明使用 Java 可编译头部

当 class 文件的 `ACC_ANNOTATION | ACC_INTERFACE` 声明及接口表确认为 Java 注解类型的 canonical 形状时，class-source SHALL 输出 `@interface Name` 而不输出显式 `extends java.lang.annotation.Annotation`。报告 MUST 继续保留该 class 的原始接口表、flags 和物理身份；此源代码拼写不得修改普通接口与普通类的父类型语义。

#### Scenario: Canonical annotation type with ordinary defaults
- **WHEN** Java 8 注解类型只声明可呈现的整数、字符串和数组默认值
- **THEN** 输出完整注解类 SHALL 通过 `javac --release 8`，反射取得的所有默认值与原 class/JADX 完整类相同，源码头没有显式 `extends`

#### Scenario: Ordinary interface and class inheritance
- **WHEN** class 文件声明一个普通接口继承父接口或一个普通类实现接口
- **THEN** 原有 `extends` 或 `implements` 发射 SHALL 不变，不把普通接口误写成注解，也不丢弃其父类型

#### Scenario: Physical declaration and adjacent unsupported defaults
- **WHEN** 请求声明事实或完整来源，或同一个注解类型包含当前尚不支持的嵌套默认值
- **THEN** 原始 interface 列表 SHALL 仍见于结构化报告；头部修复 MUST 不伪称已恢复嵌套默认值，也不得从 class 文件中删除该事实
