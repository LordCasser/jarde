## Why

当前 `lcmp/fcmpl/fcmpg/dcmpl/dcmpg` 尚未进入呈现事实，21 个自写 long/float/double 条件方法全部回退。jadx 虽能重编译，却在同组 1,309 个输入中产生 34 个 NaN 错值；已有条件 AST 足以正确表达这些极性，无需扩张区域系统。

## What Changes

- 对同一基本块中紧邻零分支、且结果只由该分支消费的数值比较，恢复两操作数的关系条件。
- 浮点条件保留 NaN 偏置及 taken/fall-through 含义；必要时写 `!(a < b)`，不能改成 NaN 上含义不同的 `a >= b`。
- 继续保留来源、最终消费位置与求值次数；不满足组合证据时明示引用，不静默消掉比较或其生产者。
- 建立源码、javac、jadx、jarde 对照及原 class/恢复正文的受控执行验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：新增 long/float/double 直接比较条件的呈现与 NaN 极性要求。

## Impact

前置条件为现有 SSA、分支极性、Binary/Not 与来源映射。范围是 `jarde-java` 的五种操作事实、既有条件构造、确实需要的直接消费准入和专项测试。没有新 crate、pass、解析器、类型闭包、外部库或被调方法体读取。

非目标：比较结果作为一般整数表达式、跨块或多个消费者、布尔 0/1 汇合、三元表达式、位运算、浮点常量、含调用的循环准入扩展，以及其它区域债务。已有 `present-proved-java-structure` 2c.7 由本项收敛承接，规划完成不代表已实现。
