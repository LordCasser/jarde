# 普通参数化方法 Signature 的源码投影证据

## 范围与结论

普通参数化方法 `Signature` 可从 `Iterable<String>`、`List<? extends Number>`、嵌套 `Map<String,List<Integer>>` 和 `List<String>[]` 中读出，且各参数/返回擦除都与 JVM descriptor 一致。当前 reader 的 grammar 和逐位置擦除证明可直接复用。`recover-ordinary-parameterized-signatures` 的当前 class-source 投影已在受限身份形状上为四种声明写出参数化头；raw 对照没有 `Signature`，仍写 raw descriptor。

独立重放表明原 `-g` / `-g:none` class、JADX 1.5.6 完整源码重编结果、Jarde 完整源码重编结果，均由 `java -Xverify:all` 执行；反射的 generic parameter/return 输出和嵌套调用结果 `7` 一致。Jarde 在这五个小方法上的单类源码、调用方编译和运行闭环已通过。冻结的复现脚本记录 Jarde 输出时的源码 hash，所以后续改实现需要重跑并更新 hash。

额外边界见 [`boundaries.md`](boundaries.md)。重点是 verifier 接受的等长 `List`→`Tree` 替换：直接 `List` 参数/返回（numbers、arrays）因逐位置擦除不符而被 Jarde 拒绝，但嵌套 `Map<String,List<Integer>>` 的最外层擦除仍为 Map，Jarde 会投影不存在的 `java.util.Tree`，其完整源码无法编译。逐位置擦除证明只约束该位置的直接擦除，不等于递归校验所有嵌套类名；嵌套 source-reference resolution 是明确剩余债务。JADX 同样会打印错误的 Tree。无法呈现的 class-level `T`、有 `type_path` 的类型注解和双重嵌套 Entry 名称分别保留 descriptor 并报告拒绝。JADX 会恢复前两类中的普通参数化类型，但丢失 TYPE_USE 参数注解；在附加 `List<String>` Signature、底层实际写入 Integer 的 verifier 接受样本中，JADX 生成的源码不能按 Java 8 编译，而 Jarde 因正文源兼容不能证明而拒绝参数化投影。

## 工具与复现结果

重放于 OpenJDK / `javac` 23.0.1、JADX 1.5.6、Cargo 1.98.1。仓库根运行 `reproduce.sh` 可独立重放 1.1；`reproduce_boundaries.sh` 重放 1.2。两个脚本都把 Cargo 输出指向各自 `/tmp` 临时 target 并在退出时清除。class、反编译源码、CLI 文本和 `javap` 大输出只保留在临时目录；evidence 目录保存 fixture、脚本、结论和完整 SHA-256。

1.1 每个编译模式先对原 class 执行反射 runner，再反编译、重编 JADX/Jarde 类并以同一 runner 比对；类与 runner 均用 `javac --release 8 -Xlint:-options`，原 class、两份重编类均用 `java -Xverify:all`。JADX/Jarde 编译及运行成功。观察到的输出相同：

```text
strings parameter=java.lang.Iterable<java.lang.String> return=java.lang.Iterable<java.lang.String>
numbers parameter=java.util.List<? extends java.lang.Number> return=java.util.List<? extends java.lang.Number>
nested parameter=java.util.Map<java.lang.String, java.util.List<java.lang.Integer>> return=java.util.Map<java.lang.String, java.util.List<java.lang.Integer>>
arrays parameter=java.util.List<java.lang.String>[] return=java.util.List<java.lang.String>[]
raw parameter=java.util.List return=java.util.List
7
```

有调试表下 `javap -v` 的 method Signature：

```text
strings: (Ljava/lang/Iterable<Ljava/lang/String;>;)Ljava/lang/Iterable<Ljava/lang/String;>;
numbers: (Ljava/util/List<+Ljava/lang/Number;>;)Ljava/util/List<+Ljava/lang/Number;>;
nested:  (Ljava/util/Map<Ljava/lang/String;Ljava/util/List<Ljava/lang/Integer;>;>;)Ljava/util/Map<Ljava/lang/String;Ljava/util/List<Ljava/lang/Integer;>;>;
arrays:  ([Ljava/util/List<Ljava/lang/String;>;)[Ljava/util/List<Ljava/lang/String;>;
```

脚本用等长的 `java/util/Tree` 替换 Signature 内的 `java/util/List`，不改 descriptor。`java -Xverify:all` 成功载入该 class。反射访问 `numbers` 时因不存在 `java.util.Tree` 抛 `TypeNotPresentException`；单独访问 `nested` 可见 `Map<String,java.util.Tree<Integer>>`。Jarde 对 numbers/arrays 报 `jvm_signature_erasure_mismatch` 并回退 descriptor 的 List；但 nested 的 Map 擦除仍匹配，实际投影 Tree，Jarde bad class-source 的 Java 8 编译因找不到 `java.util.Tree` 失败。JADX 也打印 Tree 并编译失败。故负例证明直接擦除门有效，同时揭示递归嵌套类引用尚未解析；不能据此声称所有坏 Signature 都被 Jarde 拒绝。class verifier 只接受字节码，不保证 Signature 可供反射或 Java 源使用。

