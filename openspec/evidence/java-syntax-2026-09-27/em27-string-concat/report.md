# EM-27：内嵌 char[] 构造实参和 String 身份

固定 JADX 为 `2fb1b16386941660fda07e9017285aec40fcb37f`，Jarde 基线 CLI SHA-256 为 `8f1f0012324350e4fc65c7fef4b3e3835102e6d4fda00df280b466270c98e727`。七份相关 JADX 测试的 SHA-256 由 [replay.py](replay.py) 核对。`TestConstStringConcat`、`TestStringBuilderElimination`、`TestStringBuilderElimination4Neg`、`TestStringConcatWithoutResult` 与 `TestStringConstructor` 有实际源码断言；`TestStringConcatJava11` 另有 Java 8/J11 profile，不能用本轮 Java 8 夹具代表 J11 invokedynamic；`TestIssue13a` 明确 `disableCompilation()` 且只检查字符字面量，不作为 Java 8 全源码正向证据。`TestStringConstructor` 只检查输出含 `abc`，没有检查新对象身份。

[input/em27/](input/em27/) 用完整 Java 8 类隔离直接 `StringBuilder` 链、常量链、含局部变量的 builder 负例、字符加字符串、无使用结果的拼接，以及 `new String(new char[]{'a','b','c'})` 与先存局部数组再构造两种形态。共同 Runner 同时检查字符内容和 `== "abc"`，使对象身份变化可观测。命令和完整源码、工具版本、日志、哈希见 [baseline/summary.json](baseline/summary.json)：

```sh
python3 openspec/evidence/java-syntax-2026-09-27/em27-string-concat/replay.py \
  --jarde /tmp/jarde-cli-accepted-dt31 \
  --jadx /Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx \
  --jadx-checkout /Users/lordcasser/workspace/testzone/jadx \
  --out openspec/evidence/java-syntax-2026-09-27/em27-string-concat/baseline
```

原源码与 JADX 全类源码以 `javac --release 8 -g:none` 重编、`java -Xverify:all` 执行成功。前六行字符串结果一致；两种 `new String(char[])` 的身份检查原 class 均为 `false`，固定 JADX 均为 `true`，因为它把显式构造写成了字符串字面量。这个有运行反例的改写不能照搬。Jarde 其余方法在同轮报告中有结构化正文：直接链已写为 `+`，局部 builder 保留调用链，字符从字符串上下文开始，未使用结果的拼接仍保留赋值。Jarde 的完整源码无法重编，错误是 `explicitConstructor` 缺返回；其方法回退于 BCI 0/3/25 的外层 `new`/`dup`/返回链。`explicitStored` 则已写为 `char[] local0 = new char[]{'a','b','c'}; return new java.lang.String(local0);`。

[javap](baseline/javap-Concat.txt)显示直接形态先 `new java/lang/String; dup`，再 `newarray char`、三个 `castore`，最后 `String.<init>([C)V; areturn`。`jarde-java::build::ArrayInitializers` 已能完整证明 char[] 链且把 `Invoke` 视为允许的终端消费者；`init::sites` 在更早的阶段证明外层构造，却把数组 `newarray`/`dup`/`castore` 当作不得移动的独立效果。缺的是同次证明计划之间的精确父子交接与求值位置核对，不是新的 Java 表达式 AST；已有 `NewArray`、`New`、`Return` 足以写 `new String(new char[]{...})`。仅把内嵌字节码当纯常量并换成 `"abc"` 会改变身份。先让数组计划在外层构造站点可读，再把准确数组实参链作为该构造表达式的子范围证明，保持所有物理 BCI 和预算边界；具体设计见 [窄 OpenSpec](../../../changes/recover-proved-inline-string-char-array/proposal.md)。

本次只证实 Java 8 直接返回、常量 char[]、准确 `String([C)V` 的缺口。byte[]/charset、空构造、数组别名或突变、异常 handler、额外副作用和 J11 concat invokedynamic 尚未证明；EM-27 整项仍是部分已测且有明确差距。
