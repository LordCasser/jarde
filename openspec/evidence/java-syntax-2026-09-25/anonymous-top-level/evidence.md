# 顶层类型匿名捕获正例

本证据为 `present-proved-java-structure` 任务 5.3 增加一个独立正例，未修改生产代码或任务表。源码声明了顶层接口 `Renderer`、顶层抽象基类 `Base` 和入口类 `AnonymousTopLevel`；匿名类继承 `Base`，源码层级不含 `$Renderer` 或 `$Base` 类型拼写。

冻结 class 文件唯一副本位于 [`tests/fixtures/proved-java-structure/anonymous-top-level/`](../../../../tests/fixtures/proved-java-structure/anonymous-top-level/)。临时目录只用于执行或工具对照，证据均引用此冻结副本。

## 原始源码与执行

从仓库根目录生成冻结 class 文件的命令：

```sh
javac --release 8 -g -d tests/fixtures/proved-java-structure/anonymous-top-level tests/fixtures/proved-java-structure/anonymous-top-level/AnonymousTopLevel.java
```

可复跑入口：

```sh
tests/fixtures/proved-java-structure/anonymous-top-level/run.sh
```

环境为 `javac 23.0.1`、`java 23.0.1`、`jadx 1.5.6`。JDK 23 对 `--release 8` 打印选项过时警告。原始 class 在 `java -Xverify:all` 下通过，输出：

```text
value=23:captured
events=capture,choose,base(23),render
counts=1,1,1,1
```

`create()` 先调用一次 `captureLocal()` 得到稳定局部，再求值 `choose()` 作为 `Base(long)` 的真实实参。基类构造器记录传入的值；匿名体读取 `seed()` 与捕获局部并记录一次 `render`。事件序列和计数输出都证明各动作恰好发生一次且先后次序如上。

`renderCalls` 声明为包可见。上轮 Jarde 对照发现，若它是私有字段，Java 8 会为匿名类访问它生成合成 `access$008`；该 accessor 在 Jarde 中成为 `explanation_only`，挡住整类重编。当前可见性让匿名体直接读写计数器，隔离了这个无关 accessor 呈现问题；运行时观察值与原来的私有字段版本相同。

## 冻结 class 与 javap 证据

全部冻结 class 的 SHA-256：

| class 文件 | SHA-256 |
| --- | --- |
| `AnonymousTopLevel.class` | `132207f90cbec6aab8ec7070628ffdfb2dbe5830494c314ae80ff8b70b121c43` |
| `AnonymousTopLevel$1.class` | `89db5ebc180357b82dd2824637037a1b926f022209378b9cfdc48e38c1de6d28` |
| `Base.class` | `d26aa60934ee434e5c486b635da16dd08307d1248aaae00da75a0321b1313460` |
| `Renderer.class` | `05d77405e783870842f7b70fc38dae5f2ad67879690f7fcac7fc58ba32593c12` |

复核命令：

```sh
shasum -a 256 tests/fixtures/proved-java-structure/anonymous-top-level/*.class
javap -classpath tests/fixtures/proved-java-structure/anonymous-top-level -p -s -c 'AnonymousTopLevel' 'Base' 'AnonymousTopLevel$1'
```

`AnonymousTopLevel.create()` 的指令顺序是 `captureLocal()`、存局部、分配 `AnonymousTopLevel$1`、`choose()`、装载捕获局部、调用构造器 `(JLjava/lang/String;)V`。匿名 class 声明一个 `val$captured: String` 字段；其构造器将字符串参数写入该字段，再用唯一的 `long` 参数调用 `Base.<init>(J)V`。这将真实基类实参与合成捕获清楚分开。`render()` 从基类读回 `seed()`，从捕获字段读取字符串，并直接读写包可见 `renderCalls`。class 中没有合成 `access$008`。

## JADX 1.5.6 对照

同一冻结 class 集合被复制到临时目录、打成 jar 后完整反编译：

```sh
work=$(mktemp -d /tmp/jarde-anonymous-top-level.XXXXXX)
mkdir -p "$work/input"
cp tests/fixtures/proved-java-structure/anonymous-top-level/*.class "$work/input/"
jar --create --file "$work/anonymous-top-level.jar" -C "$work/input" .
jadx -d "$work/jadx" "$work/anonymous-top-level.jar"
find "$work/jadx/sources" -type f -name '*.java' -print0 | xargs -0 javac --release 8 -g -d "$work/jadx-classes"
java -Xverify:all -cp "$work/jadx-classes" defpackage.AnonymousTopLevel
```

JADX 生成的 `AnonymousTopLevel.java` 保留 `final String captured = captureLocal();` 和 `return new Base(choose()) { ... return seed() + ":" + captured; };`，并将计数器呈现为对包可见字段的直接递增，没有 `access$008`。它另行生成 `Base.java` 与 `Renderer.java`，三份完整源码可用 Java 8 模式重编，随后 `-Xverify:all` 运行结果与原始 class 一致：`value=23:captured`、事件顺序相同，计数均为 1。JADX 给默认包类加了 `defpackage` 包名；这是其源码拼写差异，不影响此样例的编译执行结论。

## Jarde 当前对照

架构师从当前工作树以 `CARGO_TARGET_DIR=/tmp/jarde-anonymous-root-target cargo build -p jarde-cli --locked` 构建 CLI，将四个冻结 class 打入临时 jar，运行 `jarde-cli class-source --input input.jar --class AnonymousTopLevel --format json`。CLI 退出码 `0`，报告 `outcome=performed`、`execution.status=complete`。`create()` 当前为：

```java
java.lang.String captured = captureLocal();
return new AnonymousTopLevel$1(choose(), captured);
```

这保留了 `choose()` 与捕获值的顺序，也没有错误写成 `new Base(choose(), captured)`；但它仍以可见的二进制匿名类名调用合成构造器，尚未生成源码里的 `new Base(choose()) { ... }`。将这份 `AnonymousTopLevel.java` 用 `javac --release 8 -cp input.jar` 重编，退出码 `0`；以新编译的入口类优先、原始 jar 在后执行 `java -Xverify:all`，输出仍为 `value=23:captured`、事件顺序一致、四项计数都是 `1`。这里依赖原始 jar 提供 `AnonymousTopLevel$1`、`Base` 和 `Renderer`，只证明调用者源码可与原始匿名类协作，**不**证明匿名类已从反编译源码重建。

对同一 jar 的 `AnonymousTopLevel$1` 单独运行 Jarde `class-source`，构造器文本仍是 `this.val$captured = arg3; super(seed);`。按 Java 8 模式重编退出码 `1`，报构造器调用前赋值需要预览的灵活构造器。其 `render()` 正文已成结构化语句，但构造器顺序是 OpenSpec 2.10 的独立缺口；在 5.3 投影前不能把该类源码当作可编译的匿名体证明。

两次 CLI 请求、javac 与运行都在临时目录完成；冻结 class 仍只有 fixture 一份。Jarde 的两个类源码均未宣称为自包含项目，故上述编译对照明确列出所需原始 jar。