### 哈希（SHA-256）

生成于 2026-09-24；编译器为 OpenJDK 23.0.1 的 `javac --release 8`。`reproduce.sh` 每次重跑都会在脚本输出中逐项重算完整摘要（包括两种原 class、两份 decompiler source、bad class 与坏投影 source）；下列已知原/JADX/Jarde 成功闭环 SHA 只对应记录的实现快照。

- `OrdinaryParameterizedSignatures.java`: `32fda26827de840c2bbe93e6e6055bad49fa561e2f803f71feb5b4ae57a733f`
- `ReflectionRunner.java`: `94d74f29d1dc0413de529e8801a2e86f8091731b46977539eb230271f99c9322`
- `-g` class: `9bbfb43fb440b893be50dc7988c9180c7c1471b31f3b5e5d4bae650c696f8d3f`
- `-g:none` class: `2da818875518d62fbed477cac53fe28195f9221f52c34fd57452a9db26679fda`
- JADX `-g` / `-g:none` source: `3664ae37cb480b7773f190d381b708dea5519ed191050a868246a443b856b2ea` / `36b6312c14fb8a7fd41c60abfd160f4769077c4190376080780cdac7bb425329`
- Jarde `-g` / `-g:none` source: `fe7088a24f0470fadd1bc6d8be65bb2e82ca870d0d9522be72b495487957f220` / `ce044e233e357341e85a7237bec62e21814acb22783b71e9401156121e8a5e7a`
- Jarde List→Tree refusal source / altered class: `df7f6982cda6f23e9b4322d82af41ae81d94f27a33cb26d1bdec1bbf3b5e0f7a` / `c886d2419974d2633bfa93cfc5975facd820b707612d0b791d3e0653b69fcbe7`

## JADX 解析路径

- `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/SignatureProcessor.java`：`parseMethodSignature`（约 180 行）调用 `SignatureParser.consumeGenericTypeParameters/consumeMethodArgs/consumeType`，展开 type variables 后 `validateAndApplyTypes` 对 descriptor 参数/返回做 `validateParsedType` / `TypeCompareEnum.CONFLICT` 检查，再 `mth.updateTypes`。List→Tree 例显示它会警告但仍将错误类型打印进源码。
- `.../nodes/parser/SignatureParser.java` 是 method Signature grammar parser；`ArgType` 承载参数化/通配符/数组。JADX 从 jar 的 `InnerClasses` 事实拼出了 `Left.Entry` / `Right.Entry`，可用于比较 Jarde 当前对同类二进制路径的拒绝边界。
- `.../visitors/regions/LoopRegionVisitor.java` 的 `fixIterableType` 用 iterable generic argument 修 foreach 局部类型；它与此处的方法声明 Signature 拼写不是同一层机制。

## Jarde 架构判断

1. `jarde_reader::signature::{parse_method_signature, prove_method_signature_erasure}` 应继续做唯一语法及 erasure 基础；无需再建 grammar。当前 reader 包含 `SignatureType::{Base, Array, Class, TypeVariable}`，通配符由 `TypeArgument` 表达，本 fixture 所需普通参数化/通配符/数组都有节点表示。
2. 不要直接把 `SignatureType` 打印进声明：现有 `SignatureType::Class` 还需要递归写出实际类型参数、wildcard variance、数组、内部类后缀，并将二进制类名经过 `class_source` 已有合法 Java type spelling，而非复用 Signature 的 `/` 或 `$` 原始字符。
3. 既有声明是 `method_descriptor` 建出的擦除类型；它连接物理槽位、参数名和 descriptor。普通 generic 拼写需要在这个声明位点做 descriptor→Signature 对位覆写，且每个参数/返回分别接受 parser erasure proof；raw descriptor 方法无 Signature 时保持原样。
4. 当前 `project_method_signature` 的普通参数化分支复用同一 reader parser/erasure proof，再以同轮 Program/SSA、方法调用绑定和 source name/context proof 限制投影；正例仅限已实测源结构。不能只因 Signature 擦除正确就宣称正文 Java 类型关系闭合。
5. 1.1 的四个 identity 类方法和原 runner 在当前实现下均通过完整重编闭环；边界 1.2 显示正文和同类调用仍有独立 source proof 门槛。

这些证据对应 `recover-ordinary-parameterized-signatures` 的 1.1、1.2。它们覆盖普通签名闭环、可复现负例与边界债务；实现、定向测试和整类总体验收仍由该 change 的其他任务负责。1.2 的同类调用绑定证据由无内类依赖的 `OverloadBinding` 给出；混合 `Boundaries` 整类重编失败记录为独立 source-spelling 债务。
