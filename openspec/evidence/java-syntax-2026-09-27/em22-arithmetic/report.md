# EM-22：算术优先级、eager 位运算及布尔异或对照

固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `arith/TestArith2` 明确断言 `(a+2)*3`、`a-(b-c)`、`a/(b/c)`、左结合加法和布尔 `|`/`&` 的源级括号。`arith/TestXor` 断言布尔 `x ^ true` 写成 `!x`，`x ^ false` 保留 `x`；`arith/TestArithNot` 的 `~` 断言来自 Smali 的直接 `not-int`/`not-long`，不能冒充 Java 8 `javac` 的正向恢复证据。`others/TestRedundantBrackets` 还混合 cast、`instanceof`、条件和数组形态，本次仅涉及其括号方向。四份测试文件的 SHA-256 由 [replay.py](replay.py) 核对。

[input/em22/](input/em22/) 用 Java 8 无 debug 信息编译完整 `Arithmetic` 与共同 Runner，覆盖非结合减/除、布尔 eager `|` 的双调用计数、整数/长整型 `~` lowered 为 `^ -1`、布尔参数与有副作用调用的 `^ true`/`^ false`。原 class、固定 JADX 与 Jarde CLI (`8f1f0012324350e4fc65c7fef4b3e3835102e6d4fda00df280b466270c98e727`) 的完整源码均通过 Java 8 重编、`java -Xverify:all`，八行输出逐字相同。独立第二次重放的摘要及两份反编译源码逐字节相同；[acceptance/](acceptance/) 保留源码、日志和哈希：

```text
18:6
14:5
true:false
-1:-1
false:true
true:2
false:3
true:4
```

Jarde 与 JADX 对关键括号、`|`/`&`、副作用执行次数均一致。固定 JADX 在这份 **Java 8 class** 中也将 `~int`/`~long` 输出为 `i ^ (-1)`/`j ^ (-1)`；Jarde 的 `^ -1`/`^ -1L` 与它同等语义，因此 Smali 专有 `~` 断言不构成此首片的 Jarde 差距。真正的源码质量差距是布尔异或：JADX 输出 `!z`、`!left()` 和 `left()`，Jarde 输出 `arg1 ^ true`、`this.left() ^ true` 和 `this.left() ^ false`。双方调用均恰好执行一次，且 `false` 情况不能丢掉这个调用。

Jarde 的 `build.rs::Operation::Bitwise` 已有布尔类型证据与 `ExprKind::Not`，只需在准确 `ixor`、左值由描述符/已决局部证明为 boolean、右值为准确 `0`/`1` 常量且没有未证来源时，选择已有 `Not` 或原左表达式；没有必要新增 pass/AST。整型/长整型 XOR、非常量 RHS 和不能证明布尔类型的结果仍走原有位运算。[窄 OpenSpec](../../../changes/simplify-proved-boolean-xor-literal/)记录实现和拒绝边界。EM-22 移为“部分已测”，其余优先级、溢出、浮点、Smali/Dex 位取反及混合条件还需逐项审计。
