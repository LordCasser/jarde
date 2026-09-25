# 同名方法变量的错误擦除控制

从仓库根目录执行 [`negative-erasure-control.py`](negative-erasure-control.py)，传入当前 Jarde CLI：

```text
python3 openspec/evidence/java-syntax-2026-09-25/method-type-variable-shadowing/negative-erasure-control.py --jarde-cli /path/to/current/jarde-cli
```

脚本先以 `javac --release 8` 分别编译 `-g`/`-g:none` 原始 `ShadowPlain`、`ShadowBounded`、`StrongCaller`，随后**只改 `ShadowBounded.class` 常量池中的方法 `Signature` UTF8**，把方法局部 `<T extends CharSequence>` 换成 `<T extends Number>`。它同步更新该 UTF8 项的长度，不改 descriptor、Code、类 `Signature` 或调用方。`javap` 的[无调试版记录](negative-javap-g-none.txt)显示物理 `echo(CharSequence):CharSequence` 与伪造的 `<T:Ljava/lang/Number;>(TT;)TT;` 并存；类自身的 `T extends Number` 保持不变。重编 class 摘要见 [`negative-class-sha256.txt`](negative-class-sha256.txt)。

JVM 不使用这个属性来验证方法参数和返回的物理类型；patched class 在 `java -Xverify:all` 下仍由原调用方打印 [`plain`、`bounded`](negative-original-run-g-none.txt)。这使其成为 verifier-valid 的元数据负例，而非改动可观察 Java 代码路径的测试。

当前 Jarde 的 reader 优先以方法局部 `T` 查找并按该变量的 `Number` 首界擦除，再与物理 `CharSequence` descriptor 比较，在参数 0 准确报 `jvm_signature_erasure_mismatch`。[Jarde 完整类](negative-jarde-ShadowBounded-g-none.java)只把该方法保守写为物理 `CharSequence echo(CharSequence)`，保留类头原有的 `T extends Number`，没有把错属性投影成源码方法泛型。与未改 `StrongCaller` 按 Java 8 重编的[诊断](negative-jarde-javac-g-none.txt)显示强类型调用方因此不能编译；这是有意拒绝无证据的签名，不可为了“可编译”把方法错误写成 `<T extends Number> T echo(T)`。

脚本所有 class/JAR/重编目录都在 `TemporaryDirectory` 下清理；输出清单见 [`negative-control-sha256.txt`](negative-control-sha256.txt)。有无调试表的拒绝相同。这个控制只证明结构化 `Signature` 的作用域和逐位置擦除门；它不评价复杂方法正文或其它属性的完整源码恢复。
