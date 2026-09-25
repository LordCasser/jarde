# Lambda 多语句体内联正例

本例用于 `present-proved-java-structure` 5.2 的 Java 8 行为对照。所有源码均为顶层类型：`IntAction` 是 SAM 接口，`LambdaBodyInline` 建立并调用 lambda。冻结 class 只有一份，保存在 [`tests/fixtures/proved-java-structure/lambda-body-inline/`](../../../../tests/fixtures/proved-java-structure/lambda-body-inline/)；本目录只保存散列、反汇编、工具输出和 JADX 生成源码。

## 原始源码与行为

从仓库根目录冻结 class 的命令：

```sh
javac --release 8 -g -d tests/fixtures/proved-java-structure/lambda-body-inline \
  tests/fixtures/proved-java-structure/lambda-body-inline/IntAction.java \
  tests/fixtures/proved-java-structure/lambda-body-inline/LambdaBodyInline.java
```

复跑命令为 `tests/fixtures/proved-java-structure/lambda-body-inline/run.sh`。该脚本使用临时目录重新编译两份源文件，以 `java -Xverify:all` 执行，再删除临时目录。环境为 javac/java 23.0.1；JDK 23 会对 `--release 8` 发出选项过时警告，详见 `original-javac-stderr.txt`。

程序通过断言后输出：

```text
created captureCalls=1 bodyCalls=0 events=[capture]
first result=12 bodyCalls=1 events=[capture, start:10:2, end:10:2]
second result=13 bodyCalls=2 events=[capture, start:10:2, end:10:2, start:10:3, end:10:3]
```

`build(capture())` 在创建期间执行一次捕获表达式，lambda 正文此时仍为零次。正文按 `start`、增加计数、`end` 的顺序执行，返回捕获值与 SAM 参数之和；两次 SAM 调用各执行完整正文一次。事件列表、调用计数和结果均在程序内断言。

## 冻结 class 与字节码证据

所有 class 的 SHA-256 见 [`class-sha256.txt`](class-sha256.txt)。反汇编见 [`javap.txt`](javap.txt)，使用命令：

```sh
javap -classpath tests/fixtures/proved-java-structure/lambda-body-inline \
  -p -s -c LambdaBodyInline IntAction
```

`LambdaBodyInline.build(int)` 只把 `captured` 作为 `invokedynamic` 捕获参数并返回 SAM。合成方法 `lambda$build$0(int,int)` 的指令体含两次事件写入、中间一次计数器递增和最终返回，方法描述符与捕获参数及 SAM 参数吻合。接口 `IntAction.apply(int)` 的描述符为 `(I)I`。classfile major version 为 52。

## JADX 1.5.6 完整源码对照

本机工具为 [`jadx-version.txt`](jadx-version.txt) 所示的 JADX 1.5.6。以下命令把两个冻结 class 打成临时 jar，完整反编译，编译生成的全部 Java 源码并运行；构建和输出目录在脚本退出时清理：

```sh
work=$(mktemp -d /tmp/jarde-lambda-body-inline-jadx.XXXXXX)
mkdir -p "$work/input" "$work/jadx-classes"
cp tests/fixtures/proved-java-structure/lambda-body-inline/*.class "$work/input/"
jar --create --file "$work/lambda-body-inline.jar" -C "$work/input" .
jadx -d "$work/jadx" "$work/lambda-body-inline.jar"
find "$work/jadx/sources" -type f -name '*.java' -print0 |
  xargs -0 javac --release 8 -g -d "$work/jadx-classes"
java -Xverify:all -cp "$work/jadx-classes" defpackage.LambdaBodyInline
```

JADX 生成的两份完整源码保存在 [`jadx-source/`](jadx-source/)，反编译与编译日志分别在 `jadx.log`、`jadx-javac.log`。JADX 保留了箭头块中的三条有序语句，且没有在创建时执行它们。完整源码编译成功，运行结果与原始 class 完全相同，见 [`jadx-run.txt`](jadx-run.txt)。JADX 将默认包映射为 `defpackage`，因此执行类名带该前缀。

## Jarde 当前工作树对照

架构师独立复跑 `run.sh`，原始四行输出与上文一致，两个冻结 class 的 SHA-256 与记录一致。用 `CARGO_TARGET_DIR=/tmp/jarde-lambda-body-root-target cargo build -p jarde-cli --locked` 构建当前工作树 CLI，把冻结 class 打入临时 jar，分别请求 `jarde-cli class-source --input fixture.jar --class LambdaBodyInline --format json` 与 `--class IntAction`，两次退出码都为 0、`outcome=performed`、`execution.status=complete`。临时 jar/Java 源码/编译目录均已清理，私有 Cargo target 在验收后清理。

完整 Jarde 输出保存在 [`jarde-LambdaBodyInline.java`](jarde-LambdaBodyInline.java)。`build(int)` 当前文本仍是转发箭头，未把三条有序正文放进箭头块：

```java
return (int p0) -> LambdaBodyInline.lambda$build$0(captured, p0);
```

`lambda$build$0(int,int)` 本身已结构化地写出两次 `events.add(...)`、一次 `bodyCalls` 自增与 `return captured + value`。这证明正文恢复与使用点投影是两个不同边界。`main` 还留下四处分支条件转 `boolean` 的 `@bytecode` 缺口；本证据不能将整类质量称为无缺口。

用 CLI 的完整 `LambdaBodyInline.java` 文本（按正确文件名放在临时目录）和原始 `IntAction.java` 执行 `javac --release 8`，退出码 **1**；[完整编译日志](jarde-javac.log)已保存。核心错误为：

```text
符号lambda$build$0(int,int)与LambdaBodyInline中的 compiler-synthesized 符号冲突
```

即使箭头仍只是转发调用，javac 也会为该箭头生成同名私有 helper，撞上被保留的原合成方法。因此 5.2 除了证明内联正文，还须在类级源码投影中解决合成方法的 Java 源码命名冲突，且保留原物理方法身份、正文和所有仍需调用它的站点。只改箭头正文不能让此样例整类重编。
