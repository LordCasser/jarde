# 循环测试值：调用和字段读取

`present-proved-java-structure` 2c.6 的正例复用[永久 iterable fixture](../../java-syntax-2026-09-22/enhanced-for/string-iterable/StringIterableForeach.class)：Java 8 class SHA-256 为 `81a0dcb4a53a51eb721df72f2b30825b33e8fb4dc6f2390e93167f4012e09f36`。原始 Java 源与 runner 在 `../../java-syntax-2026-09-22/enhanced-for/`，旧版 Jarde 的 `@bytecode` 与缺少 return 记录也在该目录。此次从共享工作树构建的 Jarde CLI SHA-256 为 `4d8bd10e0d6ff09ab765e05242e9d0922687e372b18ea1cc6f56cddba03fd00a`；本地 JADX 源码版本为 `2fb1b1638694`，实际反编译程序为 1.5.6。

root 在独立临时目录对同一 class 分别取原始字节码、JADX 反编译源码和 Jarde `class-source --policy single-class --release 8 --format text`，各用 `javac --release 8` 重编并用 `java -Xverify:all` 跑原 runner。三份都编译、验证、运行 10 行；Jarde 10 行与原 class 逐行相同，`sumLengths` 的条件为 `while (local2.hasNext())`，恢复的类无 `@bytecode`。JADX 第 10 行仅将空元素的 helpful-NPE 局部归因写成 `Iterator.next()` 的返回值；原 class 与 Jarde 写成 `<local3>`，三者异常类型及 `iterator=1,hasNext=2,next=2` 完全相同。没有因恢复循环条件而重排或重复调用。

新[源码 fixture](../../../../tests/fixtures/p3-loop-test-values/LoopTestValues.java)与提交的 class 以 `javac --release 8 -g:none` 重编逐字节一致，SHA-256 为 `1b0cb0e941a2db027e955bce6b6f14171da18a3a65f1af680d4e0732f360ebe9`。[定向测试](../../../../tests/p3_loop_test_values.rs)共 4 项通过：`this.n` 只在条件中读取一次，`istore` 与 `iinc` 测试仍按 `StatementFree` 拒绝；相邻 `p3_forward_join` 3 项和 `p3_array_access` 11 项由实现代理验证通过。条件 SSA 依赖集合只准入被分支消费的调用/字段读取，独立调用不会进入集合；尚未单独造出含此形状的 verifier-valid 负 class。
