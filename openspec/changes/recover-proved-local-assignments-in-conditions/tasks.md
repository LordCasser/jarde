## 1. 物理证明与拒绝边界

- [x] 1.1 对照固定 CF-06 的 `javap`、SSA、区域及局部声明流程，为 BCI 11/12/14 和 12/13/14 的 `dup; store; test` 写准确单次消费证书；以定向测试显示两正例仅被现有复制/短路门拒绝。
- [x] 1.2 构造额外复制消费者、错局部类型/作用域、异常边与交错副作用负例；验证所有负例保留 BCI quote、没有假完整源码。

## 2. 源级局部赋值表达式

- [x] 2.1 给现有表达式 AST/打印器增加最小局部赋值形状及 Java assignment 优先级/类型/来源；用打印测试验证比较与判空中的括号、右值求值次序。
- [x] 2.2 在现有条件值/短路测试闭包中消费证书，令已证 store/dup 归表达式一次、局部声明与后续读取仍由原词法计划完成；运行 CF-06 定向正负例并检查两方法无重复调用或字段读取。
- [x] 2.3 验证低预算、取消和不完整赋值候选均不发布部分方法/来源映射；运行相邻链式赋值、条件值、局部作用域和 CF-03/CF-07 定向回归。

## 3. 三方完整类验收

- [x] 3.1 提高 `cf06-inner-assignment/replay.py` 对 Jarde 完整类的重编与运行硬门槛；原 class、固定 JADX、Jarde 均以 Java 8 重编、`-Xverify:all` 运行七行逐字一致并记录源码 SHA-256。
- [x] 3.2 更新 CF-06 报告并运行 `cargo fmt --check`、`git diff --check`、相关测试与 `openspec validate recover-proved-local-assignments-in-conditions --strict`；注明仍未覆盖的字段/数组赋值左值及 Smali 弱断言。
