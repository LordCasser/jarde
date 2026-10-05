## ADDED Requirements

### Requirement: Comparable 实参 SHALL 按封闭实现者表扩宽

当调用的参数声明为 `java.lang.Comparable` 且实参呈现类型是 release 8 javadoc 声明的 java.lang 内 Comparable 实现者（`String`、`Byte`、`Short`、`Integer`、`Long`、`Float`、`Double`、`Character`、`Boolean`——封闭 9 行）时，系统 SHALL 接受该实参并按既有 `cast_argument` 形呈现——与 java.util 集合表、Throwable 通道、CharSequence 表**逐字同构**的第四张平台小表（含装箱参与的调用点：`int` 实参经既有装箱呈现后入表判定）。

实参类型**不在**封闭表（如 `Object`→`Comparable`）时 SHALL 保持既有拒绝。java.lang 之外的 JDK Comparable 实现者（`java.math` 等）SHALL NOT 入表（未实测）。用户类的 Comparable 实现走既有 snapshot 通道 SHALL 不变。

#### Scenario: 泛型方法调用点恢复

- **WHEN** `static <T extends Comparable<T>> T max(T a, T b)` 的调用点 `max("a","b")` 与 `max(1,2)`（擦除参数 Comparable；String 直传/int 装箱 Integer）经 `class-source` 呈现
- **THEN** 两调用点恢复（0 引注，cast 形呈现）；整类渲染源集 `javac --release 8` exit 0、`main` 输出与原 class 逐行一致

#### Scenario: 非实现者仍拒绝

- **WHEN** 实参 `Object` 传 Comparable 参数位
- **THEN** 保持既有拒绝——表是封闭的 javadoc 事实

#### Scenario: 姊妹通道零回退

- **WHEN** java.util 集合、Throwable、CharSequence 三条通道的既有测试经运行
- **THEN** 全部通过——既有通道逐字不变
