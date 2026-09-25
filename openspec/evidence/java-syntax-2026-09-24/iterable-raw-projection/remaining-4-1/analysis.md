# Iterable 4.1 剩余边界探针

所有输入由 `javac 23.0.1 --release 8` 编译，class major 均为 52；分别保留 `-g` 与 `-g:none` 版本。JADX 为 1.5.6。独立 Cargo target `/tmp/jarde-iterable-remaining-target` 构建的当前 Jarde CLI SHA-256 见 [`cli.sha256.txt`](jarde/cli.sha256.txt)。各 subject 的 SHA-256 见 [`class-hashes.debug.txt`](original/class-hashes.debug.txt) 与 [`class-hashes.nodebug.txt`](original/class-hashes.nodebug.txt)；原始源码、runner、class、两方完整输出源码、javac 错误及运行记录均保留在本目录。

## 结果

`RawIterableIterator` 的参数是 raw `Iterable`，循环手写 `iterator()/hasNext()/next()`，保存 `Object item` 后显式 `(String) item`。另两个方法复用同一次 `next()` 结果：先转 `String` 后计副作用，或先计副作用再转；第四个方法先计副作用再调用 `next()`。原 class、JADX 和 Jarde 的 `-g` 源码都可用 Java 8 重编，`-Xverify:all` 运行结果相同：`raw=3`、两种正常路径均为 `6,touches=2`、错误元素两路径分别为 `ClassCastException,touches=0/1`、抛错的 `next()` 为 `IllegalStateException,touches=1`。Jarde 与 JADX 均保留普通 `while`，没有增强 `for`。

无调试表版本里，JADX 仍生成可编译的 while，但在“先 cast、再递增 touches”的方法里把 `touches++` 提到了 `(String) next` 之前。坏元素输入因此变成 `ClassCastException,touches=1`，与原 class 的 `touches=0` 不同；Jarde 的 `-g:none` 源仍保持原顺序，运行相同。两种路径说明同一 `next()` 值的多重消费和 cast 位置必须纳入证明，不能只认 iterator 外形。

`touchBeforeNext` 的循环体先做 `touches++` 再调用 `next()`。runner 提供一个 `next()` 会抛 `IllegalStateException` 的合法 iterator；原 class、JADX、Jarde 均打印 `IllegalStateException,touches=1`。两方都保留 while。把 `next()` 移入增强 `for` 头部会先调用 next，副作用数将变为 0，因此此形状必须拒绝投影。

`CatchNextScope.consume` 把 `hasNext()` 放在 try 外，但把 `next()` 放在捕获 `IllegalStateException` 的 try 内；其 [`ExceptionTable`](original/CatchNextScope.debug.javap.txt) 精确覆盖 BCI 18–39，包含 BCI 19 的 `Iterator.next()`，不覆盖测试调用。原 class 对“next 抛一次”的 runner 输出 `nextHandler=caught:1`。JADX `-g` 输出 `for (Object item : values)`，把隐式 next 移到 try 外；重编后输出 `nextHandler=escaped:IllegalStateException`，行为改变。`-g:none` 时 JADX 保留 while，行为相同。Jarde 两种输入都对该方法给 protected-region explanation-only 回退，生成文本因缺返回语句无法重编；这是保守拒绝，没有 Jarde 运行结果可比较。

`NamedIteratorCursor` 含一个 `iterator()` 方法和 `Iterator<String>` 字段，但 classfile `interfaces: 0`，并不实现 `Iterable`；`sum` 手写地调用该方法后遍历 iterator。调用 owner 是 `NamedIteratorCursor.iterator()`，不是 `java/lang/Iterable.iterator()`。JADX 与 Jarde 对 `-g`、`-g:none` 都输出 while；四份源码均重编、通过 `-Xverify:all`，输出 `nonIterable=4`。本探针没有触发 JADX 对非 `Iterable` 的误投影，但证明方法名相同本身不能当作容器源类型证据。

## 最小候选与拒绝边界

目前证据只支持把“容器源码类型明确为 `Iterable`、classfile 调用点直接是 `java/lang/Iterable.iterator()`”作为后续最小候选入口；raw `Iterable` 可合法写成 `for (Object item : input)`，并在循环体原位置保留显式 cast。此处只证明这种 Java 源写法可表达，不表示 Jarde 已实现投影。Jarde 在上述 raw 手写循环仍输出 while。

next 结果若有额外消费者、cast 与其它副作用的次序不同、`next()` 先于或后于原循环体效果、或 next 与测试调用落在不同异常处理集合，均须按原位置保留普通循环。尤其，`for (String value : input)` 会把 cast 放在 body 前；`for (Object item : input)` 也会把 `next()` 移到 body 前，只有转换位置与效果顺序确实不变时才可能等价。JADX 的 debug 反例显示保护区边界若漏证会直接改异常路径。

Jarde 的 exact-owner 第一切片不应悄悄扩成任意 `iterator()` 调用者。外部 `List<String>`／`Collection<String>` 增强 for 的编译字节码分别调用 `java/util/List.iterator`／`java/util/Collection.iterator`（该对照由 root 独立取得）；把它们纳入需要显式、可定位的源类型继承证明。非 `Iterable` 的 `NamedIteratorCursor.iterator()` 必须拒绝。raw cast、List/Collection、catch scope 和其它 `next()` 消费形态尚未形成一个全部通过的准入切片，因此 OpenSpec 4.1 继续未勾选。

## 复现

在仓库根目录使用本目录的 `original/*.java` 与 `runners/*.java`：

```sh
javac --release 8 -g -d /tmp/original <subject.java> <runner.java>
jadx -d /tmp/jadx /tmp/original/<Subject>.class
cargo build -p jarde-cli --target-dir /tmp/jarde-iterable-remaining-target
/tmp/jarde-iterable-remaining-target/debug/jarde-cli class-source \
  --input /tmp/original/<Subject>.class --class <Subject> \
  --policy single-class --release 8 --format text --output /tmp/<Subject>.jarde.java
javac --release 8 -g:none -d /tmp/rebuilt <complete-source.java> <runner.java>
java -Xverify:all -cp /tmp/rebuilt <Runner>
```

JADX 输出位于 `defpackage`，编译时使用 `runners/*.jadx.java`。所有运行结果及 Jarde explanation-only 编译诊断都已保存；Jarde 的 debug/no-debug raw 与 cursor outputs 都能完整编译执行，catch scope 则按上文所述无法执行。
