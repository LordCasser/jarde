# 基类构造期间的匿名覆写分派与捕获值

这个独立 Java 8 样例记录一个构造顺序约束：基类构造器虚调用匿名子类的覆写方法时，捕获局部已经存入合成字段，覆写方法能读到该值。若将该捕获字段写入延迟到 `Base.<init>` 返回后，覆写方法会在构造期间读到默认值，改变程序行为。

唯一冻结的 class 副本与源码位于 [`tests/fixtures/proved-java-structure/anonymous-super-dispatch/`](../../../../tests/fixtures/proved-java-structure/anonymous-super-dispatch/)。没有修改 OpenSpec 任务或生产代码。

## 原始源码编译与执行

从仓库根目录生成冻结 class 的命令：

```sh
javac --release 8 -g -d tests/fixtures/proved-java-structure/anonymous-super-dispatch tests/fixtures/proved-java-structure/anonymous-super-dispatch/AnonymousSuperDispatch.java
```

复跑入口：

```sh
tests/fixtures/proved-java-structure/anonymous-super-dispatch/run.sh
```

环境为 `javac 23.0.1`、`java 23.0.1` 与 `jadx 1.5.6`。JDK 23 对 `--release 8` 打印选项过时警告。原始 class 在 `java -Xverify:all` 下通过，输出：

```text
observed=captured-value
visibleDuringSuper=true
```

`Base()` 先标记自己仍在构造中，再调用可覆盖的 `observe()`；基类构造器返回前，会立即读取匿名覆写写入的 `observed` 并记录可见性。结果为 `true`，表明 `captured-value` 已在该虚调用期间到达覆写方法。计数和状态均由顶层包可见字段观察，不需额外 synthetic accessor。

## 冻结 class 与 javap 证据

目录内全部 class 的 SHA-256：

| class 文件 | SHA-256 |
| --- | --- |
| `AnonymousSuperDispatch.class` | `0f964fd3f4657ac49250203d05cb555e22a72e5ecb497650c15a0e365445caf0` |
| `AnonymousSuperDispatch$1.class` | `06ae20e411c051ae4293144248259d08bc80168d9fd472b8cb4d5560eabc3886` |
| `Base.class` | `f957579bad5e74049e41d61966de5ba6fe951f053aff8cc82130607028240d2a` |

复核命令：

```sh
shasum -a 256 tests/fixtures/proved-java-structure/anonymous-super-dispatch/*.class
javap -classpath tests/fixtures/proved-java-structure/anonymous-super-dispatch -p -s -c 'AnonymousSuperDispatch' 'Base' 'AnonymousSuperDispatch$1'
```

`AnonymousSuperDispatch$1.<init>(String)` 的指令先把参数写入 `val$captured: String`，再调用 `Base.<init>()`。`Base.<init>()` 随后调用虚方法 `observe()`；匿名覆写体的字节码从 `val$captured` 读取，并写到 `AnonymousSuperDispatch.observed`。这与运行时在基类构造仍进行时观察到捕获值相互印证。

## JADX 1.5.6 对照

将上述同一组冻结 class 复制到临时目录、打成 jar，完整反编译并编译全部生成的 Java 文件：

```sh
work=$(mktemp -d /tmp/jarde-anonymous-super-dispatch.XXXXXX)
mkdir -p "$work/input"
cp tests/fixtures/proved-java-structure/anonymous-super-dispatch/*.class "$work/input/"
jar --create --file "$work/anonymous-super-dispatch.jar" -C "$work/input" .
jadx -d "$work/jadx" "$work/anonymous-super-dispatch.jar"
find "$work/jadx/sources" -type f -name '*.java' -print0 | xargs -0 javac --release 8 -g -d "$work/jadx-classes"
java -Xverify:all -cp "$work/jadx-classes" defpackage.AnonymousSuperDispatch
```

JADX 完整输出 `Base.java` 与 `AnonymousSuperDispatch.java`。前者仍在 `Base()` 中调用 `observe()` 并在返回前检查状态；后者输出 `return new Base() { ... observed = captured; };`。完整生成源用 Java 8 模式编译通过，`-Xverify:all` 输出同样的 `observed=captured-value` 与 `visibleDuringSuper=true`。JADX 为默认包补了 `defpackage` 包名，这是其拼写调整，未影响本例结果。

## Jarde 对照

架构师以 `CARGO_TARGET_DIR=/tmp/jarde-anonymous-root-target cargo build -p jarde-cli --locked` 构建当前工作树，将冻结的三个 class 打入临时 jar，逐类运行 `jarde-cli class-source --input input.jar --class 类名 --format json`。三个请求均退出 `0`、`outcome=performed`、`execution.status=complete`，但方法级正文质量不同。

