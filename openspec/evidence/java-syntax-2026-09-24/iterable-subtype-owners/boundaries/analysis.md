# List/Collection 增强循环边界

## 输入与编译

`tests/fixtures/p3-iterable-foreach/PlatformIterableOwners.java` 冻结了两个平台正例，以及额外消费、iterator 逃逸、自定义 `TextIterable` 和仅声明同名 `iterator()` 的 `IteratorSurface` 反例。fixture runner 对平台循环实际执行空集合、null receiver、混合类型坏元素和 cast 后副作用；坏元素前的 `touches` 计数用来确认 `ClassCastException` 发生在副作用之前。

`javac 23.0.1 --release 8` 分别使用 `-g` 和 `-g:none` 编译完整 subject、runner 与两个外部接口。完整原 class 均通过 `java -Xverify:all`，两次输出相同，见 [`original/debug/run.txt`](original/debug/run.txt) 与 [`original/nodebug/run.txt`](original/nodebug/run.txt)。空字符串触发的 `continue` 也计入完整运行结果。编译产物 SHA-256 分别见 [`debug-class-hashes.txt`](debug-class-hashes.txt) 和 [`nodebug-class-hashes.txt`](nodebug-class-hashes.txt)。

两个平台调用在有无局部变量表时均保持同一 owner：`List.iterator()` 是 `java/util/List`，`Collection.iterator()` 是 `java/util/Collection`；descriptor 均为 `()Ljava/util/Iterator;`。`javap` 中 `listCast` 的 owner 调用位于 BCI 1，`hasNext` 位于 BCI 10，`next` 位于 BCI 19，随后原始 `checkcast java/lang/String` 位于 BCI 26，记录见 [`original/debug/PlatformIterableOwners.javap.txt`](original/debug/PlatformIterableOwners.javap.txt)。

## 原/JADX/Jarde 对照

JADX 1.5.6 对 debug 与 no-debug class 生成的 `listCast`、`collectionCast`、`listSkipEmpty`、额外消费者、自定义子接口和同名非 `Iterable` 形态均为显式 iterator/while。两份输出经 `javac --release 8` 重编并在 `java -Xverify:all` 下运行，输出与各自原 class 相同，分别见 [`jadx-complete/debug/rebuild/run.txt`](jadx-complete/debug/rebuild/run.txt) 与 [`jadx-complete/nodebug/rebuild/run.txt`](jadx-complete/nodebug/rebuild/run.txt)。源文件保存在各目录 `sources/` 和 `rebuild/src/`。

Jarde 的可执行覆盖在 [`tests/p3_iterable_foreach.rs`](../../../../../tests/p3_iterable_foreach.rs)：两种调试设置均要求 `listCast`、`collectionCast`、`listSkipEmpty` 投影成 `for (Object ...)` 并保留循环体 `(String)` cast；完整 recovered class 以 Java 8 重编、`-Xverify:all` 执行，stdout 与原 class 完全相同。`listConsumesTwice`、`collectionEscapes`、`userSubtype` 和 `sameNameOnly` 保留 while。测试还比较 essential/all 正文、核验 BCI 1/10/19 的来源，并确认低分析预算和预取消不会发布半成形的循环。

## 异常 handler 反例

handler 负例单独编译在 `original/handler/`，避免一个 explanation-only 方法使正例完整类无法重编。其 `Iterator.next()` 先推进索引再抛一次 `IllegalStateException`，循环只在 catch 后检查 `hasNext()`，原 class 在 `-Xverify:all` 下稳定输出 `1`，见 [`original/handler/run.txt`](original/handler/run.txt) 和 [`handler-class-hashes.txt`](handler-class-hashes.txt)。集成测试确认 Jarde 没有把它投影成增强 `for`；当前 builder 对这个跨 protected region 的局部值形态给出 explanation-only，因此它只作为拒绝边界，不计入 Jarde 可执行等价回归。该局限留在本边界记录，不扩大本次架构范围。
