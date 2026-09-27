## ADDED Requirements

### Requirement: Bounded enum constant bodies with one String argument

在 Java 8 RuntimeProfile 下，系统 SHALL 仅在完整证据证明一个枚举恰有两个有序匿名常量体、每个常量只带一个可无损拼写的 ASCII 字符串字面量源实参、且两体与枚举共享构造关系等前提时；若枚举的 `ACC_ABSTRACT` 需要由接口方法实现解释，则仅当恰有一个直接接口、该接口恰有一个 `public abstract` 方法且两个常量体都精确实现该方法时，输出可用 `javac --release 8` 重编的常量体声明。呈现后的完整源码 SHALL 保持常量顺序、`name()`、`ordinal()`、常量实参、覆写分派和可观察构造副作用；不得把物理子类或访问桥当成源级构造参数。证据、依赖执行或预算不完整时 MUST 拒绝该联合投影、保留原始物理事实与来源，并且不得将结果标为已结构化恢复。

#### Scenario: Two String-argument constant bodies are recovered

- **WHEN** Java 8 enum 输入满足本切片的完整双常量、可无损拼写的 ASCII 字面量、构造转发、匿名体和预算前提
- **THEN** 完整呈现源码 MUST 能在 `javac --release 8` 下编译并通过 `java -Xverify:all`，且原版与重编运行在常量名、序号、字符串值和覆写行为上相同

#### Scenario: An uncertain constant bridge or argument is rejected

- **WHEN** 任一常量实参不是已知 ASCII 字符串字面量，构造桥未精确转发原 name/ordinal，常量/匿名体关系不唯一，抽象接口契约超出唯一直接接口/唯一抽象方法形态，或类事实、执行范围、预算不完整
- **THEN** 系统 MUST 拒绝双常量体投影并保留相关物理成员、字节码与 origin；MUST NOT 隐去 bridge 或猜测实参以输出看似完整的 Java enum

#### Scenario: Adjacent enum shapes remain outside this slice

- **WHEN** 输入没有匿名常量体、常量数量不同、使用多个/非 String 源参数、使用非 ASCII 字面量、构造实参为非字面量表达式，或包含捕获/未证明的构造链
- **THEN** 本要求 MUST NOT 授权该输入进入此证明切片；其它已有或后续要求分别决定是否可恢复
