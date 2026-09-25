# 合法三入口局部汇合：条件值规则边界

永久输入是 `tests/fixtures/p3-conditional-values/v8/ConditionalBoundarySwitch.class`（Java 8，545 B，7 Code，SHA-256 `453c00b9ebb0974d084aca7a28e465c52c42ab8b92112ac0e913eb9c8d55a7ee`）；源码与 source-only runner 在同目录。`choose(I)I` 的 `lookupswitch` 三条臂各执行一次调用，再把结果写到同一局部变量；三个不同前驱汇合后返回。它不满足 `Region::If` 的双臂 stack Phi 前提，因此新增条件表达式规则不得认领它，但已有普通 `switch` 路径可以恢复。

对照使用 JADX 1.5.6 与 CLI SHA-256 `3c65e6ec6766b83163b29ea62e0b551ef902f76262f82538aa74db399b726754`。本目录保存两份反编译完整类、编译/运行日志和 `summary.json`；原源码、JADX 与 Jarde 的完整类均以 `javac --release 8 -g:none` 编译，并以 `java -Xverify:all` 执行成功。四行输出的 SHA-256 都是 `2c02c050e89418088bfdc6fc82ecb50aa5abdc1b2ef31711fb75a6ad05e7e962`。Jarde 的 `choose` 保留 `switch`、三臂写入和汇合后的 `return local1`，没有 `?:` 或 `@bytecode`；此处不应把“条件值规则拒绝”误写成“整类拒绝”。

这一比较只冻结任务 1.2 的多入口反例。循环、异常边界和类型不明的证明/拒绝仍未完成，2.1 的新 proof 尚未用于本次 CLI 输出。
