## 1. 冻结证明边界

- [x] 1.1 对照 `cf07-basic-loops` 的物理 CFG、现有循环 `Frame`/区域所有权与局部变量词法计划，写出终止返回叶的准确候选条件；用定向测试确认 BCI 19 当前仅因循环 scope 排除而成为 quote。
- [x] 1.2 构造额外入口、共享叶、非终止出口、异常边、返回值缺证的负例并固定拒绝断言；验证它们仍有 bytecode/来源而没有假完整正文。

## 2. 循环区域实现

- [x] 2.1 在已有循环区域中仅将完整证明的终止返回叶纳入该循环体可达范围，不新增 Region/AST 形状；验证命中分支的 `return local4` 只出现一次，BCI 19/21 获来源映射，循环正常出口仍在外层。
- [x] 2.2 保持现有局部定义使用、预算/取消与区域所有权拒绝路径；运行 CF-03 共享尾和 CF-07 循环定向测试，确认无重复归属或半份源码。

## 3. 三方验收

- [x] 3.1 提高 `cf07-basic-loops/replay.py` 的修后 Jarde 硬门槛，重建 CLI，独立输出原/JADX/Jarde 完整 Java 8 类源码并验证八行运行结果一致；保留三方源码哈希和 `javap`/区域证据。
- [x] 3.2 运行相关循环/局部作用域测试、`cargo fmt --check`、`git diff --check` 与 `openspec validate own-proved-loop-terminal-return --strict`，在 CF-07 报告中注明验收范围和剩余 CF-06 依赖。
