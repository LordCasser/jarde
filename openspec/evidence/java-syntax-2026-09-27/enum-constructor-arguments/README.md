# DT-11：枚举构造实参的证据边界

该目录冻结了一个 Java 8 原始 enum class、当前 Jarde 和固定 JADX 的完整类源码，以及重编译/运行对照。fixture 编译目标为 class-file major 52；证据不是对任意反编译源码可编译性的承诺，而是对这三个输入的逐项结果。

从仓库根目录重放：

```sh
python3 openspec/evidence/java-syntax-2026-09-27/enum-constructor-arguments/run_audit.py
```

脚本只清理并重建自己的 `generated/`，fixture 和 reference 输入不动。Cargo 的 `CARGO_TARGET_DIR` 位于 `TemporaryDirectory`，成功或失败均自动清理。JADX 默认取 `/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx`，也可用 `JADX` 指定 CLI；`JADX_CHECKOUT` 指向其 source checkout；可用 `JARDE_CLI` 指定另一个 Jarde binary，否则从当前仓库临时构建 CLI。Jarde 对 `IntArgs` 使用包含 `Ints.class` 的临时 plain-JAR classpath，满足外部字段 selected-environment 解析要求。运行环境与 class/source 哈希记录在 `generated/tool-versions.txt`、`generated/sha256.txt` 和 `generated/summary.json`。

输入 Java 文件位于本目录根部；`reference/` 是固定 JADX checkout 中两个 JUnit 测试的原文副本。`generated/javap-*.log`、`generated/jadx/<Enum>.java`、`generated/jarde-<Enum>.java` 和各阶段 `.log` 保存完整 enum 输出、类文件事实及编译/运行记录。Class files、javac class output 和 JADX 工作目录都只存在于自动清理的临时目录；`summary.json` 保存各阶段退出状态。状态码 `125` 表示运行未尝试，因为上一阶段 javac 已失败；它不是 JVM 的 exit code。

## 结果

| Input | 原始 class | JADX 1.5.6 | Jarde 当前窄切片 |
| --- | --- | --- | --- |
| `LiteralOnly`：`FIRST(7), NEXT(-2)` | Java 8 编译、`-Xverify:all` runner 通过 | 完整源码编译、runner 通过 | 有效 enum constants；完整源码重编、runner 通过 |
| `IntArgs`：`FIELD(Ints.THREE)`、`EXPR(Ints.THREE + 1)` | 编译、runner 通过 | 保留两个实参表达式；重编、runner 通过 | 通过同次 Code 证书和 selected-environment 字段表证明；完整源码 `--release 8` 重编、`-Xverify:all` runner 通过 |
| `StringVarargs`：字符串元素与 varargs | 编译、runner 通过 | 源码编译、runner 通过；`EMPTY` 输出为 `new String[0]` | constructor/参数形状未获证明，回退普通字段和 BCI markers；javac 拒绝 |

Jarde 对三个 class-source 命令均返回 0。StringVarargs 仍产出普通 `ACC_ENUM` 字段和 BCI markers，完整文本重编时 javac 指向 `public static final StringVarargs PAIR;` 并报告 `enum constant expected here`；运行未尝试，状态 `125` 只表示前一阶段 javac 失败。IntArgs 的 class-source 输出保留 enum 常量列表中的静态字段读和加法，并通过完整源码重编与 `-Xverify:all`。命令日志与状态码位于 `generated/jarde-*.log`、`generated/jarde-javac-*.log` 与 `generated/summary.json`。

## 字节码形状

`generated/javap-IntArgs.log` 是含 `-v -p -c` 的完整输出。`<clinit>` 的构造调用顺序是：

- `LITERAL`：BCI `0 new; 3 dup; 4 ldc "LITERAL"; 6 iconst_0` (ordinal); `7 iconst_1`; `8 invokespecial (String,int,int)V`; `11 putstatic LITERAL`。
- `FIELD`：BCI `14 new; 17 dup; 18 ldc "FIELD"; 20 iconst_1`; `21 getstatic Ints.THREE:I`; `24 invokespecial (String,int,int)V`; `27 putstatic FIELD`。
- `EXPR`：BCI `30 new; 33 dup; 34 ldc "EXPR"; 36 iconst_2`; `37 getstatic Ints.THREE:I; 40 iconst_1; 41 iadd`; `42 invokespecial (String,int,int)V`; `45 putstatic EXPR`。

