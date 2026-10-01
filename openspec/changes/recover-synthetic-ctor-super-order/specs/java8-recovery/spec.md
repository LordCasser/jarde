## ADDED Requirements

### Requirement: javac 合成构造器的 pre-super 字段存以 super 先行序呈现

系统 SHALL 在构造器的 super 调用之前存在连续的合成字段直传存组（字段具 ACC_SYNTHETIC 或名字为 `this$N`/`val$` 合成模式，存值为参数直传）时，将该组移至 super 调用之后按原序呈现，使 super() 成为源码首句、输出合法 Java。用户命名字段（无合成标志）、非直传值或交错形态 SHALL 保持现有逐字呈现；既有构造器呈现 SHALL 逐字不变。

#### Scenario: 捕获型构造器合法化

- **WHEN** 匿名/局部捕获类（`val$x`）或非静态成员内部类（`this$0`）的构造器以 family 三方 Java 8 重编
- **THEN** super() 首句、合成字段存其后，`javac --release 8` 通过，`java -Xverify:all` 运行与原 class 一致

#### Scenario: 非合成与交错形态不变

- **WHEN** pre-super 存的目标为无合成标志的用户字段、值非直传、或与其它指令交错
- **THEN** 输出与本变更前逐字一致