调用者 `AnonymousSuperDispatch.create()` 目前仍写 `return new AnonymousSuperDispatch$1(captured);`，没有内联匿名体。只把调用者的 Jarde 输出用 `javac --release 8 -cp input.jar` 重编，运行时让新调用者 class 优先、原始 jar 提供 `Base` 和 `$1`，结果仍是 `observed=captured-value`、`visibleDuringSuper=true`。这证明当前调用者转发保持行为，不代表匿名体已经重建。

`AnonymousSuperDispatch$1` 单独输出仍把 `this.val$captured = arg1;` 放在 `super();` 前。`javac --release 8 -cp input.jar` 拒绝这份文本，诊断为构造器调用前字段赋值需要预览的灵活构造器。直接把两句换序虽能消除该语法错误，却会使 `Base()` 的虚调用先于捕获字段写入，本 fixture 的 `visibleDuringSuper` 从 `true` 变为 `false`；所以 OpenSpec 2.10 不能无条件要求换序。

另一个独立缺口在 `Base` 的 Jarde 正文：它把 `capturedVisibleBeforeBaseReturns = inBaseConstructor && "captured-value".equals(observed)` 的汇合写入留成 `@bytecode 34`，生成文本虽可编译，但单独用新 `Base.class` 优先、其余原始 class 在后的运行输出为 `visibleDuringSuper=false`。这是条件值汇合/字段写入恢复问题，不能作为 2.10 换序的修法，也不能与本构造顺序问题混成一次实现。以上每次编译都在独立临时目录，只编译所测类的一个 `.java` 文件，避免相邻源码被 javac 自动编译而污染对照。

`recover-conditional-field-writes` 1.1 的独立验收进一步收紧了缺口：当前 [Jarde Base.java](jarde-Base.java) 把两次分支、true/false 生产者、BCI 34 字段写入及同一块的 BCI 37/38/41 后缀放在**同一条可见 `@bytecode` 引用**中，source map 也保留这些 BCI；不再把 BCI 34 重访误报为循环。此阶段尚未证明 SSA Phi 和唯一消费，所以整块保守拒绝。只用冻结 `.class` 打包的原 jar 作 classpath，独立重编这份 `Base.java` 成功，但新 `Base.class` 优先运行仍输出 `visibleDuringSuper=false`（[编译日志](jarde-Base-javac.log)、[运行输出](jarde-Base-run.txt)）。这份可编译源码有明确的缺口标记，不能被当作语义等价；恢复赋值属于本 change 的 1.2/1.3。

## 条件字段写入 1.3 的独立复验

上述 `Jarde Base.java` 是 **1.1 阶段快照**。1.3 实现后，架构师重新从三个冻结 class 打 jar、用当前工作树 CLI 生成 [新的 Base.java](jarde-Base-after-1.3.java) 和 [JSON 报告](jarde-Base-after-1.3.json)。构造器中 `capturedVisibleBeforeBaseReturns` 只赋值一次，右侧是按外层、内层短路条件嵌套的 `Conditional` 值；后续 `inBaseConstructor = false` 仍在该赋值之后。新的文本没有 `@bytecode` 标记。方法报告为 `quality=structured`、`representation=java`、`fallbacks=[]`；CLI `outcome=performed` 且执行完整。聚焦测试同时校验赋值源指令包含两次分支、1/0 生产者和 BCI 34 字段写入，后缀 BCI 37/38/41 仍各自发射；CLI 的 JSON 默认未请求 `source_map` 详情，故不能用这份 JSON 代替测试的指令来源断言。

仅重编这份 Base 源码（`javac --release 8 -cp input.jar`）成功（[编译日志](jarde-Base-after-1.3-javac.log)）。运行时把新 `Base.class` 置于原 jar 前，其余 class 保持冻结原件，`java -Xverify:all` 输出 [记录](jarde-Base-after-1.3-run.log)：`observed=captured-value`、`visibleDuringSuper=true`，逐行等于原件和 JADX 的执行输出。架构师独立复跑 `jarde-java` lib 156 项、匿名侧车 4 项、条件值定向 2 项、`class_source` 47 项、CLI `class_source_cli` 16 项，格式与 OpenSpec strict validation 均通过。左侧为假时 RHS 不求值、非规范 `Z` 等额外控制尚待 2.1/2.2，不能由这个正例外推。
