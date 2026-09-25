# Java 字段泛型 Signature 取证

本记录只冻结字段 `Signature` 的当前行为，没有修改生产代码或已有 OpenSpec 变更。可从仓库根目录运行 `sh openspec/evidence/java-syntax-2026-09-24/field-generic-signatures/replay.sh` 完整重放。脚本以 `javac --release 8 -g` 和 `-g:none` 编译 fixture，执行 `javap -v`、JADX 1.5.6、当前工作树 Jarde `class-source`，编译调用方并用 `java -Xverify:all` 运行及检查 `Field.getGenericType()`。Signature 变造器只替换字段属性引用的 UTF-8 Signature，不改字段 descriptor、Code 或 flags。Cargo target 放在私有临时目录，退出时清理。

环境是 OpenJDK/Javac 23.0.1（目标 class 文件为 Java 8）、JADX 1.5.6、Cargo 1.98.1。最终脚本重放退出码为 0；私有 Cargo target 和中间产物已清理。以下 Jarde 结果描述本次共享工作树状态，不代表未来实现的限制。

## 可编译正例

最小类 `StandaloneFieldBoundary<T>` 没有字段读写方法，只有编译器生成的默认构造器。`javap -v` 确认构造器不含指向本类字段的 `Fieldref`。字段覆盖 `List<String>`、private `List<? extends Number>`、`T`、`T[]`、protected `List<? super T>`，以及带 `ConstantValue` 的 `public static final int TOKEN` 与 `String LABEL`。`-g` 和 `-g:none` 两种原 class 都通过 verifier；反射得到的字段类型依次为 `List<String>`、`List<? extends Number>`、`T`、`T[]`、`List<? super T>`、`int`、`String`。调用方按 `StandaloneFieldBoundary<String>` 编译，检查 `String current`、`String[] vector` 和参数化 List 赋值；常量值为 `41` 和 `field-signature`。

| class | `-g` SHA-256 | `-g:none` SHA-256 |
|---|---|---|
| `StandaloneFieldBoundary` | `3a7a3332c948ca4fb00d76797022395828132fc38a72a971c4ebf2638b9a97a3` | `0cde2b9a5c3765f50d63bf444365efe64368797c92b212c95ba661e64e2cd00b` |
| `FieldSignatureBoundary` | `598546e44ecec8c46b9602a10d5c7a90eb8b1bb34321a14a5861ba6ccd1528e7` | `106055412de6b1e983e89d5903e2c50cb885f26082ea38bcb41a41bd892ce1fc` |

JADX 的 Standalone 输出保留上述泛型字段和 class 变量，重新编译调用方后 `-Xverify:all` 执行成功，反射与原 class 一致。JADX 对带方法体的 `FieldSignatureBoundary` 也恢复 `List<String>`、有界通配符、`T` 和 `T[]`，并保留私有 final `numbers` 初始化；该 class 与泛型调用方完成 Java 8 重编、执行和反射。

当前 Jarde 输出仍把字段 Signature 降为 descriptor 类型：`List`、`Object`、`Object[]`。无 Fieldref 的 Standalone class 本身可以重编，但同一个强类型调用方不能编译，失败点是 `Object` 不能赋给 `String`，以及 `Object[]` 不能赋给 `String[]`。以反射 runner 对新编 class 检查时，字段类型也分别是原始 `java.util.List`、`java.lang.Object`、`java.lang.Object[]`。`-g` 与 `-g:none` 的 Jarde 输出 SHA 相同，说明此处是字段 Signature 投影缺口，不依赖调试属性。静态常量仍输出 `int`/`String` 及其常量值；它们没有泛型 Signature。

## 字段正文交互及非法 Signature

`FieldSignatureBoundary<T>` 加入实例方法、数组字段写入和 private final `numbers` 初始化，用来区分字段声明与正文依赖。原 class 与 JADX 输出均可编译执行并读回 `name,current,vector,7,sink`，反射字段仍保留泛型类型。当前 Jarde 生成的这个带正文 class 在两种调试变体下都无法重编：构造器输出结束时 private final `numbers` 未初始化。因此，这个 body-bearing 变体只证明当前源码形状会被其它字段初始化债务阻挡，不单独把失败归因于 Signature 投影；上面的 Standalone 正例隔离了字段声明缺口。

三种 verifier-valid 的 Signature 变造都保持 descriptor 不变：

| 变造 | 反射/运行结果 | JADX 1.5.6 | 当前 Jarde |
|---|---|---|---|
| `names` 的 descriptor 是 `List`，Signature 从 `List<String>` 改成 `String` | `-Xverify:all` 执行通过；`getGenericType()` 报 `java.lang.String` | 输出 raw `List`，不把擦除不符类型拼进源码 | 输出 raw `List`；物理 descriptor 得以保留 |
| `current` 的 descriptor 是 `Object`，Signature 从 `TT;` 改成未绑定的 `TV;` | verifier 与字段读取通过；反射 `getGenericType()` 抛 `NullPointerException` | 输出 `public V current`；重编时报找不到类型 V（另有本 fixture 构造器表达式的 JADX 编译错误） | 输出 `Object current`，不生成孤儿 V |
| `FieldSignatureConflict.items` 的 descriptor 是 `List`，Signature 从 `List<Object>` 改成 `List<String>`；正文仍加入整数 42 | verifier 与执行通过；反射称字段为 `List<String>`，实际取出的值是 `Integer` | 输出 `List<String>` 并生成 `items.add((String) 42)`，Java 8 重编因 int 不能转为 String 失败 | 输出 raw `List`，正文以 Object 接收 Integer；重编通过 |

最后一例说明，单凭字段 Signature 与 descriptor 擦除相符不能证明该泛型字段类型与本类方法体中的读写相容。类文件 verifier 不检查泛型字段 Signature 与 `List.add` 的类型关系。当前 Jarde 的 raw 输出避开了错误的 `List<String>` 源码声明，但丢失了调用方可用的字段泛型信息。未绑定变量则需要独立的字段 Signature 作用域门，不能将 V 直接拼入输出。

## 重放身份

核心 fixture 源码 SHA-256：`StandaloneFieldBoundary.java` `658b21e2ae8544e7181c70698244b6c35766941b6ea6c1a9d6f8c46346ccc14b`；`FieldSignatureBoundary.java` `0fa074afde85e1d569ed63c03f9c186321e24e4f02643e707c18f85d5dbc7425`；`FieldSignatureConflict.java` `fdf02c774f4094b11f88a62bc116f405d729bda69c390faec3f0bbdae06ab516`。变造 class SHA-256：擦除不符 `32a90ee5229f71b89afdad867e0d38060c35073a7f3d9c10aebfb011aff13d0d`；正文冲突 `75289fd45651998ca4b51c94947d3739c3cf3220c77797dfef83d7d156c605fb`；未绑定变量 `d28bc9bde0612e6fcee918e4ed9d01e587e179b5de8bb40ed3cf1e92e90592d2`。`mutate_field_signature.py` SHA-256 为 `ef693deb9fab71ea8c8f525708f82e649911a1cf50d45dbf5da5f2608b948bf7`；最终 `replay.sh` 为 `1010741c034f1ea550299d7ae81cbd190eb77ce0483978e4c13437d2ada94278`。重放会打印其余 fixture、JADX/Jarde 输出和 class 文件哈希。
