# OnlyNarrow

这是 overload 边界审计的独立无 int 重载输入，不属于 numeric-comparison fixture。

原 class 由 `original-src/OnlyNarrow.java` 用 `javac --release 8 -g:none` 编译，
`java -Xverify:all` 输出：

```text
runByte=1
runShort=3
```

当前 jarde 文本中的调用是 `onlyByte(3)` 与 `onlyShort(3)`。由于 class 没有 int 重载，
真实 javac 失败并报告两处 narrowing loss：从 int 转换到 byte、从 int 转换到 short；没有
伪造恢复执行结果。JADX 1.5.6 只删除 `package defpackage;` 后可编译，`-Xverify:all` 执行
仍为 `runByte=1`、`runShort=3`。

完整源码、class、`javap`、jarde/JADX 文本、编译日志和运行日志都在本目录；临时重放目录为
`/tmp/jarde-overload-boundaries/cases-v1/OnlyNarrow/`。
