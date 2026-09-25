# 匿名类同方法双分配点：JADX 类身份偏移

`tests/fixtures/proved-java-structure/anonymous-double-site/AnonymousDoubleSite.java` 用 `javac --release 8 -g` 编译出两个匿名类。冻结脚本只改调用者 `create(boolean)` 第二个 `new`/`invokespecial` 的常量池索引，使 BCI 4 和 12 的 `new` **都指向物理 `AnonymousDoubleSite$1`**，构造调用 BCI 8 和 16 也都指向 `$1.<init>()V`；原 `$2` class 保留但不再被分配。`javap -p -c -s AnonymousDoubleSite` 明确显示两处 `new #7`/`invokespecial #9`，不是按类名或方法名推断。`java -Xverify:all` 执行冻结 class 通过。

| 输入 | 输出 `first` / `second` / `sameClass` |
| --- | --- |
| 未变异的 Java 8 源码编译 | `11` / `22` / `false` |
| 冻结变异 class | `11` / `11` / `true` |
| JADX 1.5.6 对冻结 class 的完整源码重编 | `11` / `11` / **`false`** |

JADX 生成的 [`AnonymousDoubleSite.java`](jadx-source/AnonymousDoubleSite.java) 在两个分支里各写一次 `new DoubleBase() { int value() { return 11; } }`，并把未使用的 `$2` 留作具名 `AnonymousClass2`。生成的全部源码以 `javac --release 8 -g` 编译成功，在 `java -Xverify:all` 下运行成功，但两个匿名类花括号被编译成两个不同的运行时类，因此 `sameClass` 错误。工具与编译日志、[运行输出](jadx-run.txt)、[反汇编](javap.txt)均已保存。本地 [ProcessAnonymous.java](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/ProcessAnonymous.java) 的 `checkUsage` 检查 `ctr.getUseIn().size() == 1`，此集合按使用**方法**聚合，同方法两个 BCI 仍只算一个。Jarde 5.3 的唯一性证明必须按完整扫描中的分配 BCI 计数；此例应拒绝单点内联，保留物理 class 身份。JADX 的可编译输出不是语义通过条件。

冻结 class SHA-256：

```text
df4f898a433c7726e4623e4aaa4bae136d2c5d159f79d749e2d678ea48cc81bb  AnonymousDoubleSite$1.class
3084a63213df4b6f937093c91032fe6297031bf7a781a43d76d862c8514123a2  AnonymousDoubleSite$2.class
af511d8d8e28c8f2c04b30600f33a598a5c7e893a50622de3b3d3e17faefa812  AnonymousDoubleSite.class
7a9f1b4c0923f72b8c8490dcfb9a3614bb471a7aeae558c6d25224c23260ed56  DoubleBase.class
```

复现：运行 fixture 的 `freeze.py`；以 `jar --create --file input.jar -C tests/fixtures/proved-java-structure/anonymous-double-site .` 打包后执行 `jadx -d jadx-out input.jar`；将 `jadx-out/sources` 的全部 `.java` 用 `javac --release 8 -g` 编译，执行 `java -Xverify:all -cp <输出目录> defpackage.AnonymousDoubleSite`。中间目录用临时目录并在退出时清理。

## Jarde 当前工作树

架构师从当前工作树构建 `jarde-cli`，对同一冻结 class 运行 `class-source --class AnonymousDoubleSite --format json`；CLI 退出码 0、`outcome=performed`、`execution.status=complete`。保存的 [Jarde 类源码](jarde-AnonymousDoubleSite.java)在 `create(boolean)` 的两处分支都写 `return new AnonymousDoubleSite$1();`，因而没有像 JADX 那样发明两个不同匿名类花括号，物理目标身份在文本中一致。当前 5.3 尚未内联；`main` 仍有独立的局部值多消费者 `@bytecode` 缺口。

以**仅含冻结 `.class`** 的 jar 为 classpath，将 Jarde 完整源码按正确文件名 `AnonymousDoubleSite.java` 用 `javac --release 8` 重编，退出码 1：匿名 `$1` 的二进制名字在 Java 源码里不能解析（[编译日志](jarde-javac.log)）。因此当前结果归类为“按规格保留身份和缺口”，**没有**通过完整源码重编/执行；不能把 CLI 的分析完成状态当作 Java 源码完整性。5.3 后续若要支持可编译的多站点共享身份，需要另作源级命名设计，本项只要求拒绝错误的单点内联。
