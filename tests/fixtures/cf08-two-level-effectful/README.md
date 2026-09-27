# CF-08 双层分支的拒绝边界

`TwoLevelIfNegatives` 保留外层判空、内层长度判断与结果汇合，分别改变循环入口、出口目标、内层汇合路径、汇合前驱数、效果调用数、异常边和体内结果的额外消费。测试只要求新的三来源双出口证书拒绝这些形态，并保留对应物理 BCI；其他既有恢复规则可独立处理其已证明的部分。

以下命令从仓库根目录重建 Java 8 class 并验证可加载运行：

```sh
javac --release 8 -g:none -Xlint:-options -d tests/fixtures/cf08-two-level-effectful \
  tests/fixtures/cf08-two-level-effectful/cf08twolvl/TwoLevelIfNegatives.java \
  tests/fixtures/cf08-two-level-effectful/cf08twolvl/VerifierRunner.java
java -Xverify:all -cp tests/fixtures/cf08-two-level-effectful cf08twolvl.VerifierRunner
```

本次验证输出依次为 `8 / 9 / 7 / 11 / 13 / 14 / 15`。两个 class 的 SHA-256 分别为 `8f29f5db6c51e8a62fbcec09b4f162e498004ba19159daa56721563a2af1b1be` 和 `55499494c10891f9a3bb2d29dc32a27b75a7383ae801dac744ab687f8ab28f0a`。
