# AssertCore 条件值最小对照（Java 8）

本目录冻结一个常规 Java 8 断言类。`AssertCore.java` 由 javac 23.0.1 的 `javac --release 8 -g -d v8 AssertCore.java` 生成；`AssertRunner.java` 是外置 source-only runner，不交给反编译器，在测试时临时编译。JVM 对照使用 `-Xverify:all`。原 class 大小 1187 B，SHA-256 `71753fbec0b66a6511bbb2b66855c50589060b729bc963cb6cd78377b75f4aaf`；受控重编 runner class 应为 1007 B，SHA-256 `b821b8f12c8b4bdddeb36271dc283f37a6f0cc016499cc6ed1b67050afc8bb66`。冻结 subject class 位于 `v8/`。

`JadxAssertCore.java` 和 `JardeAssertCore.java` 保存两种工具的完整类正文；同目录的 `*-wrong-owner.java` 保存唯一把 `<clinit>` 的 `AssertCore.class` 来源改成 `StringBuilder.class` 后的完整正文。JADX 原类和修补类都可编译；Jarde 原类和修补类均只因 `<clinit>` 的 final `Z` 字段缺少赋值而编译失败。普通 `guard`、`detail`、`check`、`counts` 方法均已恢复。原 class 的 `-ea`/`-da` 行为：

```text
-ea: 1|0;bad|2|1
-da: 0|0;0|0
```

JADX 的两个完整类分别以相同 flags 编译后输出与原 class 相同的两行。Jarde 文本未编译成功，故不声称有可执行对照。`AssertCore-wrong-owner.class` 是合法 classfile 补丁：仅将 `<clinit>` 的 BCI 0 单条 `ldc` 从 `AssertCore` 改为 `StringBuilder`，SHA-256 `af3c1190da44c938138da327a6232220acd3b8e8161808e2d9d517f11a4e71dc`。用 `-ea:AssertCore -da:java.lang.StringBuilder` 执行，原 class 输出 `1|0;bad|2|1`，错误来源 class 输出 `0|0;0|0`；两者由不同类的断言状态决定。

目录文件：原始 `AssertCore.java`、source-only `AssertRunner.java`、`v8/AssertCore.class`、错误来源补丁 class，以及原/修补的 JADX、Jarde 完整类源码。二进制及正文均来自 `openspec/evidence/java-syntax-2026-09-24/assert-core/`；没有复制临时重放日志或 summary。原字节码中普通方法正确消费值；`<clinit>` 的 `desiredAssertionStatus()` 两臂在 BCI 13 汇合后写 `putstatic $assertionsDisabled:Z`，故这一点失败属于条件值恢复边界，来源 class 必须保留。

`AssertCore-non01-arms.class` 是另一个仅改 `<clinit>` 两条单字节常量指令的有效负例。`patch_non01_arms.py` 对原 SHA 和唯一指令片段做断言，将 BCI 8 的 `iconst_1` 改为 `iconst_2`、BCI 12 的 `iconst_0` 改为 `iconst_3`；其余 Code、控制流、字段 descriptor 均未变。补丁 class SHA-256 为 `b52d39dd6ae7943d70b510c4925f23faadcc2fcc64c5cb7a2c309ae450de4e2c`，`javap -c -p` 可直接核对两个新值。将该 class 复制为临时目录的 `AssertCore.class`，从 `AssertRunner.java` 临时编译 runner 后执行，`java -Xverify:all -ea` 输出 `0|0;0|0`，`-da` 输出 `1|0;bad|2|1`，与原类正好相反。JVM 可接受这两个整数写入 `Z`，但 Java 条件表达式不能把字面量 `2`/`3` 当作布尔分支；后续实现只能在有精确 `0`/`1` 证据时做该布尔呈现，不能从 `putstatic Z` 猜测。

对这一补丁另以 JADX 1.5.6 `jadx --no-res -d <out> AssertCore-non01-arms.class` 生成完整类，原样冻结为 `JadxAssertCore-non01-arms.java`。其 `<clinit>` 写成 `$assertionsDisabled = !AssertCore.class.desiredAssertionStatus() ? 2 : 3;`，`javac --release 8` 报 `int无法转换为boolean`，因此该补丁只有原 class 可执行，不能把 JADX 或 Jarde 的未编译文本当作行为 oracle。这个负例只约束整数 Phi 的保守准入，不要求本 change 将任意 JVM `Z` 写入转换为 Java 源。
