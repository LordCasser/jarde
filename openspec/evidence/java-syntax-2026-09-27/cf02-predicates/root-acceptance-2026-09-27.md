# CF-02 主线独立验收

Root 将实现 `4971ead40d8b0a07ae0327d4763be407ebbb0c9a` 拣入主线为 `c1ad42aa` 后，重新构建 CLI，在新空目录独立执行 [replay.py](replay.py)。固定 JADX 修订和测试/实现哈希通过；原 class、JADX、Jarde 的**完整 `Predicates` 类源码**均以 Java 8 重编、`java -Xverify:all` 运行，十行逐字相同，末行 `named("x")` 均为 `true`。四条效果路径另有 `false:0`、`false:0`、`false:1`、`true:1` 的原/Jarde 一致结果。主线 Jarde `Predicates.java` 与 [`accepted/source/jarde`](accepted/source/jarde) 逐字节相同。`jarde-java` 227 项库测试、格式和 OpenSpec strict 验证通过。

该实现只准入已证的单一布尔尾 `ireturn` 闭包；复杂异常边、`TestTernary3` 外部类型层级及 CF-03 的多分支共享尾未据此算追平。
