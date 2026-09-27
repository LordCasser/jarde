## ADDED Requirements

### Requirement: 受证静态泛型成员声明保持嵌套作用域与桥语义

系统 SHALL 在根类与唯一静态抽象成员的物理关系、完整 Signature 作用域、成员表和编译器桥再生均被证明时，于根源码原子输出嵌套 `A<T> implements Comparable<A<T>>`、`T` 字段及 typed `compareTo(A<T>)`。输出的完整 Java 8 源码 SHALL 保持原 class 的泛型反射、合法调用及错误 Object 实参导致的桥参数 cast 异常；物理 child 报告仍可独立查询。任一事实不明时 MUST 不发布半份泛型声明。

#### Scenario: 无构造使用点的合法成员

- **WHEN** root 与唯一 child 的双向 `InnerClasses` 关系准确，child 完整，且 root 没有需要改写的 child 使用点
- **THEN** 声明关系可独立获证；不要求凭空存在构造调用或 `StaticNoCapture` 使用目标

#### Scenario: 签名与桥一致

- **WHEN** class/field/method Signature 的类型变量、擦除及物理父接口一致，物理 bridge 精确实现由该泛型声明生成的参数 cast、一次 typed 调用与同值返回
- **THEN** 根源码只写 typed 方法，重编后具有一个正确的合成 bridge，泛型反射与异常行为同原类

#### Scenario: 关系、签名或桥不一致

- **WHEN** child self row、接口表、类型变量、擦除、bridge flags/Code/目标/参数 cast 或根使用闭包不完整或冲突
- **THEN** 系统 MUST 拒绝泛型成员投影，并保留物理子类及具体拒绝证据

#### Scenario: 停止与来源

- **WHEN** 关系/Signature/bridge 证明或源单元写入被取消或耗尽预算
- **THEN** 系统 MUST 原子停止，不得发布只含部分成员或 raw 父接口的根源码；成功时派生类头、字段、方法和桥省略决定均可追溯到物理来源
