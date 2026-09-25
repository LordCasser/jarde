## 1.5 计算写入验收（2026-09-24）

冻结输入 `LoopTryHandlerEntry.class` 的 SHA-256 是 `79254ba10b17846830ed420a8c13451a92630209edef9cb5d977fb6765e77f38`；root 将原 Java 源码以 `javac --release 8 -g:none` 原样重编，所得 class 与冻结输入逐字节相同。实现后的 CLI `/tmp/jarde-exception-computed-replay/jarde-cli` SHA-256 是 `c0fd756cea0b682f6e4b8dd4734e8933d82f1c2f5d547e57bbd2a311108089de`。

root 用该 CLI 独立重放 `class-source --policy single-class --release 8 --evidence source_map`。`loopTry(II)I` 完整恢复，没有 `@bytecode`：共同作用域声明 `int local2;`，循环前 `local2 = 0`，try 内 `local2 = local2 + maybeFail(arg0, arg1)`，catch 内 `local2 = -1`，循环后 `return local2`。source map 中 BCI 1、9、13、19、27 分别落到这些声明/赋值、调用、计算赋值、catch 赋值与返回使用。

原源码、冻结 JADX 1.5.6 完整类文本、Jarde 完整类文本分别以 `javac --release 8 -g:none` 编译；同一个 `Runner` 在三个类上均通过 `java -Xverify:all`，逐行输出 `normal=6`、`caught=0`。这覆盖一次正常循环和一次真实抛出的 catch 路径。

负向边界只把冻结 class 中唯一的 `iadd` opcode 改为同样可校验的 `iand`，所得 class SHA-256 `77efc628473fef8dfa9b50541d826060505777399e33a738f2935d02fb33e019`。同一 CLI 将整个相关依赖切片拒绝为 `@bytecode 0 2 6 17 20 27`，诊断为 `local 2 crosses a protected region`，没有在 catch 外写出未证的 `return local2`。本验收只勾选 1.5，不自动完成 1.2、1.3 或 2.1–2.5。
