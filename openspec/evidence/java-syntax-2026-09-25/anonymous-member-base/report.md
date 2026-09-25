# 非静态成员基类匿名类的限定接收者边界

## 输入与复现

手写的 Java 8 输入位于 `tests/fixtures/proved-java-structure/anonymous-member-base/AnonymousMemberBase.java`，同时包含 `outer.new Base(sideEffect()) { ... }` 和限定接收者为 null 的路径。`javac --release 8 -g` 生成的五个 class 唯一冻结在该 fixture 目录；本 evidence 目录不重复保存 class。`class-sha256.txt` 记录各 class 的 SHA-256，`javap-all.txt` 保存全部五个 class 的 `javap -c -p -s -v` 输出。`run.sh` 在临时目录编译并以 `-Xverify:all` 执行，退出时调用 Python `shutil.rmtree` 清理目录。

原 class 的运行输出：

```text
normal=8;events=outer,argument,base(7),anonymous
null=nullOuter
```

正常路径的事件顺序区分外层表达式求值、真实的 `Base(int)` 实参求值、基类构造和匿名覆盖方法调用。null 路径没有调用 `sideEffect()`。在 `javap-all.txt` 中，两处分配都先对限定的 `Outer` 调用 `Objects.requireNonNull`，之后才调用 `sideEffect()`。匿名构造器描述符是 `(LAnonymousMemberBase$Outer;I)V`，成员基类构造器描述符也是 `(LAnonymousMemberBase$Outer;I)V`。首个 `Outer` 是编译器注入的外层实例；源码中唯一普通实参是 `sideEffect()` 返回的 `int`。不能把构造器描述符里的 `Outer` 当成源码 `Base(...)` 的普通实参。

## JADX 1.5.6 对照

JADX 1.5.6 对同一组五个冻结 class 进行了完整反编译，完整输出保存在 `jadx-source/AnonymousMemberBase.java`，运行日志在 `jadx.log`。输出在第一处分配前保留了 `Objects.requireNonNull(outer)`；但第二处分配被写成 `new 2(outer, sideEffect())`，不是可编译的 Java。`jadx-javac.exit` 为 `1`，诊断信息见 `jadx-javac.log`。由于完整反编译源码无法编译，没有执行 JADX 产物进行运行对照。这份输出仅作为诊断证据，不能作为源码级正确性的证明。

## Jarde 当前对照

用仓库根目录的 `CARGO_TARGET_DIR=/tmp/jarde-anonymous-root-target cargo build -p jarde-cli --locked` 构建当前工作树，随后将 fixture 中的五个冻结 class 打入临时 jar，运行 `jarde-cli class-source --input input.jar --class AnonymousMemberBase --format json`。命令退出码为 `0`，报告的 `outcome=performed`、`execution.status=complete`；这只说明请求执行完毕，不代表生成文本可编译。

Jarde 对 `normal()` 和 `nullPath()` 均保留 `AnonymousMemberBase$Outer outer = ...()`，后续分配、复制、`Objects.requireNonNull` 与构造调用未被证明为一个 Java 构造表达式，按 BCI 输出 `@bytecode` 说明，未伪造 `outer.new Base(sideEffect())`，也没有把合成外层实例印成普通 `Base` 实参。对生成的 `AnonymousMemberBase.java` 使用 `javac --release 8 -cp input.jar` 重编退出码为 `1`：诊断首先指出 `AnonymousMemberBase$Outer` 与 `AnonymousMemberBase$Outer$Base` 源级名字无法解析。该失败同时包含既有嵌套类型命名问题，不能单独归因于匿名类投影；上述两个方法本身仍有正文缺口。

## 范围与限制

此 fixture 固定了 Java 8 的字节码与运行时序，说明成员基类构造器描述符中出现的 `Outer` 是合成外层实例状态，而不是源码 `Base` 实参。源码发射必须保留限定创建及其空值检查位置，或拒绝内联。本 fixture 的完整类文本尚不可编译，不能用它声称匿名类内联已实现。测试中的 CLI 与 jar 均在临时目录构建/打包，冻结 class 始终只保存在 fixture 目录。
