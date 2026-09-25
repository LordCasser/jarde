# JADX `Iterable` 增强 `for` 的调试表边界

## 独立 Java 8 三方重放

[主体源码](IterableForEach.java)将本地 JADX `jadx-core/src/test/java/jadx/tests/integration/loops/TestIterableForEach.java` 的 `TestCls.test` 方法体独立出来，配以[四行 runner](IterableForEachRunner.java)。`javac 23.0.1 --release 8 -g` 生成 major 52 的主体 class，SHA-256 `a2d976648695bf287b23d7004ecd376cb650b9da7e2afb3a00fae783689bd746`；runner 为 `018b2359136e15b3720a5544c2006f77625a276df556b58d2c13d0435ff64aed`。源码、class 与三份恢复文本都留在本目录。

安装的 JADX 1.5.6 对该主体输出 `for (String s : a)`。从当前工作树重建的 Jarde CLI SHA-256 为 `30f526e176ee526707ee6c89fb48ec2b0e35ab5e1c453aa5714231a5965a857f`，输出显式 iterator 局部、`while (local3.hasNext())` 与体内 `String s = (String) local3.next()`。Jarde 现有泛型方法签名规则因正文并非“直接返回参数”而拒绝泛型头投影，原始 `Iterable` 方法头与保留的 cast 仍可编译。

原源码、JADX 类源码加包名匹配 runner、Jarde 类源码加 runner 均通过 `javac --release 8 -g:none` 和 `java -Xverify:all`，四行逐字相同：

```text
multi=abc
empty=
nullElement=anullb
nullContainer=NullPointerException
```

三者语法分别是原 `for`／JADX `for`／Jarde `while`。对旧冻结 `StringIterableForeach.class`（SHA-256 `81a0dcb4a53a51eb721df72f2b30825b33e8fb4dc6f239e0e93167f4012e09f36`）再次用当前 JADX/Jarde 运行，两者在 `sumLengths` 仍输出 `while`。旧 class 同是 major 52，带 `Iterable<String>` 的方法 `Signature`，但没有 `LocalVariableTable`。

## 哪一项元数据改变了 JADX 输出

同一主体源码分别以 `--release 8 -g` 与 `--release 8 -g:none` 编译。前者 SHA-256 `a2d976648695bf287b23d7004ecd376cb650b9da7e2afb3a00fae783689bd746`，后者 `74b06183647c0202d355ba5530fa628256c60541fc81205d99bc425b1c61b21a`；major 都为 52，`javap -c -p` 的方法指令相同，差异在调试版包含 `LocalVariableTable` 和 `LocalVariableTypeTable`。JADX 对调试版输出增强 `for`，对无调试版输出 iterator 加 `while`。

两个控制组复现同一切换：忠实保留嵌套 `TestCls` 的源码在有/无调试表时，JADX 也分别输出增强 `for`/`while`（class SHA `6420d2383f972d172b693f342ce448f1c53a01496b67c816bc22a8574e77f921`、`7bb6aba6c57edf320f30244f03ff3cb23461016be9e3d784436f417ed82e60b1`）；同体 `sumLengths(Iterable<String>)` 控制组亦然（调试版 SHA `d0da96703212fce1e2008733d46f83c1894dde83c1fe31126bd95594a06214a1`，无调试版 `bd069e94b2841bcd85cb5a16c6f4b5f08919e41b243a16707e9b17415c3411d9`）。对应源码、JADX 输出和 `javap` 在 [debug-control](debug-control/)。

调试局部变量元数据是这组 Java 8 样本中最强的已证实区分因素；原 JADX JUnit 测试实际生成的 class 未在工作树，故不推断它的精确属性。旧 `sumLengths` 的 `-g:none` 冻结输入也符合这一观察。测试源码写过增强 `for` 不能证明字节码保留了作者唯一的语法；本项目应以完整 SSA/类型/作用域证明决定投影，而非把调试表当硬前置。

这份证据只覆盖参数化 `Iterable<String>` 的一个方法形状。OpenSpec 4.1 仍须补 raw `Iterable` 保留显式 cast、非 `Iterable.iterator()` 及其它 `next()` 转换边界，不因这一正例提前勾选。

## 复现

从仓库根目录执行以下主体编译与恢复；各版本完整源码及运行 stdout 留在本目录。

```sh
D=/tmp/jarde-iterable-jadx-claim
javac --release 8 -g -d "$D/classes" \
  openspec/evidence/java-syntax-2026-09-24/iterable-raw-projection/jadx-iterable-for-claim/IterableForEach.java \
  openspec/evidence/java-syntax-2026-09-24/iterable-raw-projection/jadx-iterable-for-claim/IterableForEachRunner.java
jadx -d "$D/jadx" "$D/classes/IterableForEach.class"
cargo build -p jarde-cli --target-dir /tmp/jarde-iterable-jadx-claim-cargo-target
/tmp/jarde-iterable-jadx-claim-cargo-target/debug/jarde-cli class-source \
  --input "$D/classes/IterableForEach.class" --class IterableForEach \
  --policy single-class --release 8 --format text --output "$D/IterableForEach.jarde.java"
```