`generated/javap-StringVarargs.log` 记录物理 constructor descriptor `(Ljava/lang/String;I[Ljava/lang/String;)V`。PAIR 在 invokespecial 前有 `iconst_2; anewarray String; dup/index/ldc/aastore` 两个元素写入，调用位于 BCI 21；SINGLE 是一个元素；EMPTY 在 BCI 57 创建长度 0 的 String array，再于 BCI 60 把 array 传入构造器。源级的无参 `EMPTY` 并不是 class file 中的零参数 constructor 调用。

JADX 的 `IntArgs.java` 直接得到 `LITERAL(1), FIELD(Ints.THREE), EXPR(Ints.THREE + 1)`。它把 ConstructorInsn 里的参数继续作为 enum argument；在外部寄存器场景，`EnumVisitor.inlineExternalRegs` 只有在来源是同类 enum constructor 且用途受限时才把寄存器替换为字段读。`StringVarargs.java` 得到 `PAIR(".dex", ".class")`、`SINGLE(".xml")` 和 `EMPTY(new String[0])`，保留行为但没有消除空 varargs 的冗余写法。

## 对照的 JADX 测试断言

本次固定的 `TestEnums3.java` 包含 `ONE(1), TWO(2), THREE(three), FOUR(three + 1)`。活动断言只检查 `ONE(1)` 和 `Numbers(int n)`。文件里的这两行都以 `//` 注释，不能算通过的测试：

```java
// assertThat(code, containsOne("THREE(three)"));
// assertThat(code, containsOne("assertTrue(Numbers.ONE.getNum() == 1);"));
```

后者检查外层 `check` 方法，不是 enum 参数，但同样是关闭状态。`TestEnums4.java` 活动断言检查 `CODE(".dex", ".class"),` 与 `ResType(String... extensions) {`，没有对 `UNKNOWN` 的参数形状作断言。本 fixture 复现相同的静态 int 读/加法与 String varargs 构造形状；它不把 JUnit test class 的外层嵌套类布局伪装成相同 class file。

## 当前证书和差距

`src/enum_constants.rs` 的 class 级普通 enum 证明仍然是完整组证明：class Java 8 header，完整字段/方法表，常量字段 flags 与顺序，构造器，`<clinit>` 常量前缀，`$VALUES` 工厂、`values()`、`valueOf()`，以及隐藏引用的 use census 共同通过后才进入 `ProvedEnumConstantGroup::Ordinary`。int source argument 现在是闭合表达式树，可包含原始 literal、一次 `getstatic:I`、或该读取加一个 literal。完整、无 handler 的同次 raw Code/BCI 证书权威证明 initializer prefix；generic AST 不能表示交错 `new; dup; getstatic; invokespecial` 时，其 method fallback 与 diagnostics 仍在物理 `<clinit>` 报告中保留。静态字段还必须在 selected environment 唯一解析为 Java 8 可写的顶级 owner 和完整字段/类属性，并证明同包、可访问、`static !final I`；表达式使用同包短类型名，若 enum member type 遮蔽该名则整组拒绝。自身 enum、跨包、`$` binary owner、缺失/重复字段、member owner class attributes 和停止读取均拒绝。

GETSTATIC int 与 GETSTATIC + literal / IADD 的冻结差距已由该闭合证书修复，且仍作为一个完整常量组发布。String varargs 是独立形状：constructor descriptor、hidden enum source Signature 尾参、new String array 的 literal 元素证明和 source argument 发射都尚未建模。当前首要拒绝是 enum constructor descriptor 只允许 int-tail 或 no-arg-tail。Jarde varargs 文本另有 Signature 擦除拒绝 marker；不能把它误报为 JUnit 或运行时失败。

第一实现切片只涵盖有界 int 参数 grammar：literal、一个 int `getstatic`，以及同一字段读与一个 int literal 相加的直接字节码形状；只接受实际操作数顺序，不做交换律改写。无方法调用、存储、分支、其他算术或部分数组/常量发布。必须保留 class-source 同 run 的物理索引、完整常量表顺序、constructor call 到 putstatic 的绑定、`$VALUES`/标准方法证明、允许的 terminal return 或既有静态赋值 suffix 证明和 use census。任何一项无法证明都拒绝整个常量组。读取 static 字段仍可能触发类初始化；“纯”指表达式 grammar 没有额外 call/store/control-flow，不可消除或移动 getstatic，也不得声称它无可观察效果。

String... 是未来独立切片：本目录仅留反例和现状输出，不应成为当前 change 的实现范围。它要另行证明变长数组的每次写入、空数组语义、varargs source Signature 和 constructor 绑定。当前架构可复用 enum class-source 的同 run 证书边界，不需要新顶层分析机制。
