# 匿名类捕获映射 fixture 证据

本记录只覆盖 `present-proved-java-structure` 任务 5.3 的两个判别样例，不表示 5.3 完成。源码与冻结 class 文件位于 [`tests/fixtures/proved-java-structure/anonymous-capture/`](../../../../tests/fixtures/proved-java-structure/anonymous-capture/)。没有改动任务表或生产源码。

边界：本 fixture 的 `Base` 是静态嵌套类，因此展示的是规格已承诺的 `new Base(...) { ... }`。继承非静态成员基类时，Java 源码需要 `outer.new Base(...) { ... }`，还要证明并保留外层实例。本轮范围不覆盖该形状；除非后续规格明确扩展，内联器应拒绝它，不能套用这里的 `new Base(...)` 证据。

## 输入与原始执行

源码是独立手写的 Java 8 文件 `AnonymousCaptureCases.java`。使用以下命令生成并冻结目录里的六个 class 文件：

```sh
javac --release 8 -g -d tests/fixtures/proved-java-structure/anonymous-capture tests/fixtures/proved-java-structure/anonymous-capture/AnonymousCaptureCases.java
```

运行复现命令：

```sh
tests/fixtures/proved-java-structure/anonymous-capture/run.sh
```

环境：`javac 23.0.1`、`java 23.0.1`、`jadx 1.5.6`。原始源码使用 Java 8 API/字节码目标；JDK 23 对 `--release 8` 打印选项过时警告。原始 class 在 `java -Xverify:all` 下通过，输出为：

```text
base=7:captured
events=capture,choose,base(7),render
counts=1,1,1,1
receiver=20:10
```

首例依次执行一次局部捕获表达式、一次 `choose()`、一次 `Base(long)` 和一次匿名体 `render()`。`choose()` 返回 7，匿名体读回同一次构造传入的 7 和稳定局部 `captured`。事件记录与四个独立计数都在 `receiver` 样例开始前打印。

次例以 `new Outer(10).captureOther(new Outer(20))` 建立不同接收者。匿名方法读 `other.state` 与 `Outer.this.state`，输出 `20:10`。两对象的字段类型相同、值不同，所以把 `other` 错写成词法外层接收者会改变输出。

## class 文件冻结与结构证据

以下 SHA-256 是目录内提交的全部编译产物：

| class 文件 | SHA-256 |
| --- | --- |
| `AnonymousCaptureCases.class` | `3a9edf8c6eec7e7e84c76331432c8759c1e5dd1944205c37b329723d6dffb338` |
| `AnonymousCaptureCases$1.class` | `f971b244c4d035b21a185fdd1ddd39981ad9db45d440b61309bafe5a3f40d61c` |
| `AnonymousCaptureCases$Base.class` | `7be0609da863cfdf67b1b3af54ef897809423f31e3d404756d3ca73f06fefe78` |
| `AnonymousCaptureCases$Outer.class` | `381122ded21a0e055db47332ecb0aaa6f75be679500bd42b1101fd706cc5ec9f` |
| `AnonymousCaptureCases$Outer$1.class` | `889c34dd857a56ec016b214c090437588f0c5c23a4e48cfbb539ae578bb91d02` |
| `AnonymousCaptureCases$Renderer.class` | `40450d94f919930d2f7722b5f1740e28a9b31100e78f643bffdec92ee76110a5` |

核对命令：

```sh
shasum -a 256 tests/fixtures/proved-java-structure/anonymous-capture/*.class
javap -classpath tests/fixtures/proved-java-structure/anonymous-capture -p -s -c 'AnonymousCaptureCases' 'AnonymousCaptureCases$Outer' 'AnonymousCaptureCases$1' 'AnonymousCaptureCases$Outer$1'
```

`baseArgumentAndCapture` 的字节码先调用 `captureLocal` 并存入局部，再调用 `choose`，最后调用匿名构造器 `(JLjava/lang/String;)V`。`AnonymousCaptureCases$1` 声明 `val$captured: String`；其构造器把第三个槽位存入该字段，再以 `long` 参数调用 `Base.<init>(J)V`。因此 `choose()` 的 `long` 是唯一真实基类构造实参，字符串只用于捕获字段。

`Outer.captureOther` 的字节码把 `this` 和参数 `other` 按序送入匿名构造器 `(LOuter;LOuter;)V`。`Outer$1` 分别声明 `this$0` 与 `val$other`；构造器分别从第一个与第二个参数槽存入它们，`render` 再分别读取两个字段。源码输出 `20:10` 与此值流一致。

## JADX 1.5.6 对照

将同一批冻结 class 复制到临时目录并打成输入 jar 后执行完整类反编译：

```sh
work=$(mktemp -d /tmp/jarde-anonymous-capture.XXXXXX)
mkdir -p "$work/input"
cp tests/fixtures/proved-java-structure/anonymous-capture/*.class "$work/input/"
jar --create --file "$work/anonymous-capture.jar" -C "$work/input" .
jadx -d "$work/jadx" "$work/anonymous-capture.jar"
javac --release 8 -g -d "$work/jadx-classes" "$work/jadx/sources/defpackage/AnonymousCaptureCases.java"
```

JADX 首例输出 `return new Base(choose()) { ... return seed() + ":" + captured; };`，这段保留基类实参与局部捕获的区别。第二例输出关键行 `return new Renderer(this)`，随后在匿名类里生成 `this.this$0 = this`。这是接口匿名类携带参数并把匿名对象赋给 `Outer` 字段的错误文本。

用 Java 8 源码模式编译 JADX 的完整生成文件失败，JDK 23 报两个相关错误：匿名类实现接口不能带参数；`<匿名Renderer>` 不能转换为 `Outer`。因此没有运行 JADX 产物。此处记录 JADX 偏离原始源码，不把它作为 Jarde 的目标。

## Jarde 当前对照

架构验收时用当前工作树源码执行 `CARGO_TARGET_DIR=/tmp/jarde-anonymous-root-target cargo build -p jarde-cli --locked`，再以 `jar --create --file fixture.jar -C tests/fixtures/proved-java-structure/anonymous-capture .` 把冻结 class 与同目录源码打成一个普通 jar（Jarde 只读取其中的 class），分别运行：

```sh
/tmp/jarde-anonymous-root-target/debug/jarde-cli class-source --input fixture.jar --class AnonymousCaptureCases --format text --budget output_bytes=400000
/tmp/jarde-anonymous-root-target/debug/jarde-cli class-source --input fixture.jar --class 'AnonymousCaptureCases$Outer' --format text --budget output_bytes=400000
```

两次命令均退出 0。`baseArgumentAndCapture` 的方法体保留 `captureLocal()` 产生的局部，使用点写 `new AnonymousCaptureCases$1(choose(), captured)`；`Outer.captureOther` 写 `new AnonymousCaptureCases$Outer$1(this, other)`。它们明确还是独立的二进制匿名类，没有把合成构造参数误称为 Java 匿名类源级实参，也没有恢复花括号里的方法体。

将两份输出分别以外层类名保存为 `.java`，以原 fixture jar 作为 classpath 用 `javac --release 8 -Xlint:-options` 编译，都失败。主类源码的首批错误是找不到用二进制名拼写的嵌套 `Renderer` / `Outer` 类型；`Outer` 源码的直接错误是 `new AnonymousCaptureCases$Outer$1(this, other)` 的源级构造器无参，无法接受两个合成捕获实参。因此这两份 class-source 目前只是有标记的物理类呈现，不能执行其重编译产物。这里的 Jarde 失败和 JADX 的 `new Renderer(this) { ... }` 失败形状不同；两者都不能替代原 class 的执行语义。
