# 直线 try/finally 中清理调用抛错的完成顺序

`FinallyStraightThrow.java` 是源级 `try { return value(); } finally { cleanup(); }`，两次调用各自改变 `trace`，并能分别抛出预先保存的不同异常对象。source-only runner 穷举正常返回、try 抛错、cleanup 覆盖返回、cleanup 覆盖 try 异常四条路径；比较异常对象身份与副作用次数。`javac 23.0.1 --release 8 -g:none` 生成的 Java 8 完整 subject 为 786 B，SHA-256 `ce79125f7752333349583e6b1ca4a0297143b6b62bb002467db8934da314bdf9`。源和 runner 的 SHA-256 分别为 `b2b47a54bc734e4712ecb2893c40babde5ddd3efef73c078d0033e49c883855a`、`1650a38661843da8af40e8d2156ba62457f3b9aee9c7c703f37bd9a7d8830fc6`。

原 class、JADX 1.5.6 和 Jarde CLI SHA-256 `8d93cf642dbca5de2a625677c3e13a44a989b2d5824ed43058d5724927744faf` 输出的**完整且未修改** Java 类均通过 `javac --release 8` 与 `java -Xverify:all`；三者逐行相同：

```text
false:false:return=41:trace=12
true:false:try=true:cleanup=false:trace=12
false:true:try=false:cleanup=true:trace=12
true:true:try=false:cleanup=true:trace=12
```

Jarde 的 `run()` 在 try 内先保存 `value()` 结果，再 `return` 保存值；单个 `finally` 调用 `cleanup()`。这一次跨四条完成路径的对照证明直线样本的当前呈现保持清理调用抛错时的覆盖优先级。`ImplicitCleanup.run()` 的受保护正文含条件分支，当前门槛仍拒绝它；本样本不替代该边界的结构证明。原 class、`javap`、三方源码和编译/运行日志在本目录，`summary.json` 保存本次重放状态。JADX 自动加入的 `package defpackage;` 仅在将其放回默认包与 source-only runner 重编时删除。
