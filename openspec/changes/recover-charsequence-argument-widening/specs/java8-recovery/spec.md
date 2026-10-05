## ADDED Requirements

### Requirement: CharSequence 实参 SHALL 按封闭实现者表扩宽

当调用的参数声明为 `java.lang.CharSequence` 且实参呈现类型是 release 8 javadoc 声明的 CharSequence 实现者（`java.lang.String`、`java.lang.StringBuffer`、`java.lang.StringBuilder`、`java.nio.CharBuffer`——封闭四行，Segment 为 9+ 不入表）时，系统 SHALL 接受该实参并按既有 `cast_argument` 形呈现——与 java.util 集合表和 Throwable 通道**逐字同构**的第三张平台小表。

实参类型**不在**实现者封闭表（如 `Integer`→`CharSequence`）或目标是其它接口（如 `String`→`Runnable`）时 SHALL 保持既有拒绝。既有 `DIRECT_EDGES` 集合表与 `java_lang_throwable_widens` 通道 SHALL 逐字不变。表内容 SHALL 以 javadoc 为准逐一核对，SHALL NOT 凭记忆外推或加入 9+ 实现者。

#### Scenario: String.join 恢复

- **WHEN** `static String join(String[] xs){ return String.join("-", xs); }`（首参 String→CharSequence）经 `class-source` 呈现
- **THEN** 方法恢复 0 引注（修复前整方法拒绝）；整类渲染源集 `javac --release 8` exit 0、`main` 输出与原 class 逐行一致

#### Scenario: 非实现者与其它接口仍拒绝

- **WHEN** 实参 `Integer` 传 CharSequence 参数位，或 `String` 传 `Runnable` 参数位
- **THEN** 保持既有拒绝——表是封闭的 javadoc 事实，不是通用接口扩宽

#### Scenario: 姊妹通道零回退

- **WHEN** java.util 集合扩宽与 Throwable 扩宽的既有测试经运行
- **THEN** 全部通过——两条既有通道逐字不变
