# 基线三方对照（实现前）

这不是验收通过记录。它只固定「自写源码、javac、jadx、jarde」这条对照在当前二进制上的结论，供 P09 实现后重跑。

- 源码：`/tmp/jarde_syntax/src/synth/p/Scenes.java`（尚未入库；入库路径由任务 7.3 定为 `tests/fixtures/proved-java-structure/`）
- `javac 23.0.1 --release 8`
- jadx 1.5.6，`--no-res --no-imports`
- jarde：`target/release/jarde-cli class-source --policy plain-jar`，对照当时的 release 二进制
- 输入：`/tmp/jarde_syntax/scenes.jar`

判据是源码，不是 jadx。`whileNotFor` 与 `tryWithResources` 两条说明跟随 jadx 会更差。

| 场景 | 源码 | jadx | jarde | 结论 |
| --- | --- | --- | --- | --- |
| `oneArmed` | `if (n > 0) x = n;` 然后 `return x`。字节码无异常表，`ifle 8` | 无 `else` 的 `if`，与源码一致 | 整方法 `explanation_only`：`ArmsDoNotMeet` 于块 0，块 6 与 8 被说成未覆盖 | 失败。P02 要求写成无 `else` 的 `if` |
| `armExits` | `if` 里赋值，`else` 抛出，汇合后 `return x` | 改成 `then` 直接 `return 2`，去掉了 `x = 1` 与汇合 | 留下 `local1 = 1` 和 `if/else`，但 `else` 是 `new`/`throw` 的引用，不是 `throw new` | 失败。前缀留下了；抛出语句没有写成 |
| `namedCatch` | `try` 包住判断、抛出和 `return`，`catch (IllegalArgumentException)` | `try` 被缩小到只包 `throw`，类型正确 | 把该区域说成 TWR 资源初始化失败，处理器块 14 只被引用 | 失败。普通 `try` 被误认为守卫。不得改去对齐 jadx 那个缩小的 `try` |
| `twoCatches` | 两个 `catch`，先 `IllegalArgumentException` 再 `IllegalStateException` | 类型与顺序与源码一致 | 同样被误认为 TWR，两个处理器只被引用 | 失败 |
| `tryWithResources` | `try (StringReader r = …)` | 展开成 `catch (Throwable)`、`close`、`addSuppressed` | 整方法引用：处理器序列不是该守卫规则证明的那一串。声明上的 `throws IOException` 也没写 | jarde 按规格留缺口（不得抄 jadx 的 `catch (Throwable)`）。`throws` 仍失败 |
| `finallyIncrements` | `try { x = n; } finally { x = x + 1; }` | 折叠成 `return i + 1` | 只写出正常路径 `x = n; x = x + 1; return`，没有 `finally` | 字节码里 `catch_type` 为 0 的处理器只覆盖 BCI 2–4（`iload`/`istore`）。按规格不写 `finally`。jadx 的折叠不是目标 |
| `countedFor` | `for (int i = 0; i < n; i++)` | `for`，与源码同形 | `while`，初值、测试、`i + 1` 都在 | 失败。结构在，P04 要求这三条同时成立时写 `for` |
| `breakAtThree` | `while (i < n) { if (i == 3) break; i++; }` | 合成一个 `while (true)`，条件变成 `i >= n \|\| i == 3` | 循环整段 `LoopShape` 引用，只留下 `i = 0` | 失败。不要改成 jadx 那个合并条件 |
| `whileNotFor` | `while (i < n) i = i + 2;` | `while (true)`，`else return` | `while (local1 < arg0) local1 = local1 + 2` | jarde 对齐源码。这是不跟随 jadx 的正例 |
| `lambdaPlusOne` | `v -> v + 1`，使用点没有合成方法名 | 箭头函数体内联 | 使用点是 `lambda$lambdaPlusOne$0`；该方法体是 `return arg0 + 1` | 失败。体已经是语句，P06 要求放回使用点，方法本身仍保留 |
| `readSecret` / `Inner.get` | 内部类直接读 `secret` | `Scenes.this.secret` | `get` 写成 `access$000(this.this$0)`。`access$000` 自身是 `return arg0.secret` | 失败。纯转发的调用点没有改写 |
| `anonymous` | `new Runnable() { run() { secret++; } }` | 匿名类内联，`run` 里是 `secret++` | 使用点是 `new Scenes$1(this)`；`run` 里是 `access$002(...)`，写 accessor 本身没有恢复 | 失败 |
| `stringSwitch` | `switch (s) { case "a" / "b" }` | 字符串 `switch`，臂与源码一致 | `hashCode` 的 `switch` 加上第二个 `switch (local2)`，`equals` 臂是缺口，返回值还在 | 不在本 change 完成条件里。不是空 `switch`，记录即可 |
| `merge` | `byte[]...`，增强 `for` | `byte[]...` 与 `for (byte[] … : …)` | 声明是 `byte[][]`，标志 `0x0089` 没有写成 `...`；循环是 `LoopShape` 引用 | 失败。varargs 与增强 `for` 都没有 |
| `declaredThrow` | `throws IOException`，体里抛的是 `IllegalStateException` | `throws` 与源码一致，`throw new` 在 | 没有 `throws`；`throw new` 是引用 | 失败。不得把 `IllegalStateException` 写进 `throws` |
| `letters` | `"正在"` | `"正在"` | `"\u6b63\u5728"` | 失败 |
| `controls` | `"a\u0000b"`。`javap` 常量同样是一个 U+0000 | `"a\u0000b"` | `"a\ufffd\ufffdb"` | 失败。不是转义风格问题：进入字面量的值已经不是 U+0000 |
| 类声明 | `package synth.p;` 与简单名 `Scenes` | 一致 | `public class synth.p.Scenes`，没有 `package` | 失败 |

`Scenes$Inner` 与 `Scenes$1` 由 jarde 各自成文件。jadx 把它们折进 `Scenes.java`。P06 允许在条件不成立时保持单独装配；条件成立后使用点必须看得到方法体，合成类型不能删到调用方对不上。

## 第二批自写场景（`More`，同样未入库）

`javac 23.0.1 --release 8 -g:none`，类在默认包。jadx 1.5.6 与当时的 release `class-source --policy plain-jar`。源码在 `/tmp/jarde_syntax2/src/More.java`。

| 场景 | jarde | 和 jadx 处理方式的关系 | 结论 |
| --- | --- | --- | --- |
| `doWhileSum` | `do/while`，与源码同形 | 已有 `LoopForm::DoWhile` | 对齐源码 |
| `intSwitch` | `case 2` 与 `case 3` 共用一个 `return` | 已有 `SwitchGroup` | 对齐源码 |
| `concat` | `arg0 + ":" + arg1` | 已有拼接 | 对齐源码 |
| `nestedIf` | 整方法 `ArmsDoNotMeet` | 字节码是两条 `ifle` 跳到同一出口。单臂 `if` 修好后应是两层 `if`，不收成 jadx 的 `&&` | 同一处修复，先不另开机制 |
| `continueSkips` | 循环整段 `LoopShape` | `continue` 已在 P04。等单臂 `if` 落地后再量，避免和正在改的走法并行 | 先不另开 |
| `ternary` | 认成 `if/else`，`then` 是空的，`ineg` 被引用 | jadx 的 `TernaryMod` 是区域成 `if` 之后的第二步。两臂都是纯值才收成 `? :` | 不是单臂语句。`ineg` 还要表达式子集认它 |
| `throwNew` | `if/else` 在，`throw new` 被拆成 `new`/`dup`/`athrow` 引用 | `new@1` 要求分配到构造之间没有别的效果；这里构造实参是字符串常量 | 现有 `new` 规则的缺口，不是新管线 |
| `synchronizedInc` | 整方法引用，monitor 体被拒 | 体是 `iaload`/`iastore`。普通数组读写今天只为枚举 `switch` 的分派表开放（`build.rs` 的 `Operation::ArrayLoad`） | 先补数组读写，再谈 monitor 体能不能包住它 |
| `instanceOf` / `castString` | `instanceof` 与 `checkcast` 整段引用 | `checkcast` 今天只在 `bridge@1` 证明是擦除时才写 | 表达式子集加这两个操作数，不是新管线 |
| `arrayInit` / `forEach` | `newarray`/`iastore`/`arraylength`/`iaload` 被引用。`forEach` 的循环在，但加数写成了没有声明的 `local5` | `iaload` 的存储被引用吃掉，后面的 `iload` 仍按槽位拼名 | 数组读写是同一处子集缺口。`local5` 是引用吃掉定义、使用还在，要和数组读写一起看 |

## 第三批（`Ops`，未入库）

源码在 `/tmp/jarde_syntax3/src/Ops.java`，`javac 23.0.1 --release 8 -g:none`。对照用的是实现单臂 `if` 之前编出的 release `class-source`，所以下面的失败不是单臂改动引入的。这一批还没有跑 jadx。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| `bits` `(a & b) \| (a ^ b)` | `iand` / `ixor` / `ior` | 全部 `Other`，整方法 explanation_only | 失败。`BinaryOp` 已有算术和比较，没有位运算。补 opcode 表即可，不需要新区域 |
| `shifted` `(a << 1) >> 1` | `ishl` / `ishr` | 全部 `Other` | 失败。同一处 opcode 表 |
| `narrowed` `(byte) n` | `i2b` | `Other` | 失败。`ExprKind::Cast` 与 `Type::Byte` 已有，没接到这条 opcode。类型只由 opcode 决定 |
| `ifNull` | `ifnull` | `if (arg0 == null) return 0; else return 1;` | 对齐源码。两臂都离开方法，走已有两臂路径 |
| `negated` `return !flag` | `ifne` 两臂各一个常量，汇合处 `ireturn` | `if (!arg0) {}`，随后把汇合值说成入口栈 | `!` 写出来了。两臂常量在汇合处丢失。这是三元，不是再改单臂判断 |

jadx 1.5.6 对同一份 `ops.jar` 写成 `(i & i2) | (i ^ i2)`、`(i << 1) >> 1`、`obj == null ? 0 : 1`、`(byte) i`、`return !z`。`ifNull` 和 `negated` 被收成表达式，这是它的三元改写，不是 jarde 必须抄的文本。`bits`、`shifted`、`narrowed` 与源码同形，jarde 缺的是运算符和转换，不是区域。jadx 还加了 `package defpackage`，那是它自己的偏离，不跟随。

## 第四批（`More2`，未入库）

源码在 `/tmp/jarde_syntax4/src/More2.java`，`javac 23.0.1 --release 8 -g:none`。对照仍是单臂 `if` 之前的 release `class-source`，以及 jadx 1.5.6。

| 场景 | jarde | jadx | 结论 |
| --- | --- | --- | --- |
| `this(1)`、`super()`、`this.count = arg1` | 与源码同形 | 同形，去掉了 `super()` | 对齐源码。构造链已经能写 |
| `this.count = this.count + 1`、`More2.shared = More2.shared + 1` | 与源码同形 | 收成 `count++` / `shared++` | 对齐源码。不跟随 `++` 糖 |
| `return ++i` | `local1 = local1 + 1; return local1;` | `return i + 1` | 对齐源码的求值。先改槽位再读，名字仍然有效 |
| `return i++` | 写出加一，然后引用：返回值是改槽位之前的值 | `int i2 = i + 1; return i;` | 失败。返回旧值，但不能用已经被 `iinc` 改过的槽位名。不新开区域 |
| `n += 2` | `arg0 = arg0 + 2` | `return i + 2` | 对齐源码。`+=` 不必单独成为节点 |
| `case 1` 穿透 `case 2` | `SwitchArmsOverlap`，块 37 和 40 未覆盖 | 两个 `case`，中间没有 `break` | 失败。块只应属于后一个 case，前一个臂不补 `break`。不新增 `Region` |
| `long` 相加、`a == b` | `arg0 + arg2`；`if (arg0 == arg1)` | 同形；`==` 被收成 `? :` | 对齐源码。引用相等已有。不跟随三元 |
| `synchronized (lock) { n = n + 1; }` | `synchronized` 与体内加法都在 | 同形 | 对齐源码。monitor 体是普通加法时已经能包住。之前的失败是数组体 |
| `while (true) { if (n > 3) break; n++; }` | `while (arg0 <= 3)` | 同样的 `while` | 对齐字节码。javac 把 `break` 折进了头部测试，class 里没有 `while (true)` |
| `return '中'` | `return 20013;` | `return (char) 20013;` | 值对，类型没写。返回描述符是 `C`。`(char)` 或字符字面量都由描述符和常量证明。jadx 没写字符本身 |
| `static { shared = 7; }` | 静态块里 `More2.shared = 7` | 挪到字段 `static int shared = 7` | 对齐源码。不把 `<clinit>` 搬到字段上 |

## 第五批（`More3`、`Ops2`、`Color`，未入库）

源码在 `/tmp/jarde_syntax5/src/More3.java`，`javac 23.0.1 --release 8 -g:none`。对照仍是单臂 `if` 之前的 release 二进制，以及 jadx 1.5.6。

| 场景 | jarde | jadx | 结论 |
| --- | --- | --- | --- |
| `n / 2.0`、`1.5f` | `ldc2_w` / `ldc` 是 `Other`。`ddiv` 本身在算术范围内 | `d / 2.0d`、`1.5f` | 失败。常量种类没有 float/double。不需要新区域 |
| `assert n >= 0` | 认成可重入的 `if`，`throw new AssertionError` 被引用 | `if ($assertionsDisabled \|\| i >= 0) return i; throw new AssertionError()` | 结构失败来自两条分支落到同一 `ireturn`，即已落地的单臂 `if`。不另写 `assert` |
| `for (Integer value : list)` | `iterator()` 留下了，`hasNext` 循环因测试块里的调用被 `StatementFree` 拒绝 | `while (it.hasNext())`，并把 `next()` 收进循环体 | 失败。条件表达式已经能写调用，纯度检查过严。不猜 `for-each` |
| `break outer` | 外层循环整段 `LoopShape` | `break loop0` | 失败。P04 要求这条边写成带标记的 `break`，且不能吞掉两层循环。无标记 `break` 会离开错的那一层 |
| 多 catch 同一处理器 | 仍被 `jre_guard_resource_init` 领走，处理器块 26 未覆盖 | `catch (IllegalArgumentException \| IllegalStateException e)` | 与命名 `catch` 同一处。两条记录指向同一入口时是一个多 catch，不能把处理器走两遍 |
| `left.equals(right)` | `return arg0.equals(arg1);` | 同形 | 对齐源码 |
| 接口默认方法与静态方法 | 方法体对齐，但 `plus` 没有 `default` | `default int plus`、`static int zero` | 体已对齐。声明缺 `default`。标志 `0x0001`、接口、非 static、非 abstract |
| `enum` 的 `ordinal()` 与构造 | `return this.ordinal()`，`super(arg1, arg2)` | 常量写在枚举头，方法同形 | 这两处对齐。`$values` 的 `anewarray` 和 `<clinit>` 的 `new` 仍是已知缺口，不新增枚举区域 |

## 第六批（`More4`，未入库）

源码在 `/tmp/jarde_syntax6/src/More4.java`，`javac 23.0.1 --release 8 -g:none`。对照仍是单臂 `if` 之前的 release 二进制，以及 jadx 1.5.6。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `~n` | `iconst_m1; ixor` | `ixor` 是 `Other` | `i ^ (-1)` | 失败，但不是一元 `~`。2c.1 的 `ixor` 写出来就是 jadx 这句，不必再加 `~` |
| `n >>> 1` | `iushr` | `Other` | `i >>> 1` | 已在 2c.1 |
| `n &= 3` | `iand; istore; iload; ireturn` | `iand` 被引用后仍 `return arg0` | `return i & 3` | 值在引用之后用了槽位名。`iand` 成为表达式后应是 `arg0 = arg0 & 3; return arg0`。不跟随 jadx 把赋值收进 `return` |
| `for (;;) return 1` | 只剩 `iconst_1; ireturn` | `return 1` | `return 1` | 对齐字节码。javac 删掉了到不了的回边，没有 `for` 可恢复 |
| `continue` | `if_icmpne` 的落空边是 `goto` 回头部 | 空的 `then`，`else` 里做减法 | `if (i2 != 2) i--` | 这个形状的值是对的：`continue` 是整个 `then`，跳过的语句在 `else`。P04 仍要求能写成 `continue`。jadx 的取反不跟随 |
| `try (StringReader)` | 标准 close / addSuppressed / athrow | `jre_guard_handler`，整方法引用 | 展开成 `try/catch (Throwable)` 并内联 `close` | 允许的缺口。这是 TWR 证明没通过，不是普通 `catch`。不得照抄 jadx |
| 空 `catch` 后 `return 0` | 处理器是 `astore; iconst_0; ireturn` | 仍被 TWR 误认 | `catch` 里 `return 0`，并把 `if` 挪到 `try` 外 | 处理器体是 `return 0`，不是空块。命名 `catch` 落地后应按处理器写，不跟随 jadx 缩小 `try` |
| `throw e` | `astore; aload; athrow` | 同上，处理器未覆盖 | `throw e` | 2a：`athrow` 读到的值能渲染时写成 `throw <expr>`。这里的值是 catch 参数 |
| `More4.class` | `ldc` Class | `Other` | `More4.class` | 已在 2c.5 |
| `new int[2][3]` | `multianewarray [[I, 2` | `Other` | `new int[2][3]` | 新的表达式缺口。维度只取操作数，类型只取池里的数组类。不是两次 `newarray` |
| `String::length` | `invokedynamic`，无捕获 | `(String p0) -> p0.length()` | 同样是 lambda，不是 `::` | 对齐含义。`lambda@1` 已经把它写成等价的 lambda。不单开方法引用糖 |
| 实例初始化块 | 被 javac 拷进 `<init>` | `super(); More4$Nested.k = 1;` | 同样在构造器里 | 对齐字节码。不把语句搬回独立的初始化块 |
| `inner.read()` | `invokevirtual` | `return arg1.read()` | 同形 | 对齐源码 |

## 第七批（`More5`，未入库）

源码在 `/tmp/jarde_syntax7/src/More5.java`，`javac 23.0.1 --release 8 -g:none`。对照仍是单臂 `if` 之前的 release 二进制，以及 jadx 1.5.6。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `super.value()` | `invokespecial Base5.value` | `this.value()` | `super.value()` | 失败，而且是另一个程序。`invokespecial` 的属主不是当前类时必须写成 `super`。私有调用保持 `this` |
| `long`/`float` 比较 | `lcmp`/`fcmpg` 后接 `ifge` | 整段 `Other` | 三元表达式 | 失败。比较指令是值，不是新区域。`lcmp` 可收成 `<`。`fcmpg` 只有与 NaN 同为假的极性能收成运算符，不跟随三元 |
| `static final int N = 3` | 字段有 `ConstantValue: int 3`；`named` 是 `iconst_3` | 字段无初值，`return 3` | 字段 `= 3`，方法 `return N` | 字段初值是属性，该写。方法保持字面量。不跟随 jadx 把内联还原成字段 |
| `static synchronized` | 标志 `0x0020`，体里没有 `monitorenter` | `static synchronized`，`return arg0 + 1` | 同形 | 对齐 |
| `volatile`、`null`、`%`、空方法 | 普通指令 | `volatile`、`return null`、`%`、`return` | 空方法没有 `return` | 对齐字节码。空方法的 `return` 不删 |
| `continue outer` | `goto 2` 指向外层头 | 外层整段引用 | 改写成内层 `do/while`，没有 `continue` | 失败。与带标记 `break` 同一规则。不跟随 jadx 的改写 |
| 协变桥 | `invokevirtual get()String`，标志 `0x1040` | `return this.get()` | 删掉桥，只留 `String get()` | 失败。读起来是自调用。桥保留，文本必须带被调用的描述符。不跟随删除 |

## 第八批（`More6`，未入库）

源码在 `/tmp/jarde_syntax8/src/More6.java`，`javac 23.0.1 --release 8 -g:none`。对照仍是单臂 `if` 之前的 release 二进制，以及 jadx 1.5.6。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `byte[]` 读取和写入 | `baload`/`bastore`，循环测试含 `arraylength` | 循环整段引用；`bastore` 是 `Other`，`return` 留下 | `for (byte b : bArr)`，`bArr[i] = b` | 失败。同一条下标表达式。不跟随 `for-each` |
| `char[]` 的 `do-while` | 闩锁上 `caload` 与 `arraylength`，回边到头部 | 前缀赋值在，测试失败 | `do { i += cArr[i2]; } while (...)` | 失败在值，不在循环形状。已有 `DoWhile` |
| `long`/`float`/`double`/`short`/`boolean` 读取 | `laload`/`faload`/`daload`/`saload`/`baload` | 整段 `Other` | `return xs[i]` | 失败。`boolean[]` 与 `byte[]` 共用 `baload`，类型只能来自已证明的数组类型 |
| `h.outer.value` | 两次 `getfield` | `return arg1.outer.value` | 同形 | 对齐 |

## 第九批（`More7`，未入库）

源码在 `/tmp/jarde_syntax9/src/More7.java`，`javac 23.0.1 --release 8 -g:none`。对照仍是单臂 `if` 之前的 release 二进制，以及 jadx 1.5.6。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `&&` / `||` | 两条前向边汇到同一个 `return 0` | 汇合块写成可重入循环，`return 0` 丢失 | 收成三元 | 失败。该块不是循环头。写在两层 `if` 之后一次。不跟随 `&&`、`||` 或三元 |
| 三元 | 两臂各推一个值，汇合处 `ireturn` | 空 `if`，汇合值丢失 | `return i > 0 ? i : i2` | 仍是延后的三元 |
| `synchronized`、接口调用、`this(1)`、私有 `invokespecial` | 已有形状 | `synchronized (arg0)`、`arg0.run(arg1 + 1)`、`this(1)`、`this.hidden(arg1)` | 同形 | 对齐 |
| `(String)` / `instanceof` / `i2l` / `i2b` | `checkcast`、`instanceof`、`i2l`、`i2b` | `Other` 或未证明的转换 | `(String) obj`、`instanceof`、`return i`、`(byte) (b + 1)` | 失败。接到已有转换节点。`i2l` 保留 `(long)`，不跟随省略 |
| `this.count++` | `getfield; dup_x1; iadd; putfield` | `dup_x1` 是 `Other` | 拆成局部变量再赋值 | 失败。只有这一条栈形写成 `this.count++`。不发明局部变量 |
| `new String[n]` | `anewarray` | `Other`；局部名没有声明 | `new String[i]`、`new String[]{str}` | 失败。复用 `NewArray`。不跟随初始化糖 |
| 捕获 lambda、匿名类、无标签 `switch` | 已有形状 | 等价 lambda、`new More7$1()`、`switch` 含 javac 补的 `default` | 内联体、匿名类、同样的 `switch` | 对齐字节码。内联仍是 5.2/5.3，不在这批 |

## 第十批（`More8`，未入库）

源码在 `/tmp/jarde_syntax10/src/More8.java`，同一编译和对照条件。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| 字符串 `+` | `StringBuilder` | `return arg0 + ":" + arg1` 与 `return arg0 + arg1` | 同形 | 对齐。不加任务 |
| `return -n` 的三元 | `if`、`ineg`、汇合 `ireturn` | `ineg` 是 `Other`，汇合值丢失 | `return i < 0 ? -i : i` | 取负是已有算术的一元形式。汇合值仍是延后的三元，不拿这条验收取负 |
| `a[i] = i` 的计数循环 | `arraylength` 在测试里，`iastore` | 循环整段引用 | `for` | 已是 2b.4 与计数 `for`。不新增 |
| `try` 前有 `int x = 1` | 范围从 `istore` 之后开始 | 该 `istore` 被当成资源头，`catch` 丢失 | 把 `if` 搬出 `try` | 失败就是 2.4。不跟随把条件搬出去 |

## 第十一批（`More9`，未入库）

源码在 `/tmp/jarde_syntax11/src/More9.java`，同一编译和对照条件。release 二进制早于已经落地的命名 `catch`，所以从方法入口开始的 `try` 仍显示旧的资源头拒绝。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `while ((n = step(n)) > 0)` | `invokestatic; dup; istore; ifle` | 测试被拒绝 | `while (true)`，赋值搬进体 | 失败。条件写成 `(arg0 = step(arg0)) > 0`。不跟随改写 |
| 空 `catch` 与再抛 | 范围从 0 开始；再抛是 `aload; athrow` | 资源头拒绝 | 把 `if` 搬出 `try` | 分类已由 2.1 覆盖，这条对照过时。再抛是已有的 `throw` 值。不跟随搬家 |
| `n += 3` | `iinc 3` | `arg0 = arg0 + 3; return arg0` | `return i + 3` | 对齐。不丢掉存储 |
| `g[i][j]` | `aaload; iaload` | `aaload` 是 `Other` | `return iArr[i][i2]` | 失败。两层下标。外层不得变成 `int` |
| 标记块 `break` | javac 收成 `ifge` 与 `goto` | `if (arg0 < 0) return -1; else return arg0` | 无 `else` 的同形 | 对齐字节码。标记已经不在 |
| `long` 移位与按位与 | `lshl`、`land` | `Other` | `j << 1`、`j & 3` | 失败。同一批运算符的 `long` 形式 |
| `switch (char)` | 键是 97、98 | `case 97` | `case 'a'` | 程序相同，字符更可读。选择表达式是 `int` 时不改 |

## 第十二批（`More10`，未入库）

源码在 `/tmp/jarde_syntax12/src/More10.java`，同一编译和对照条件。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| 计数 `for` | 初值、比较、`iinc` | `while (local2 < arg0)`，体内 `local2 = local2 + 1` | `for`，`i2 += i3` | 程序已对齐。`for` 是已有任务，不新增区域 |
| `return n++` | `iload; iinc; ireturn` | 先写成 `arg0 = arg0 + 1`，返回旧值被拒绝 | `int i2 = i + 1; return i` | 失败，已有任务。不跟随多出来的局部 |
| `return ++n` | `iinc; iload; ireturn` | `arg0 = arg0 + 1; return arg0` | `return i + 1` | 对齐。不丢掉存储 |
| `switch` 穿透 | case 1 落到 case 2 | `SwitchArmsOverlap`，整段引用 | 补 `break`，把 `return` 搬到外面 | 失败，已有任务。不跟随补 `break` |
| `continue` 后 `break` | 条件臂跳回头部 | 空 `if`，离开循环的边被引用 | 合成 `&&`，`continue` 消失 | 失败，已有 `continue` 任务。不跟随合成条件 |
| `iand`/`ixor`/`ishl`/`ishr` | 这些 opcode | `Other` | 运算符对齐 | 失败，已有任务 |
| `1.5f`、`2.0`、`String.class` | `ldc`/`ldc2_w` | `Other` | `1.5f`、`2.0d`、`String.class.getName()` | 失败，已有常量任务 |
| 拆箱 | `invokevirtual intValue` | `return arg0.intValue()` | 同形 | 对齐。不收成 `return n` |
| 装箱 | `invokestatic Integer.valueOf` | `return valueOf(arg0)` | `Integer.valueOf(i)` | 失败。属主不是当前类，类型必须写出 |

## 第十三批（`More11`，未入库）

源码在 `/tmp/jarde_syntax13/src/More11.java`，同一编译和对照条件。release 二进制早于主工作区上已经落地的单臂 `if`。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `a = b = n` | `iload; dup; istore; istore` | `dup` 被引用，随后却 `return local1 + local2`，两个局部没有声明 | `return i + i`，两次存储消失 | 失败。先写入的局部写下表达式，后写入的局部读它。不得把表达式写两遍，也不跟随收成 `i + i` |
| `do { i = i - 1; } while (i > 0)` | 闩锁测试 | `do` / `while` 对齐 | `i2--` | 对齐。不改成自减糖 |
| 嵌套 `if` 后同一 `return 0` | 两个前向边进入 BCI 11 | 内层 `else` 抢走 `return 0`，外层 `else` 被当成可重入 | `\|\|` | 失败，就是已有的前向汇合。不跟随合成条件 |
| `new int[] { 1, 2, 3 }` | `newarray; dup; iastore` 三次 | `dup` 与 `iastore` 整段引用 | `new int[]{1, 2, 3}` | 不写这个花括号。创建本身是已有的数组任务；这里的 `dup` 不是两次局部存储 |
| `"a\nb\t\"c\\d"` | 字符串常量 | 转义对齐 | 同形 | 对齐 |
| `/` 与 `%` | `idiv`、`irem` | `arg0 / 2`、`arg0 % 2` | 同形 | 对齐 |
| `o.getClass()` | `invokevirtual` | `return arg0.getClass()` | 同形 | 对齐。实例调用本来就带着接收者 |

## 第十四批（`More12`，未入库）

源码在 `/tmp/jarde_syntax14/src/More12.java`，同一编译条件。release 二进制早于已经落地的单臂 `if` 和数组下标，所以 `a[i]` 与字符串 `switch` 里的 `equals` 仍显示旧拒绝。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `this.n += k` | `dup` 接收者，`getfield`，加，`putfield` | `dup` 被引用，`return this.n` 还在 | `this.n += i` | 失败。写成 `this.n = this.n + arg1`。不是 `++` |
| `a[i]++` | `dup2; iaload; dup_x2; iadd; iastore` | 整段引用。release 还没有普通 `iaload` | 发明 `int i2`，再写回 | 失败。旧元素写成 `arg0[arg1]++`。不跟随发明局部 |
| `if ((n = n - 1) > 0)` | `isub; dup; istore; ifle` | `dup` 把整段引用 | 先赋给另一个局部 | 失败。与循环里的赋值条件是同一条规则 |
| 两边都 `return` | `ifge; return; return` | `if` 的两臂都是 `return` | 丢掉 `else` 和后一个 `return` | 对齐字节码 |
| `return a == b` | 比较后 `iconst_1` / `iconst_0` | 空 `if`，返回值丢失 | `return obj == obj2` | 失败。只接受 0 和 1。别的两臂仍不写三元 |
| `n += 1L` | `ladd; lstore` | `arg0 = arg0 + 1L; return arg0` | `return j + 1` | 对齐。不丢掉存储 |
| 密集 `tableswitch` | `tableswitch` | `switch` 四个 `case` 对齐 | 同形 | 对齐 |
| 字符串 `switch` | `hashCode` 再 `equals` | `equals` 报两臂不相交。release 没有单臂 `if` | 还原成 `switch (str)` | 不还原。`equals` 两边都到下一个 `switch`，是已有的单臂 `if` |

## 第十五批（`More15`，未入库）

源码在 `/tmp/jarde_syntax15/src/More15.java`。release 二进制没有字段 `++`，也没有这一轮的循环体续接。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `return ++n` | `dup` 接收者，加法之后 `dup_x1` | 整段引用 | 发明局部再写回 | 失败。留下的是新值，写成 `return ++this.n` |
| `return n++` | `dup` 接收者，`dup_x1` 在加法之前 | 整段引用 | 发明局部，返回旧值 | 失败。已有字段后增，序列要带上接收者的 `dup` |
| `N += k` | `getstatic; iadd; putstatic` | `More15.N = More15.N + arg0` | `N += i` | 对齐。静态字段没有接收者 `dup`，不写 `+=` |
| `else if` | 两段都返回的 `if` | 嵌套 `if`，三个 `return` | 内层收成三元 | 对齐。不跟随三元 |
| 循环里的 `switch` 后还有语句 | `switch` 汇合后减一再回边 | 整个 `while` 是 `LoopShape` | `while` 里 `switch` 和两次 `--` | 失败。循环体丢掉了 `switch` 的后继。不是新区域 |
| `switch` 臂跳出循环 | `goto` 循环出口；默认臂回边 | 整个 `while` 是 `LoopShape` | 回边被写成循环里的 `return` | 失败。出口必须是带标记的 `break`，因为无标记 `break` 断开的是 `switch`。不跟随 `return` |
| 稀疏 `lookupswitch` | 键 1 与 100 | `switch` 对齐 | 同形 | 对齐 |
| `n <<= 1` | `ishl; istore` | `ishl` 引用，`return arg0` 仍是旧值 | `return i << 1` | 失败。移位是已有任务。恢复后保留存储，不写 `<<=` |
| `for` 两处更新 | 两个 `iinc` | `while`，两次更新都在 | 折进一个不完整的 `for` | 对齐。不是单点更新，保持 `while` |
| `o == null` 两臂都返回 | `ifnonnull` | `if` / `else` 两个 `return` | 去掉 `else` | 对齐字节码 |

## 第十六批（`Join`、`More16`，未入库）

源码在 `/tmp/jarde_syntax16/src`。release 二进制没有前向汇合，也没有循环体续接。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `a > 0 && b > 0` | 两条 `ifle` 都到 `return 0` | 内层 `else` 抢走 `return 0`，外层 `else` 被当成可重入 | `(i <= 0 \|\| i2 <= 0) ? 0 : 1` | 失败。两层单臂 `if`，`return 0` 只写在外层之后。不收成 `&&` 或三元 |
| `a > 0 \|\| b > 0` | `ifgt` 与内层落空都到 `return 1` | 内层两臂都写了，外层另一边被当成可重入 | `i > 0 \|\| i2 > 0` 的三元 | 失败。落空的一边是汇合，所以 `then` 是空的，`return 0` 留在 `else`。不交换两臂，也不收成 `\|\|` |
| `while (n > 0) n = n - 1` | 头测循环 | `while` 后 `return arg0` | `while` 里 `--` | 对齐。出口不是前向汇合 |
| `while` 里 `try` 之后还有赋值 | 保护范围在循环体内，汇合后才减一 | 整个循环是 `LoopShape` | `while` 还在，但把 `if` 拎到 `try` 外面 | 失败。与 `switch` 一样，是循环丢掉了结构的后继。`if` 在异常表范围内，不跟随 jadx 把它移出 `try` |

## 第十七批（`More17`，未入库）

源码在 `/tmp/jarde_syntax17/src/More17.java`。release 二进制早于单臂 `if`、命名 `catch` 和数组下标。当前 `resources()` 对「范围内没有前一条指令」已经返回 `NotGuarded`，所以空 `catch` 的 CLI 拒绝不能当成现在的失败。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `this(1)` | `invokespecial <init>(I)V` | `this(1)` | 同形 | 对齐 |
| `a + n + "b"` | `append(String)` 与 `append(int)` | `return arg0 + arg1 + "b"` | 同形 | 对齐 |
| `"x" + c` | `append(C)` | 留下 `StringBuilder` | `"x" + c` | 失败。`append(C)` 是这一个字符，应收进已有的 `+`。`append(char[])` 仍不收 |
| 捕获 lambda | `invokedynamic` | 等价 lambda，体留在 `lambda$apply$0` | 使用点内联 | 对齐现有写法。内联仍是 5.2，不在这里加 |
| `String::length` | `invokedynamic` | `(String p0) -> p0.length()` | 同样是 lambda | 对齐。不另写方法引用糖 |
| 匿名类 | `new More17$1` | `return new More17$1()` | 内联 `run` | 已有 5.3。不跟随内联 |
| `throw new` | `new; dup; invokespecial; athrow` | `if` 在，构造被引用 | `throw new` | 已有 `new@1` 缺口。不新开 |
| 空 `catch` | 保护范围从 BCI 0 开始，处理器只有 `astore` | release 按资源初始化拒绝 | 空 `catch` | 当前源码应是 `NotGuarded` 再走命名 `catch`。用测试确认，不按这份 CLI 加任务 |
| `int[]` 的 for-each | `arraylength` 加 `iaload` | release 把长度和下标都引用，还写出未声明的 `local5` | `for (int i2 : iArr)` | 数组 worktree 已覆盖下标。不写成 `for-each` |
| 迭代器 | 测试块是 `hasNext` | `StatementFree` 拒绝循环 | `while (it.hasNext())` | 已有 2c.6。不猜成 `for-each` |
| `synchronized` | `monitorenter` | `synchronized (arg0)` | 同形 | 对齐 |
| `assert` | 两个分支都到 `return`，然后 `new AssertionError` | release 把汇合当成可重入 | `if (!flag && n <= 0) throw` | release 早于单臂 `if`。不写 `assert`。剩下的是 `throw new` |
| `<clinit>` | `ldc Class`，两臂是 1 和 0，再 `putstatic` | `ldc` 是 `Other`，空分支 | `!More17.class.desiredAssertionStatus()` | 失败。`Class` 字面量是已有 2c.5。0/1 存进字段是 2c.17 的同一种值，不写成 `assert` |
| `instanceof` / `checkcast` | `0xc1` / `0xc0` | 整段引用 | `instanceof` 与 `(String)` | 已有 2c.9 |
| `n / 2.0f` | `fconst_2; fdiv` | `fconst_2` 是 `Other`，除法本身已在算术范围 | `f / 2.0f` | 失败。五个常量指令值是确定的，不走池。补进 2c.5 |
| 外层 `while` 没有回边 | 内层 `goto` 只回到内层头，出口与后面的 `break` 是同一块 | 外层 `ArmsDoNotMeet` | 收成 `&&`，并删掉只赋值一次的 `m` | 失败在内层 `break`，是已有 3.1。外层没有回边，保持 `if`，不还原成 `while` |

## 第十八批（`More18`，未入库）

源码在 `/tmp/jarde_syntax18/src/More18.java`。`switch` 穿透、移位、取负和按位与仍是当前 `region.rs` / `decode.rs` 的行为，release 二进制没有落后。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `do { n = n - 1; } while (n > 0)` | 体在前，测试在闩 | `do` / `while (arg0 > 0)` | `do` 里写成 `i--` | 对齐。不把 `n = n - 1` 收成 `--` |
| `ArrayList::new` | `invokedynamic`，没有捕获 | `return java.util.ArrayList::new` | 同形 | 对齐。已有 lambda 规则在捕获为空、句柄是构造器时就写方法引用。泛型被擦掉，不在本 change |
| `switch` 穿透 | `case 1` 落到 `case 2` | `SwitchArmsOverlap`，整段引用 | `case 1` 没有 `break`，但把一个局部拆成两个 | 已有 2c.4。停在后一个 case 入口，不拆局部 |
| `return n << 1` | `ishl` | `Other` | `return i << 1` | 已有 2c.1。这里没有再存储，所以就是 `return arg0 << 1`，不是 `<<=` |
| `return -n` | `ineg` | `Other` | `return -i` | 已有 2c.11 |
| `return n & 3` | `iand` | `Other` | `return i & 3` | 已有 2c.1。不收成 `&=` |

## 第十九批（`More19`，未入库）

源码在 `/tmp/jarde_syntax19/src/More19.java`。`break` 仍是当前循环规则的失败，release 没有落后。`continue`、`+ null` 和被编译器删掉的 `switch` 已经对齐。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `while` 里 `break` | `goto 19` 到出口，另一臂减一再回头 | 整个循环是 `LoopShape` | 收成 `while (i > 0 && i != 1)` | 失败。已有 3.1 的本层 `break`。不收成 `&&` |
| `while` 里 `continue` | 相等时 `goto` 回头，不等才相加 | 空的 `then`，加法在 `else` | 条件取反，`if (i != 2)` | 对齐。空臂就是这次 `continue`。不改成关键字，不取反 |
| `s + null` | `aconst_null` 再 `append` | `return arg0 + null` | 给 `null` 加了 `(Object)` | 对齐。不加转换 |
| 只有 `default` 的 `switch` | 编译器收成 `return -1` | `return -1` | 空 `switch` 再 `return` | 对齐。不还原编译器删掉的 `switch` |

## 第二十批（`More20`，未入库）

源码在 `/tmp/jarde_syntax20/src/More20.java`。嵌套 `try` 的「范围不同就不是一条 `try`」是当前 `catches()` 的行为，不是旧二进制。数组下标和 `i2l` 的 release 输出早于对应任务，但字节码已经对上。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `return new int[]{1, 2, 3}` | `newarray` 后三次 `dup; 下标; 值; iastore`，没有局部 | 整段引用 | 花括号 | 失败。没有槽位，不能发明局部。整段下标正好是 `0..2` 时才写成一次 `new int[]{1, 2, 3}`。一次存储仍不是花括号 |
| `return n` 到 `long` | `i2l` | `Other` | 省掉转换 | 已有 2c.8。指令在，写成 `(long) arg0` |
| 块体 lambda | 捕获后调用生成方法 | 等价 lambda，体留在 `lambda$block$0` | 使用点内联 | 对齐。不另写块体 |
| `Integer::bitCount` | 实现取 `int`，调用点绑 `Integer` | 形状不同，整段引用 | 写成 lambda，藏起拆箱 | 拒绝对。拆箱没有被证明，不写成方法引用 |
| 嵌套 `try` | `[0,12)` 与 `[0,17)`，内层处理器在外层范围内 | 当前 `catches()` 对范围不同返回 `None` | 把 `if` 拎出去，`try` 只包住 `throw` | 失败。内层 `try` 放进外层的体。不收成并列 `catch`，也不跟随把 `if` 移出 |
| `return a[i++]` | `iload` 后立刻 `iinc`，`iaload` 读的是旧值 | release 先写 `arg1 = arg1 + 1`，下标读取被引用 | `int i2 = i + 1; return iArr[i]` | 已有 2c.3。文本是 `return arg0[arg1++]`，不得用新值做下标 |

## 第二十一批（`More21`，未入库）

源码在 `/tmp/jarde_syntax21/src/More21.java`。`throw` 的引用是当前规则：`athrow` 只有被守卫领走才写。空方法已经对齐。三元仍按原决定不收。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `throw e` | `astore` 后再 `aload; athrow` | release 把 `throw new` 误认成资源，处理器没被走到 | `throw e` | 失败。用户 `throw` 不是守卫。`throw new` 与 `throw local1` 都要写。不推断 `throws` |
| 空方法 | 只有 `return` | `return;` | 空方法体 | 对齐。不删掉 `return` |
| `flag ? a : b` | 两臂各压一个值，汇合处 `ireturn` | 空 `if`，汇合的值没有生产者 | `return z ? i : i2` | 仍是推迟的三元。不写 `? :` |
| `int...` 的声明 | `arraylength`，标志 `0x0088` | release 写成 `int[]`，长度是 `Other` | `int...` 与 `.length` | 声明是已有 6.2，尚未并入。`.length` 是已有 2b.1 |
| `args(1, 2, 3)` | `newarray` 加三次 `iastore`，再 `invokestatic` | 整段引用 | `args(1, 2, 3)` | 失败。这是 2b.7 的表达式当参数：`args(new int[]{1, 2, 3})`。不猜可变参数调用 |

## 第二十二批（`More22`，未入库）

源码在 `/tmp/jarde_syntax22/src/More22.java`，枚举在同目录的 `Color.java`。枚举 `switch` 的文本是当前 `enumswitch@1` 的写法，不是旧二进制。`iushr` 和 `i2b` 仍是未解码指令。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `switch (color)` | `getstatic $SwitchMap`、`ordinal`、`iaload`、`lookupswitch` | `switch (More22$1.$SwitchMap$Color[arg0.ordinal()])`，键是 1 和 2 | `case RED` / `case BLUE` | 对齐现有证明。常量在另一个类的字段里，这个类没有写出 1 是 `RED`。不跟随 jadx 写成枚举常量 |
| `return n >>> 1` | `iushr` | `Other` | `return i >>> 1` | 已有 2c.1。不写成 `>>>=` |
| `return (byte) n` | `i2b` | `Other` | `(byte) i` | 已有 2c.2。目标类型只由 opcode 决定 |

## 第二十三批（`More23`，未入库）

源码在 `/tmp/jarde_syntax23/src/More23.java`。静态调用的文本来自当前规则：`invokestatic` 不写属主。字段 `++` 的返回形式已经另测，这里的 release 文本不代表它。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| `return Integer.valueOf(n)` | `invokestatic java/lang/Integer.valueOf` | `return valueOf(arg0);` | 失败。已有 4.4。写成 `java.lang.Integer.valueOf(arg0)`。不收成装箱 |
| `return own(n)` | `invokestatic` 属主就是当前类 | `return own(arg0);` | 对齐。不得改成 `More23.own` |
| `return N++` | `getstatic; dup; iconst_1; iadd; putstatic; ireturn` | release 整段引用 | 新的 2c.21。没有接收者。`return N++;`，不是 `+=` |
| `return ++N` | `getstatic; iconst_1; iadd; dup; putstatic; ireturn` | release 整段引用 | 同一项。`dup` 留下的是新值，写成 `return ++N;` |
| `int x = this.n++` | 实例后增后接 `istore`，再 `iload` | release 引用复制后 `return local1`，没有声明 | 同一项。必须是 `int local1 = this.n++; return local1`。不得留下没有声明的名字 |

## 第二十四批（`More24`，未入库）

源码在 `/tmp/jarde_syntax24/src/More24.java`。`y = x = 1` 的引用来自早于链式 `dup` 的二进制。`*=` 与方法上的 `synchronized` 是当前能写出的形状。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `this.n = n` | `putfield` 在 `super()` 之后 | `super(); this.n = arg1;` | `this.n = i` | 对齐。构造器里的赋值留在原处 |
| `N = 1` | `putstatic` | `More24.N = 1` | `N = 1` | 对齐现有静态字段拼写。属主类型保留，不改成裸名 |
| `y = x = 1` | `iconst_1; dup; istore; istore` | release 引用 `dup` 后 `return local2 + arg1`，`local2` 没有声明 | `return 1 + 1` | 已有 2c.14，release 过时。不得折成两个常量。当前文本应是先赋值再读这两个局部 |
| `n *= 2` | `imul; istore` | `arg1 = arg1 * 2; return arg1` | `return i * 2` | 对齐。不写成 `*=`，也不丢掉存储 |
| `a != null && a.length > 0` | `ifnull` 再 `arraylength` | release 引用长度，汇合被说成可重入 | 收成 `\|\|` 并交换两臂 | 长度是尚未并入的 2b.1。不得收成 `&&` 或 `\|\|`。release 的「可重入」早于前向汇合 |
| `synchronized` 方法 | 标志 `0x0020`，体内没有 `monitorenter` | `synchronized int sync()` 与 `return this.n` | 同样 | 对齐。不补一个不存在的块 |
| `synchronized (this) { return n; }` | `monitorexit` 后直接 `ireturn`，没有 `goto` | 整段拒绝：退出没有落在每条路径上 | 把值存进局部，`return` 在块外 | 失败。新的 2.6。写成块里的 `return this.n`。不发明局部，不二次读取 |

## 第二十六批（`More26`，未入库）

源码在 `/tmp/jarde_syntax26/src/More26.java`。同一份源码分别用 `--release 11` 和 `--release 8` 编译。Java 11 的文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `a + "/" + n`（release 11） | `invokedynamic` `makeConcatWithConstants`，配方 `\u0001/\u0001` | 整方法拒绝：引导方法不是 `LambdaMetafactory` | `str + "/" + i` | 失败。新的 2c.22。写成 `arg0 + "/" + arg1`。不写成 lambda |
| `"" + n`（release 11） | 配方只有一个 `\u0001` | 同样整方法拒绝 | `return i` | 同一项。必须是 `"" + arg0`。jadx 丢掉了转换成字符串 |
| `a + b`（release 11） | 配方 `\u0001\u0001` | 同样整方法拒绝 | `str + str2` | 同一项。`arg0 + arg1` |
| 同一三段（release 8） | `StringBuilder` 的 `append` 再 `toString` | `arg0 + "/" + arg1`、`"" + arg0`、`arg0 + arg1` | 未另跑 | 已对齐。不得改这条链 |

## 第二十七批（`More27`，未入库）

源码在 `/tmp/jarde_syntax27/src/More27.java`，`javac --release 9`。文本来自 release 二进制。`try` 与数组读取之后已有别的改动，这里只记这份二进制实际写出的句子。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `try (r) { return r.read(); }` | `aload; astore` 复制资源，`ifnull` 后 `close`，没有 `goto`，然后 `iload; ireturn` | 当前库写成 `local1 = arg0` 后 `catch (Throwable)`，里面还有 `close` 和 `addSuppressed`，正常路径的块未覆盖 | 同样展开 | 失败。已有 2.8。这不是源码。写成 `try (java.io.Reader local1 = arg0)`，`return` 在外面。不把编译器的关闭写成用户 `catch` |
| 空 `catch` 后 `return 0` | `invokeinterface`，`goto` 汇合，处理器只有 `astore` | 当前库是 `try { arg0.run(); } catch (RuntimeException local1) {} return 0;` | 两臂各写一次 `return 0` | 对齐。2.7 已写出调用。汇合只写一次，不得把 `return` 复制进两个臂 |
| `a[i] += 2; return a[i]` | `dup2; iaload; iconst_2; iadd; iastore`，然后再 `iaload` | 整段引用，`iaload` 被当成枚举分派表 | `iArr[i] = iArr[i] + 2; return iArr[i]` | 失败。新的 2c.23。不是 2c.16 的 `++`：没有 `dup_x2`，留下的也不是旧值。不写 `+=` |
| `this(n, 1)` | `invokespecial <init>(II)V` | `this(arg1, 1);` | `this(i, 1)` | 对齐。已有的构造器序言 |
| `o.value()` | `invokevirtual` | `return arg0.value();` | 同样 | 对齐 |

## 第二十八批（`More28`，未入库）

源码在 `/tmp/jarde_syntax28/src/More28.java`，`javac --release 21`。文本来自 release 二进制。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `return switch (n) { case 1 -> 2; case 2 -> 3; default -> 0; }` | 每臂 `iconst; goto`，汇合一条 `ireturn` | 三个空 `break`，汇合处引用：返回值没有生产者 | 每臂 `return 2` / `return 3` / `return 0` | 失败。新的 2c.25。写成臂内的 `return`，不写成 `return switch`，不发明局部 |
| `""" line """` | `ldc` 字符串 `line\n` | `return "line\n"` | 同样 | 对齐。字节码没有文本块，不得写成 `"""` |
| `o instanceof String s` | `instanceof`、`checkcast`、`astore`，再取 `length` | 整段引用：`instanceof` 不是已证明子集 | `if (obj instanceof String) return ((String) obj).length()` | 已有 2c.9。jadx 也没有写成模式变量。不另加模式节点 |
| `yield` 一臂先存储 | `istore` 后再 `iload; goto`，汇合 `ireturn` | `int local1 = arg0 + 1; break;`，汇合处同样引用 | `return i + 1`，存储被丢掉 | 同一项 2c.25。必须保留存储：`return local1`。不跟随 jadx |

## 第二十九批（`More29`，未入库）

源码在 `/tmp/jarde_syntax29/src/More29.java`，`javac --release 16`。文本来自 release 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| `record More29(int x, String name)` | `final class` 继承 `java.lang.Record`，`Record` 属性 | `public final class More29 extends java.lang.Record`，字段、构造器赋值、`x()`、`name()` 都在 | 对齐成类。不改写成 `record`，也不删这些成员 |
| `doubled` | `getfield; iconst_2; imul; ireturn` | `return this.x * 2` | 对齐 |
| `equals` / `hashCode` / `toString` | `invokedynamic` `ObjectMethods.bootstrap` | 整段拒绝：引导方法不是 `LambdaMetafactory` | 保持拒绝。这不是 lambda，也不把引导方法展开成方法体 |

## 第三十批（`More30`，未入库）

源码在 `/tmp/jarde_syntax30/src/More30.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。`case 1` 落到 `case 2` 的入口，没有 `goto`。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| `case 1` 穿透到 `case 2` | `case 1` 在 28，`istore` 后直接是 30 的 `case 2` | 整段 `explanation_only`：两臂占有同一块 | 已有 2c.4。赋值和 `switch` 在同一块，不是 1.2 的前缀块。不新增区域 |
| `for (int x : a)` | 复制数组、`arraylength`、下标、`iaload`、`iinc` | `while (local4 < local3)`，`arraylength` 与 `iaload` 被引用 | 已有 2b。循环已经是 `while`。不得写成 `for-each` |
| `for (String s : list)` | `iterator`、`hasNext`、`next`、`checkcast` | `iterator()` 已写出；循环被拒绝，因为测试里的 `hasNext` 不是值表达式 | 已有 2c.6。不得写成 `for-each`。未覆盖的块是这次拒绝留下的，不是新的异常边规则 |
| `switch (s) { case "ab" }` | `hashCode` 再 `equals`，然后第二个 `switch` | `switch (local1.hashCode())` 里是 `equals("ab")`，第二个 `switch` 返回 1 或 0 | 对齐字节码。不得恢复成 `case "ab"`。第一个 `switch` 的汇合是另一个 `switch`，不是单条 `return`，2c.25 不得改它 |

## 第三十一批（`More31`，未入库）

源码在 `/tmp/jarde_syntax31/src/More31.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。四处全部对齐，不新增任务。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| `for (;;) { n++; if (n > 10) return n; }` | 回边指向体首，无头部测试 | `do { arg0 = arg0 + 1; } while (arg0 <= 10); return arg0;` | 对齐。字节码本身就是 do-while 回边，写出的程序与源码同行为。不新写 `for (;;)` |
| `while (true) { if (n > 0) return n; n--; }` | 头部测试 `ifle`，回边 `goto` 头 | `while (arg0 <= 0) { arg0 = arg0 - 1; } return arg0;` | 对齐。条件转写后是同一个程序：`n <= 0` 时递减，`n > 0` 时返回。不写 `while (true)` 也可以 |
| `if (n > 0) ;` | 分支两臂落到同一块 | 整条 `if` 不写，只有 `return arg0;` | 对齐。两后继相同，无效果分支，丢弃是诚实的 |
| `s = s + "x"; return s;` | `StringBuilder` 链 | `arg0 = arg0 + "x"; return arg0;` | 对齐。concat@1 已覆盖，不写 `+=` |

## 第三十二批（`More32`，未入库）

源码在 `/tmp/jarde_syntax32/src/More32.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `do { … break; } while (false)` | `goto` 前跳，无回边 | 嵌套 `if`/`else`，与源码同程序 | 嵌套 `if`，但内层用了三元 `i3 == 3 ? 2 : 3`，且把声明拆出 | 对齐。1.4 的前向汇合已覆盖。jadx 的三元是我们明确的非目标（jadx 实测，本轮补跑） |
| 两个顺序 `try`/`catch`，体都不能抛 | 两行异常表，处理器在正常流之外 | 整段引用：第二个处理器的字节没有任何块覆盖 | 两个 `try`/`catch` 都呈现 | 失败。新的 2.9。行只挂在抛点上，图不认这张表 |
| `catch` 里嵌 `try`，两层体都不能抛 | 两行表，内层在外层处理器里 | **`catch` 无声消失**：只写 `arg0 = arg0 + 1; return arg0;`，无任何标记 | **内层 try 被丢弃**：`catch` 体只写 `i2 = -1`（死保护被删） | 失败，正确性问题。同一根因 2.9。两者都不忠实：jadx 删掉用户写的嵌套保护，jarde 丢掉整个 catch。2.9 落地后 jarde 两层都呈现，比 jadx 忠实（jadx 实测，本轮补跑） |

## 第三十三批（`More33` + `Color`，未入库）

源码在 `/tmp/jarde_syntax33/src/`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| 静态块 `static { counter = 7; }` | `<clinit>` | `static {` 块内 `More33.counter = 7;` | 同样 | 对齐 |
| 实例初始化块 `{ counter++; }` | javac 并入每个 `<init>` | 构造器里 `More33.counter = More33.counter + 1;` | 同样并入 | 对齐。字节码真相，不上提 |
| 枚举 `values()` | `getstatic $VALUES; clone; checkcast; areturn` | `Color.$VALUES.clone();` 写出，`checkcast`/返回引用 | `clone()` 完整 | 差 `checkcast`（2c.9）。`clone()` 作为语句已写出 |
| 枚举 `valueOf(String)` | `ldc class; invokestatic Enum.valueOf; checkcast; areturn` | 整段引用 | 完整 | 差 `ldc` Class（2c.5）与 `checkcast`（2c.9） |
| 枚举常量 `<clinit>` | `new; dup; ldc; iconst_0; invokespecial; putstatic` ×3，再 `$values()` | 三个 `new` 链全部引用，`Color.$VALUES = $values();` 写出 | `RED, GREEN, BLUE;` 声明 | 失败。新的 2c.26：字段写入读到 `dup` 遗留值。不跟随 jadx 的声明合并 |

## 第三十四批（`More34`，未入库）

源码在 `/tmp/jarde_syntax34/src/More34.java`，`javac --release 8 -g:none`。隔离 2c.26 的适用面：局部存储对齐，字段写入全部被拒。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| 构造器里 `this.a = new Object()` | 引用：`Duplicate` 生产者 | 2c.26 |
| 方法里 `this.a = new Object()` | 同上 | 2c.26 |
| 静态方法里 `More34.s = new Object()` | 同上 | 2c.26 |
| `Object o = new Object(); return o;` | `local1 = new java.lang.Object(); return local1;` | 对齐，new@1 已覆盖 `astore` 一路 |

## 第三十五批（`Shape`/`Square`/`More35`，未入库）

源码在 `/tmp/jarde_syntax35/src/`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| 接口抽象方法 | 无 `Code` | `public abstract double area();` 加无体标记 | 同样 | 对齐 |
| 接口 `default` 方法体 | 普通体 | 体对齐（`"area " + this.area()`）；**签名缺 `default`** | `default String describe()` | 6.5 在声明 worktree 未并入主工作区，不是新任务 |
| 接口 `static` 方法体 | 普通体 | 签名 `public static` 正确；体是 `return new Square(1)` | `return new defpackage.Square(1.0d);`（`1.0d` 是 `dconst_1` 的字节码真相，源码的 `1` 已丢失） | 体失败：新的 2c.27 |
| `return new Square(1)` | `new; dup; invokespecial; areturn` | 整段引用：`Duplicate` 遗留值无消费者拼写 | `return new Square(1.0d)` | 失败。2c.27。`areturn` 读 `dup` 遗留值 |
| `long a + b * 3L` | `lload/lmul/ladd` | `return arg0 + arg2 * 3L;`（双槽参数 arg0/arg2） | 同样 | 对齐 |
| `double + long` | `l2d` 后 `dadd` | 整段引用 | `return d + j;`——`l2d` 被丢弃，恢复成源码级算术提升 | 已有 2c.8（加宽转换）。落地后 jarde 写显式 `(double)`（指令存在，字节码真相），与 jadx 的源码级拼写是有意分歧，不跟随（jadx 实测，本轮补跑） |
| varargs 方法声明 | 标志 `0x0088` | `static int many(int arg0, int[] arg1)` | `int... iArr` 签名 + 体写成 `for (int i3 : iArr)` | 失败。新的 6.7：`ACC_VARARGS` 最后参数拼 `int... arg1`。体是数组 `for-each`，停在 2b（jadx 的 for-each 是明确非目标） |
| `many(1, 2, 3)` 调用点 | `newarray` 初始化链再 `invokestatic` | 整段引用 | `many(1, 2, 3)` | 已有 2b.7 |
| 三元 `n > 0 ? n : -n` | 两臂汇合到 phi | 单臂 `if` 空体、`-n` 引用、汇合 phi 引用 | `n > 0 ? n : -n` | 按规格正确拒绝：不写三元、不发明局部 |

## 第三十六批（`More36`/`Marker`/`Ops`，未入库）

源码在 `/tmp/jarde_syntax36/src/`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `transient`/`volatile`/`static volatile` 字段 | 各标志位 | 全部正确拼写 | 同样 | 对齐 |
| `native` 方法 | 无 `Code` | `native int compute(int arg1);` 带无体标记 | 同样 | 对齐 |
| `strictfp` 方法 | 0x0800 | 签名正确；体差 `ldc2_w` double 常量 | **`strictfp` 被丢弃**：写成 `double f(double d)`，体 `return d * 2.0d;` | 签名对齐（jarde 保留标志，jadx 丢失——竞争力优势，jadx 实测补跑）；体是已有 2c.5 |
| `@interface Marker` | ACC_ANNOTATION 类 | `public @interface Marker extends java.lang.annotation.Annotation` | 同样 | 对齐 |
| `String value() default "x"` | `AnnotationDefault` 属性（`s#10`/`I#13`） | 默认值不写：只有 `public abstract java.lang.String value();` | `default "x"` | 失败。新的 6.8：属性是工件自述的默认值，与 ConstantValue（6.6）同类 |
| 枚举常量带体（`ADD { … }`） | 每常量一个匿名子类 | `Ops$1`/`Ops$2` 作为独立类呈现；`enum Ops$1` 拼写经核实（该类确带 ACC_ENUM 0x4030）；`apply` 体对齐 | 合并进常量声明 | 按类文件忠实呈现，不合并。`Ops(String,int,Ops$1)` 合成构造器写出，保留 |
| 枚举 `<clinit>` 常量赋值 | `new; dup; invokespecial; putstatic` | 引用 | 合并 | 已有 2c.26 |
| 嵌套类 `More36$Base` | 独立 class 文件 | `abstract class More36$Base`，`abstract int size();` 无体标记，`twice()` 体 `return this.size() * 2;`（实测） | 同样分文件 | 对齐（与 jadx 的文件布局一致） |

## 第三十七批（`More37`，未入库）

源码在 `/tmp/jarde_syntax37/src/More37.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。全部落在已有任务，不新增。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| `a.equals(b)` / `equalsIgnoreCase` | `invokevirtual` | 完整写出 | 对齐 |
| 异或求和循环 | `^`/`&` 在 `Other`，`baload` 未证 | 整段引用 | 2c.1（位运算）+ 2b（数组读取） |
| `(n << 5) - n + (n >>> 3)` | `ishl/iushr/isub/iadd` | 引用 | 2c.1 |
| `(char) (c - 'a' + 'A')` | `i2c` 在 `Other` | 引用 | 2c.2 |
| `(s << 8) \| (b & 0xff)` | `ishl/iand/ior` | 引用 | 2c.1 |
| `byte b = (byte) n; return b;` | `i2b` | 拒绝，但原因被包装为「描述符返回 `byte` 而值呈现为 `int`，没有连接两者的转换证据」 | 仍是 2c.2（`i2b` 解码后此拒绝自然消失）。原因链诚实：`i2b` 是 `Other` → 无表达式 → 返回位类型不匹配。记录在案，不新增任务 |

## 第三十八批（`Merge`，未入库）

源码在 `/tmp/jarde_syntax38/src/Merge.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。四处全部对齐，不新增任务。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| 两臂各赋同一局部再汇合 | 局部 phi | `if/else` 两臂各写 `local1 = 1` / `= 2`，汇合 `return local1;` | 对齐。不写成三元 |
| `if` 前先赋初值、单臂更新 | `m = 0` 在前 | `local1 = 0; if (…) { local1 = local1 + 1; } return local1;` | 对齐 |
| 连续早期返回 `if (n<0) return -1; if (n==0) return 0; …` | 一串前向分支 | 嵌套 `if/else`，四个 `return` 各在位 | 对齐。这是 jadx 常见拼写，jarde 的嵌套写法与其同程序 |
| 汇合读取 phi 局部 | `iload` phi | 名字正确指到合并后的局部 | 对齐（P11 的既有能力） |

## 第三十九批（`Box2`，未入库）

源码在 `/tmp/jarde_syntax39/src/Box2.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| `Integer.valueOf(x)` 装箱赋值/返回 | 真实调用 | `java.lang.Integer local2 = java.lang.Integer.valueOf(arg1); return local2;` | 对齐（精度规则：不折叠装箱） |
| 拆箱 `intValue()` | 真实调用 | `return arg1.intValue();` / `+ arg2.intValue()` | 对齐 |
| 循环里 `r = r + i` | 循环内 concat 链 | `while (local4 < arg2) { local3 = local3 + local4; … }` | 对齐 |
| `return n > 0`（读字段） | `ifgt` 两臂常量汇合 `ireturn` | 空 `if` + 汇合 phi 引用 | 已有 2c.17（`this.n` 版实例） |
| `while (n > 0) { n = n - 1; }`（读写字段） | 测试块含 `getfield` | 循环整段拒绝：测试块指令不属于值表达式 | 2c.6 的同类情形，任务文本已补「字段读取可以留在条件里」。体内的 `putfield` 不经 `dup`，field@1 照常写 `this.n = this.n - 1`，不是 2c.15 |

## 第四十批（`Deep`，未入库）

源码在 `/tmp/jarde_syntax40/src/Deep.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。全部落在已有任务，不新增。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| 递归 `fib(n-1) + fib(n-2)` | 调用进算术 | `return fib(arg0 - 1) + fib(arg0 - 2);` | 对齐。调用作为值已覆盖 |
| 静态自调用 `fib(7)` | `invokestatic` | `return fib(7);` | 对齐（4.4 后同类不限定） |
| `new int[n]` 动态长度 | `newarray` | 引用 | 数组 worktree 2b.2，未并入 |
| 布尔 `p & q` / `p \| q` | `iand`/`ior` | 引用 | 2c.1 |
| `src.clone()` 返回 | `invokevirtual clone; checkcast; areturn` | `arg0.clone();` 写成语句，`checkcast` 与返回引用紧随 | 已有 2c.9（普通 `checkcast`）。落地后是 `return (int[]) arg0.clone();`。当前呈现诚实：引用紧邻，值未无声消失 |

## 第四十一批（`Node`/`Cfg`，未入库）

源码在 `/tmp/jarde_syntax41/src/`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| 链式字段 `n.next.next.val` | 三次 `getfield` 依值链 | `return arg0.next.next.val;` | 对齐。field@1 值链已覆盖 |
| 构造器里 `this.f = 参数` | `aload; iload; putfield` | `this.next = arg1;` 等 | 对齐 |
| `new Node(null, val)` 存局部再读字段 | `aconst_null` 作参数 | `Node local1 = new Node(null, arg0); return local1.val;` | 对齐。`null` 参数与 `astore` 消费都正确 |
| 接口常量 `int LIMIT = 100` | 字段 `ConstantValue` | `public static final int LIMIT;` 无初值 | 6.6 已在声明 worktree 完成未并入，非新任务 |

## 第四十二批（`Null`，未入库）

源码在 `/tmp/jarde_syntax42/src/Null.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| `if (s == null) return null; … return instance;` | `if_acmpne` + `aconst_null` | 完整对齐，`null` 字面量正确 | 对齐 |
| `return o == null;` | 两臂常量汇合 `ireturn` | 空 `if` + 汇合 phi 引用 | 2c.17 的引用比较实例（`aconst_null` 作操作数） |
| `this.helper()` / `helper() + helper()` | `aload_0; invokevirtual` | `return this.helper();` / `return this.helper() + this.helper();` | 对齐。接收者与重复调用正确 |
| `n.helper()`（字段/参数作接收者） | `aload; invokevirtual` | `return arg0.helper();` | 对齐 |

## 第四十三批（`Outer`/`Outer$Inner`/`Lbl`，未入库）

源码在 `/tmp/jarde_syntax43/src/`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| 内部类构造器捕获外部引用 | `putfield this$0` 在 `invokespecial` **之前** | `this.this$0 = arg1; super();`（按字节码顺序） | 构造器为空：捕获被整个丢弃，访问改写为 `Outer.this.count` | 失败。新的 2.10：`super()` 前触碰 `this` 不是合法 Java。javac 只把合成捕获放在 prologue 前，改序是唯一合法拼写。jadx 的丢弃不是合法替代（jadx 实测，本轮补跑） |
| `this$0` 合成字段声明与读取 | `getfield this$0` | `final Outer this$0;` 声明与 `this.this$0` 读取正确 | 字段与读取都不写（转 `Outer.this`） | 对齐 |
| 合成访问器 `access$000` | `getfield` 转发 | 方法保留，调用点写 `Outer.access$000(...)` | 内联成 `Outer.this.count` | 对齐（P05：accessor 保留，4.1 才内联） |
| `new Outer$Inner(this)` 跨类构造 | `new; dup; aload_0; invokespecial` | `Outer$Inner local1 = new Outer$Inner(this); return local1.read();` | 同样 | 对齐（`astore` 消费路径已覆盖） |
| 嵌套二维数组循环 + `break outer` | 测试含 `arraylength`，二维 `aaload` | 整段引用（loop@1 先拒测试块） | `break loop0` 完整还原（jadx 实测） | 已有任务链：2b.6（`arraylength` 进条件）→ 2b（二维 `aaload`）→ 3.1（带标签 `break`）。不新增 |

## 第四十四批（`Str`，未入库）

源码在 `/tmp/jarde_syntax44/src/Str.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| 链式调用 `s.trim().toLowerCase()` / 三层 `.replace().concat().intern()` | 值链上的连续 `invokevirtual` | 完整写出 | 同样 | 对齐 |
| `substring` / `startsWith` / `getBytes`（返回 `byte[]`） | 普通调用 | 完整写出 | 同样 | 对齐 |
| `return s.charAt(k)`（char→int 隐式加宽） | **无**转换指令 | `return (int) arg0.charAt(arg1);` | `return str.charAt(i);` 无 cast（jadx 实测） | 失败。新的 2c.29：会合位发明了 `(int)`，违反「无转换指令的加宽不写」 |
| `s.indexOf(c)`（char 参数传 int 位） | 无转换指令 | `arg0.indexOf((int) arg1)` | `indexOf(c)` 无 cast（jadx 实测） | 同上 2c.29 |
| `String.format("%d", n)` | `anewarray; dup; aastore` 初始化链 + `invokestatic` | 引用 | `String.format("%d", Integer.valueOf(i))`（可变参数糖，jadx 实测） | 已有 2b.7（装箱元素同链）。不新增；jadx 的可变参数糖正是 2b.7 明确不猜的形状 |

## 第四十五批（`Widen`，未入库）

源码在 `/tmp/jarde_syntax45/src/Widen.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。确认 2c.29 的完整边界。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| `int i = c;`（存储位隐式加宽） | 无转换指令 | `int local1 = (int) arg0;` | 2c.29 的存储位实例。落点已确认：`meeting_position` 的 `Conversion::Widening` 臂无条件包 `Cast` |
| `long l = n;`（真实 `i2l`） | `i2l` | 拒绝（`i2l` 未解码） | 2c.8 值层职责，与本任务互补：有指令的在值层写，没指令的不写 |
| `pass(s.charAt(0))`（参数位隐式加宽） | 无转换指令 | `pass((int) arg0.charAt(0))` | 2c.29 的参数位实例 |

落点结论：`meeting_position` 的 doc 自称「文本陈述字节码执行的转换」，但隐式加宽处字节码没有执行任何指令——这是把「描述符说 `char`、位置要 `int`」误当成了「发生了转换」。位置层加宽包装在任何上下文都不需要：Java 在赋值、调用、返回上下文都隐式加宽。

## 第四十六批（`Run`/`Run$1`/`Run$2`/`Poly`，未入库）

源码在 `/tmp/jarde_syntax46/src/`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。注意：本轮共享树上 2c.26+27 正在改 `build.rs`，`named()` 的 `return new Run$1();` 可能是 WIP 效果，不作为已验收结论。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| 匿名 `Runnable` 按类呈现 | `Run$1` 独立 class | `class Run$1 extends java.lang.Object implements java.lang.Runnable`，`run()` 体 `java.lang.System.out.println("anon");` 完整 | 对齐（5.3 之前的诚实状态：按类呈现，不内联） |
| 匿名类带字段与方法 | `Run$2` | 字段 `extra`、构造器 `this.extra = 3`（实例初始化块并入，字节码真相）、`add` 体 `return arg1 + this.extra;` | 对齐 |
| 接口多继承 `extends A, B` | `interfaces` 表 | `public interface Poly extends java.lang.Runnable, java.io.Serializable` | 对齐 |
| `Run$1`/`Run$2` 的 `named()`/`hold()` 调用点 | `new; dup; invokespecial; areturn` | `return new Run$1();` / `return new Run$2();` | 2c.27 的实例（WIP 树上已写出；验收时以该任务自己的 fixture 为准） |

## 第四十七批（`Not`，未入库）

源码在 `/tmp/jarde_syntax47/src/Not.java`，`javac --release 8 -g:none`。jarde 文本来自当前 debug 二进制，jadx 已实测。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `if (!b) return -1; else return n;` | `ifne` 极性 | `if (!arg0) { return -1; } else { return arg1; }` | `if (z) return i; return -1;`（换臂不取反） | 对齐。分支极性反转已覆盖，两边拼写不同但同程序 |
| 嵌套 `if` 无 else（`if (b) { if (n>0) return 1; } return 0;`） | 前向分支 | 嵌套 `if` 完整 | 同样 | 对齐 |
| `return !b` | `iload; ifne; iconst_1/goto; iconst_0; ireturn` | 空 `if` + 汇合 phi 引用 | `return !z` | 已有 2c.17（返回侧）。布尔操作数、真臂是 `0`、假臂是 `1`，拼 `!b`；任务文本已补此特例与布尔一元 `!` 的边界 |
| `boolean c = !b; if (c) …` | 同形状存储侧 | **中间文本不诚实**：`if (local1 != 0)` 用未声明局部加非法布尔比较 | `return !z ? 1 : 0;`（三元） | 已有 2c.17（存储侧）。jadx 三元是非目标；jarde 的 `!= 0` 对布尔局部是非法 Java，2c.17 落地时须一并消除，当前记录为已知中间态 |

## 第四十八批（`Local`/`Local$1Helper`/`Cont`，未入库）

源码在 `/tmp/jarde_syntax48/src/`，`javac --release 8 -g:none`。jarde 文本来自当前 debug 二进制，jadx 已实测。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| 方法内局部类按类呈现 | `Local$1Helper` 独立 class，捕获 `val$n` 与 `this$0` | 字段、构造器、`twice()` 体 `return this.val$n * 2;` 全部呈现 | **内联成不可编译代码**：`new java.lang.Object(this)`、`this.this$0 = this`（混淆内外 this），类型推断告警 | 对齐且胜出。jarde 按类忠实呈现；jadx 此处输出非法 Java（jadx 实测） |
| 局部类构造器捕获顺序 | `val$n`、`this$0` 两个 `putfield` 都在 `invokespecial` 之前 | `this.val$n = arg2; this.this$0 = arg1; super();`（按字节码顺序） | 内联乱序 | 失败。已有 2.10（任务文本已扩到 `val$` 捕获） |
| `new Local$1Helper(this, arg1)` 跨类带参构造 | `new; dup; aload; iload; invokespecial; astore` | `Local$1Helper local2 = new Local$1Helper(this, arg1); return local2.twice();` | ——（内联） | 对齐（`astore` 消费路径） |
| `continue outer`（单层循环里） | 跳到递增/头 | 整段引用（先被 2b.6 的 `arraylength` 拦住） | 条件反转收进 `if (a[i] >= 0) { … }`，标签消失 | 已有任务链 2b.6 → 3.1。jadx 本例的反转恰好同程序；3.1 落地后 jarde 写带标签 `continue`，不跟随反转（jadx 实测） |

## 第四十九批（`Fin`，未入库）

源码在 `/tmp/jarde_syntax49/src/Fin.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。`finally` 是本 change 的明确非目标：验证的是诚实拒绝，不是恢复。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| `try { } finally { }` | catch-all 行 + 副本 + 重抛 | 整段引用（处理器形状不可证） | 按规格诚实拒绝。不写成用户 `finally`，也不无声丢失 |
| `try/catch/finally` 三段 | 命名 catch + catch-all | `try`/`catch` 结构写出，体与 finally 副本各带引用 | 按规格：能证的结构呈现，finally 副本不写成用户代码 |
| multi-catch `catch (A \| B e)` | 两行共享一个处理器 | `catch (java.lang.IllegalArgumentException \| java.lang.IllegalStateException local1) { arg0 = -1; }` 完整 | 对齐（2.1/2.4 已验收工作的确认） |

注意：2.9（异常表行为图事实）落地后 `plainFinally` 的引用边界可能移动——finally 行也是表声明的行；该任务的验收清单里已含 `finallyPath` 的重测。

## 第五十批（`Cast`，未入库）

源码在 `/tmp/jarde_syntax50/src/Cast.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| 引用加宽返回（`Integer` → `Number`） | 无指令 | `return arg0;` | `return num;` | 对齐。引用加宽不需要 cast，会合位也没发明 |
| `(String) o` 强转（检查型） | `checkcast` | 引用（cast 无证明） | `(java.lang.String) obj` 写出 | 已有 2c.9（主工作区尚未包含该任务） |
| `o instanceof String` 条件 | `instanceof` | 引用 | `instanceof ? cast : ""` 三元拼写 | 已有 2c.9；jadx 的三元是非目标 |
| `(CharSequence) o` 返回 | `checkcast` | 引用 | 写出 | 已有 2c.9 |
| **组合缺陷记录** | — | `viaString`/`safeCast` 的引用块与语句**混合**：`if (local1.length() > 0)` 用未声明局部出现在空 `if` 里 | 完整恢复 | 1.1/1.2（前缀保留，在跑）与 2c.9 都落地后才干净的组合形态。当前中间态记录在案，不单独开任务——两个在途任务的验收都应包含这个形状的复查 |

## 第五十一批（`Switch9`，未入库）

源码在 `/tmp/jarde_syntax51/src/Switch9.java`，`javac --release 8 -g:none`。jarde 文本来自当前 debug 二进制，jadx 已实测。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `char` 方法返回 int 常量（`return 'A'`） | `iconst 65`（无转换指令） | `return 65;`（数值拼写） | `return 'B';`（字符字面量） | 失败。新的 2c.30：`char` 返回位写 `'A'`。`return 65` 对 `char` 描述符是非法 Java；jadx 实测用字符拼写，本条 jarde 落后 |
| 共享标签 `case 90: case 95:` | 两键同目标 | 两个标签一行体，正确 | 同样 | 对齐 |
| default-only switch | 直落 `return` | `return 0;`（无 switch） | 空 `switch {}` + `return 0` | 对齐。jarde 更简洁且同程序 |
| `case 1: case 2:` 共享返回 | 两键同目标 | 两个标签共享 `return 12;` | 同样 | 对齐 |

## 第五十二批（`Arr`，未入库）

源码在 `/tmp/jarde_syntax52/src/Arr.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。全部落在已有任务，不新增。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| 静态字段数组初始化 `{10,20,30}` | `<clinit>` 里 `newarray` 初始化链 + `putstatic` | 引用 | 已有链：2b（数组）→ 2c.26（字段写入的构造/数组值）→ 2b.7（初始化链拼写）。不把数组上提到字段声明——字段只有 `ConstantValue` 才上提（6.6），数组无该属性 |
| 实例字段 `{1}` | 构造器里初始化链 + `putfield` | 引用 | 同上（2b + 2c.26） |
| `p &= q`（布尔复合赋值） | `iload; iload; iand; istore` | `iand` 引用 + `return arg1;` 混合呈现 | 已有 2c.1（`iand`）。不写 `&=`；落地后是 `arg1 = arg1 & arg2` |
| 三元取数组元素 `n > 0 ? TABLE[0] : TABLE[1]` | 两臂各一次数组读取汇合 phi | `if/else` 结构 + 臂内数组读取引用 + 汇合引用 + `return local2` | 已有：2b（数组读取）+ 1.1/1.2（汇合 phi）；按规格不写三元 |

## 第五十三批（`Chain`，未入库）

源码在 `/tmp/jarde_syntax53/src/Chain.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| `return a > 0 && b > 0` | 两条件嵌套分支，两臂常量汇合 `ireturn` | 嵌套 `if` + 汇合引用，原因写成「block at BCI 13 can be re-entered and belongs to no loop」 | 已有 2c.17 的复合条件实例（按规格不折叠 `&&`）。**附加观察**：BCI 13 是两个前向前驱的普通汇合，`re-entered` 的说法像误诊，1.1/1.2 验收时复查此形状 |
| 早期返回 + 尾部 `return c > 0` | 一串前向分支，尾比较两臂常量 | 嵌套 `if/else` 全对齐，尾部留 1 个未覆盖块引用 | 2c.17 返回侧（`c > 0` 两臂常量） |
| `else if` 阶梯（三级） | 前向分支链 | 嵌套 `if/else` 四个 `return` 各在位 | 对齐 |

## 第五十四批（`Num`，未入库）

源码在 `/tmp/jarde_syntax54/src/Num.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。全部落在已有任务，不新增。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| `Integer.toHexString` / `Long.parseLong`（跨类静态调用） | `invokestatic` | 限定名写出，返回直接 | 对齐（4.4 后静态限定已覆盖） |
| `Math.sqrt(n)` | `i2d` 后 `invokestatic` | `i2d` 引用 | 已有 2c.8 |
| `a[i % a.length]` | `arraylength` + `irem` + `iaload` | `arraylength` 与数组读取引用 | 已有 2b/2b.6（`irem` 在算术子集内） |
| `(b & 0xff) << 4` | `iand`/`ishl` | 引用 | 已有 2c.1 |
| `d > f`（double 与 float 比较） | `f2d`/`dcmpl` + 分支 | 引用 + 汇合引用 | 已有 2c.7（NaN 极性规则正为此形状）+ 2c.8 |
| `d != d`（NaN 检测） | `dcmpl` + `ifeq` | 引用 | 已有 2c.7。浮点密集代码（加密参数混淆常见）目前整体停在比较指令上，2c.7 是该族的主闸门 |

## 第五十五批（`Ctl`，未入库）

源码在 `/tmp/jarde_syntax55/src/Ctl.java`，`javac --release 8 -g:none`。jarde 文本来自当前 debug 二进制，jadx 已实测。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| 带标签块 `block: { … break block; }` | javac 收成 if/else | 嵌套 `if/else`，同程序 | `(i == 1 ? 10 : 20) + i` 三元 | 对齐。标签在字节码里不存在，不恢复；jadx 三元是非目标 |
| 嵌套 switch（switch 内 switch） | 两层 `lookupswitch` | 两层完整写出 | 同样 | 对齐 |
| 语句位三元 `r += (i<2) ? a[i] : b[0]`（循环内） | 循环测试含数组读取 | 整段引用 | `while` + `+=` + 三元 | 已有 2b（数组读取）；`+=` 与三元都是非目标 |
| `do { } while (n > 0 && r < 100)` | 闩锁是两条件分支链 | 整段引用 | `do…while(true)+if break`（不还原复合条件） | 失败。新的 3.3：复合条件在循环测试位 |

## 第五十六批（`Loop`，未入库）

源码在 `/tmp/jarde_syntax56/src/Loop.java`，`javac --release 8 -g:none`。jarde 文本来自当前 debug 二进制，jadx 已实测。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `while (a > 0 && b > 0)` | 头部两分支链，每个假边都到出口 | **整段引用**：test/exit/latch 不可证 | `while (true) { if (i <= 0 \|\| i2 <= 0) break; … }`——条件取反（De Morgan），不还原复合条件 | 失败。3.3 的主场景。jadx 也不还原；恢复复合条件即反超 |
| `while (a > 100 \|\| n > 5)` | 头部两分支链，每个真边都进体 | 整段引用 | `while (true) + if (…&&…) return`——结构改写为提前返回 | 同上 3.3。jadx 的结构改写比取反更失真 |

## 第五十七批（`Over`，未入库）

源码在 `/tmp/jarde_syntax57/src/Over.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| 重载按参数个数（`argCount(int)` / `argCount(int,int)`） | 池描述符区分 | 调用点裸名 `argCount(arg0)` / `argCount(arg0, arg0)` 均正确 | 对齐。参数本身消歧，不需要描述符 |
| 重载按引用类型（`pick(String)` / `pick(Object)`） | 描述符区分 | `pick(arg0) + String.valueOf(pick(arg1))` 正确 | 对齐 |
| 重载按数值类型（`plain(int,int)` / `plain(long,long)`） | 描述符 + `i2l` | 调用点引用 | 已有 2c.8/2c.2（转换指令），调用解析本身无碍 |
| `plain(long,long)` 内 `(int) (a + b)` | `l2i` | 引用 | 已有 2c.2/2c.8 |

重载结论：调用点文本不需要描述符——字节码池描述符已消歧，参数拼写自然区分；只有桥方法的返回类型擦除场景（bridge@1 已验收）才需要描述符。本批不新增任务。

## 第五十八批（`Pair`/`Re`，未入库）

源码在 `/tmp/jarde_syntax58/src/`，`javac --release 8 -g:none`。jarde 文本来自当前 debug 二进制，jadx 已实测。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| 静态嵌套类按类呈现 | `Pair$Box` 独立 class，无捕获字段 | 字段、构造器（`super(); this.v = arg1;` 顺序正确）、`get()` 全部对齐；`new Pair$Box(arg0)` 调用对齐 | 同样分文件 | 对齐。静态嵌套与内部类（批次 43）不同：无 `this$0`，无 2.10 问题 |
| `catch (E e) { throw e; }`（重抛） | `aload; athrow` | `try`/`catch` 结构正确，`throw e` 被引用（「bare throw 只在守卫规则认领时写出」） | `throw e;` 完整 | 失败。已有 2c.20 的**最小验证场景**：操作数是局部，无构造。任务验证清单应含此形状 |
| `catch { throw new IllegalStateException(s); }` | 构造链 + `athrow` | 构造链与 `throw` 各自引用 | `throw new java.lang.IllegalStateException(str);` 完整 | 已有 2c.26/27（构造值）+ 2c.20（throw），两者都在途/排队。组合场景记入两任务验收 |
| 循环内条件 `sb.append(...)` | 循环测试含 `arraylength` | 循环整段引用（2b/2b.6 先拦）；`new StringBuilder()` 已写出 | 完整（`for-each` 糖） | 已有 2b 链。`sb.append(x)` 语句位与 `sb.toString()` 返回在循环可证后应自然对齐——append 是普通调用语句。验收 2b 时复查此形状：**StringBuilder 累加器**（跨语句累积、`toString` 收尾）不是 concat@1 的表达式链，不得误收 |

## 第五十九批（`Res`，未入库）

源码在 `/tmp/jarde_syntax59/src/Res.java`，`javac --release 8 -g:none`。jarde 文本来自当前 debug 二进制，jadx 已实测。

场景：`try (InputStream in = new ByteArrayInputStream(data)) { … } catch (IOException e) { … }`——TWR 与用户 `catch` 组合，真实代码最常见的资源用法。

| 工具 | 呈现 | 定性 |
| --- | --- | --- |
| jarde | 整段引用：`BCI 20: the handler's own instruction sequence is not the one this rule proves`（处理器序不匹配） | 失败。四行表里两行是 TWR 合成、两行是用户 `catch`；`guard::examine` 的 twr 拒绝后 typed catch 也没有接住（处理器块序列混合了合成与用户代码） |
| jadx | 展开成手写嵌套 `try/finally` 形状（`new ByteArrayInputStream` 裸建、`close()` 显式调用、`Throwable` 捕获与重抛全部写出来） | jadx 也不还原 `try (...)`，输出比源码长得多且引入源码没有的嵌套 |

结论：**反超点**。目标拼写是 `try (java.io.ByteArrayInputStream local1 = new java.io.ByteArrayInputStream(arg0)) { … } catch (java.io.IOException local2) { … }`——`catches()` 的分组按行序天然支持「TWR 行组 + 用户 catch 行组」并存，落入 2.8（同文件域 `guard.rs`）的自然扩展，不新增机制；记入 2.8 验收清单。`readAll` 的三元/数组部分落 2b/非目标，`new byte[8]` 已写出。

## 第六十批（`Mid`，未入库）

源码在 `/tmp/jarde_syntax60/src/Mid.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| 链式字段写 `m.inner.v = n` | `aload; getfield inner; iload; putfield v` | `arg0.inner.v = arg1;` | 对齐。接收者链上的 `getfield` 已覆盖 |
| 链式读回 `return m.inner.v` | `aload; getfield; getfield; ireturn` | `return arg0.inner.v;` | 对齐 |
| `if (xs.add(n))`（条件里的调用） | `invokeinterface add` 作分支条件 | `if (arg0.add(java.lang.Integer.valueOf(arg1))) { … }` | 对齐。装箱保留（P 精度规则） |
| `m.stat()`（经实例引用调静态） | `aload_0; pop; invokestatic` | `pop` 不在子集，BCI 1 引用；调用本身写出 `return stat();` | 失败。新的 2c.31：`Load; Pop; invokestatic` 是 `expr.stat()` 的唯一拼写；丢弃求值会丢掉 `expr` 为 null 时的 NPE，必须保留限定符 |

## 第六十一批说明

`obj.staticMethod()` 的 `pop` 与 2c.29/2c.30 同属 `build.rs` 会合位一族的拼写缺口，但机制不同：它要的是把被弹出的求值写成调用的限定符，不是省略或改写字面量。jadx 侧未实测，留待该任务实现时补。

## 第六十二批（`Ts`，未入库）

源码在 `/tmp/jarde_syntax62/src/Ts.java`，`javac --release 8 -g:none`。jarde 与 jadx 均实测。`Signature` 属性是本 change 的既有非目标，本批验证擦除呈现是否诚实。

| 场景 | jarde | jadx | 结论 |
| --- | --- | --- | --- |
| 泛型字段 `List<String>` | `java.util.List`（擦除） | `java.util.List<java.lang.String>`（读 Signature 恢复） | 按规格对齐：字节码真相是擦除类型，非目标。**注意**：`<clinit>` 里 `Ts.strings = new java.util.ArrayList();` 已写出——这是 2c.26+27（`new` 消费者）在 WIP 树上的效果，验收该任务时以此形状复查 |
| 类型变量 `<T extends …&…>` 擦除为第一个边界 | `java.io.Serializable pick(...)` | 完整类型变量恢复 | 按规格对齐：擦除即字节码真相（此处甚至比 jadx 的恢复更接近描述符） |
| `Class<?>` 字段 | `java.lang.Class` | `java.lang.Class<?>` | 同上非目标 |
| raw 类型 `List` | `java.util.List arg0` + `arg0.add("x")` 写出 | 同样 | 对齐。BCI 8 的 `pop`（`add` 的返回值丢弃）正是 2c.31 已开的 `pop` 形状族——2c.31 的任务文本只写了 `aload; pop; invokestatic`，需扩到「调用返回值的丢弃 `pop`」：`invokevirtual` 返回非 void 且无人读时，调用写为语句、`pop` 不引用。记入 2c.31 验收 |

## 第六十三批（`Sync2`，未入库）

源码在 `/tmp/jarde_syntax63/src/Sync2.java`，`javac --release 8 -g:none`。jarde 与 jadx 均实测。附注：构造器里 `this.lock = new java.lang.Object();` 已写出——是 2c.26+27 在共享树上的 WIP 效果，该任务验收时以此复查。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| `synchronized` 块内提前 `return`，块后还有语句 | 一次 enter、**多条**正常 `monitorexit`（return 路径与 fall-through 路径各一）+ 处理器 | 整段拒绝：`the monitor is not entered once and left on every path out of the region` | 两个 `return` 都写在括号内（return 释放监视器，语义等价） | 失败。新的 2.12。拒绝诚实（消息准确），但 jadx 还原了 |
| 嵌套 `synchronized` | 两对 enter/exit | 整段拒绝（同一消息） | 两层嵌套完整还原 | 失败。新的 2.13 |

## 第六十四批（`Ex`/`C`/`Hide`/`Sub`，未入库）

源码在 `/tmp/jarde_syntax64/src/`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| catch 子类顺序（`IOException` 先于 `Exception`） | 两行表按序 | 两个子句按表序写出，各带自己的 `local1`（各自处理器的槽 1） | 对齐 |
| 接口常量跨类使用 `C.LIMIT + 1` | javac 编译期常量折叠为 `bipush 43`，无 `getstatic` | `return 43;` | 对齐字节码真相。`getstatic` 不存在，无恢复问题 |
| 静态方法隐藏（`Hide.same` 与 `Sub.same`） | 两次 `invokestatic`，池属主不同 | `Hide.same(arg0) + same(arg0)`——跨类限定、本类裸名 | 对齐（4.4 规则的正确应用） |
| 接口常量声明 `int LIMIT = 42` | `ConstantValue` 属性 | `public static final int LIMIT;` 无初值 | 6.6，已在派的声明切片里 |

## 第六十五批（`Iface`，未入库）

源码在 `/tmp/jarde_syntax65/src/Iface.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| 多接口 `implements A, B` | interfaces 表 | 完整写出 | 对齐 |
| 空方法体 `void run() {}` | 只有 `return` | `return;` | 对齐（既有规则：不删空方法 return 匹配 jadx） |
| `return this` | `aload_0; areturn` | `return this;` | 对齐 |
| `return new Runnable[] { this, this }` | `anewarray; dup; dup; aastore…` 初始化链 | 引用（`anewarray` 在 2b；`aastore` 初始化链在 2b.7） | 已有链。注意 `this` 作元素——2b.7 落地时元素是任何可渲染值，含 `this`，无特例 |

## 第六十六批（`Num2`，未入库）

源码在 `/tmp/jarde_syntax66/src/Num2.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。全部落在已有任务，不新增。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| `long a / b`、`int a / b` | `ldiv`/`idiv` | `return arg0 / arg2;` / `return arg0 / arg1;` | 对齐。双槽参数命名正确 |
| `int + long * 2`（`i2l`） | `i2l` | 引用 | 已有 2c.8 |
| `a >>> 3`（long） | `lushr` | 引用 | 已有 2c.1（long 移位） |
| `n * 1.5`（double 字面量） | `i2d; ldc2_w` | 引用 | 已有 2c.5 + 2c.8 |
| `f * 2.0f`（float 字面量） | `ldc` float | 引用 | 已有 2c.5 |

确认：长整算术族（`ldiv`/`lmul`/`ladd` 在 `0x60..=0x73` 子集内）已覆盖，卡点集中在字面量解码（2c.5）与转换（2c.8）——两任务落地后整族数值方法解封。

## 第六十七批（`IUtil`/`UseI`，未入库）

源码在 `/tmp/jarde_syntax67/src/`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| 接口静态方法声明与体 | `0x0109`（public static） | `public static int twice(…)` + 体 `return arg0 * 2;` | 对齐（6.5 边界：static 不加 `default`，主工作区现有装配已正确） |
| 跨类调用接口静态 `IUtil.twice(n)` | `invokestatic` 属主 `IUtil` | `return IUtil.twice(arg0);` | 对齐（4.4 限定规则） |
| `Integer.MAX_VALUE` | 编译期折叠 `ldc 2147483647` | `return 2147483647;` | 对齐字节码真相（常量折叠不产生恢复问题） |
| `Long.MIN_VALUE` | `ldc2_w` 消极值 | `return -9223372036854775808L;` | 对齐。`L` 后缀与负值拼写正确 |

## 第六十八批（`Bits`，未入库）

源码在 `/tmp/jarde_syntax68/src/Bits.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| 字节解包循环 `(v << 8) \| (b[i] & 0xff)` | `ishl/ior/iand` + `baload` | 循环结构对齐（while/自增），体内 2c.1 + 2b 引用 | 已有链：2c.1（位运算）+ 2b（`baload`）。两任务落地后整族解封 |
| `sb.append((char) (s.charAt(i) ^ 0x20))` | `i2c` + `ixor` + `iinc` | `new StringBuilder()` 写出；循环被测试块里的 `s.length()`（`invokevirtual`）拒 | **组合观察**：2c.6（测试块允许调用）+ 2c.2（`i2c`）+ 2c.1（`ixor`）三个已有任务的叠加；StringBuilder 累加器边界（批次 58）适用。无新任务，记为三任务验收的组合复查形状 |
| `s.codePointAt(0)` / `s.endsWith(".class")` | 普通调用 | 完整写出 | 对齐 |

结论：恶意软件字符串解码族（XOR 循环、字节解包）的卡点完全收敛到 2c.1/2c.2/2c.5/2c.6/2b 五个已在队列的任务，无新机制需求。

## 第六十九批（`Init`，未入库）

源码在 `/tmp/jarde_syntax69/src/Init.java`（`javac --release 8 -g:none`，初版 `this(b)` 引用实例字段被 javac 拒绝后改用静态常量）。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| 实例字段初始化器（`a = 1`、`b = a + 1`） | javac 并入每个 `<init>` 首部 | `this.a = 1; this.b = this.a + 1;` 在构造器里按序写出 | 对齐字节码真相。不上提回字段声明（实例初始化器无数码自述来源） |
| 静态字段初始化器（`s1 = 2`、`s2 = s1 * 3`） | `<clinit>` | `Init.s1 = 2; Init.s2 = Init.s1 * 3;` 按序写出 | 对齐。只有 `ConstantValue` 属性的才上提（6.6）；计算型初始化留在 `<clinit>` |
| `this(ZERO)` 构造器链 | `invokestatic`? 否——`getstatic` 后 `invokespecial <init>` | `this(Init.ZERO);` | 对齐（init@1 + 4.4 限定） |
| `this.a = a + c`（字段自引用更新） | `aload; getfield a; iload; iadd; putfield` | `this.a = this.a + arg1;` | 对齐。不经 `dup`，field@1 直写 |

## 第七十批（`Grid`，未入库）

源码在 `/tmp/jarde_syntax70/src/Grid.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。**隔离批**：纯局部变量嵌套循环，无数组依赖——此前带标签循环批次（43/48）的失败都混着 `arraylength`（2b.6）阻塞，本批证明 3.1 自身就拦在嵌套循环上。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| 内层 `break`（无标签） | 内层出口边 | 整段引用：外层 loop 的 test/exit/latch 不可证 | 3.1 的场景集（`nestedBreak`）。内层汇合在外层循环内部，`header_tested_loop` 丢后继 → `LoopLeavesEarly` 族 |
| `continue outer` | 内层跳外层头 | 同上 | 3.1（`labeledContinue`）：目标是包围循环的头 → 带标签 `continue` |
| `break outer` + 外层尾语句 | 内层跳外层出口 | 同上 | 3.1（`labeledBreak`）：目标是包围循环的出口 → 带标签 `break`；外层尾部的 `total += 10` 在外层体内 |

结论：嵌套循环族（含无标签 `break`）的根因完全收敛到 3.1 的循环体接续规则，不需要新任务。3.1 落地时以本批三个方法作 fixture 候选（纯局部、无数组噪声）。前缀声明（`int local1 = 0;`）在引用前写出是既有块内行为，与 1.1/1.2 无关。

## 第七十一批（`Side`，未入库）

源码在 `/tmp/jarde_syntax71/src/Side.java`（`javac --release 8 -g:none`；初版 `return r ? count : -1` 因 int 不能转 boolean 被 javac 拒，改写为 if）。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| 三段 concat `a + b + c` / 混型 `s + n + "x"` | StringBuilder 链 | 完整写出 | 对齐（concat@1） |
| 编译期常量折叠 `"he" + "llo"` | `ldc "hello"`（池里已是合并串） | `local0 = "hello"` 写出，后续 `== "hello"` 比较对齐 | 对齐字节码真相 |
| 字面量初始化的局部声明类型 | `ldc` String 存入槽 | **`Object local0 = "hello";`**——合法（Object 到 String 的 `==` 合法）但类型不精确，源码是 `String a` | 小缺口：无 debug 信息时局部声明类型取自帧类型，`ldc` 字符串的帧类型是泛引用。改进在 build.rs 声明拼写（用存储值的帧类型），非新机制。记为观察项，暂不开任务 |
| `n > 0 && bump()`（短路副作用） | 嵌套分支，`bump()` 在内层测试块 | 嵌套 `if` 写出；`local1 != 0`（2c.17 已知）+ BCI 19 `re-entered` 误诊（1.1/1.2 复查项，与批次 53 `both` 同形） | 已知任务，无新增 |

## 第七十二批（`Cfg2`，未入库）

源码在 `/tmp/jarde_syntax72/src/Cfg2.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| 声明链 + 调用 + 返回（`x=5; y=x*2; return y+unknown(n)`） | 直线码，异常表为空 | 三条语句完整按序写出 | 对齐 |
| `n / 0`（运行时必抛、无 try 保护） | `idiv`，无异常表 | `return arg0 / 0;` 写出 | 对齐——无表即无结构，`idiv` 本身在算术子集 |
| 存储后调用再返回存储值 | `bipush; istore; iload; invokestatic …` | `int local1 = 7; someCall(local1); return local1;` | 对齐。调用不改写槽位时存储值存活正确 |

结论：直线码 + 存储存活的组合在本层无缺口。`idiv`-by-zero 说明「必抛但未保护」不构成结构问题——异常表为空就不需要 try。

## 第七十三批（`Multi`，未入库）

源码在 `/tmp/jarde_syntax73/src/Multi.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。注意：本批体都含能抛的指令（`iadd` 对参数其实不抛，但此处 2.9 已落地的无抛点行也覆盖了），是 2.9 验收后的组合复查。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| 三顺序 catch 子句（子类到父类） | 三个子句按表序、各自 `local1` | 对齐 |
| multi-catch 与普通子句混排（`A|B` 后接 `C`） | `catch (A | B local1)` + `catch (C local1)` | 对齐 |
| try/catch 后接 while 循环 | 两个结构独立写出 | 对齐。try 的 join 正确接住循环 |

结论：异常表多行组合（顺序子句、multi-catch 混排、结构衔接）在 2.1/2.4/2.9 落地后全部对齐，无需新任务。这批同时是 2.9 落地后的正向回归证据。

## 第七十四批（`Cmp2`，未入库）

源码在 `/tmp/jarde_syntax74/src/Cmp2.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| `return a.equals(b)` | 完整写出 | 对齐 |
| `!a.equals(b)` 返回 | 空 `if` + 汇合引用 | 2c.17 布尔取反返回侧（批次 47 已扩任务文本） |
| else-if 链（`equals("x")/equals("y")/isEmpty()`） | 嵌套 `if/else` 完整 | 对齐 |
| `startsWith && endsWith` 返回 | 嵌套 `if` + 汇合引用 + `re-entered` 误诊 | 2c.17 复合条件 + 1.1/1.2 复查形状（与批次 53/71 同族，无新增） |

## 第七十五批（`Anno`，未入库）

源码在 `/tmp/jarde_syntax75/src/Anno.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| `@Deprecated` / `@SuppressWarnings` 运行时注解 | 声明不带注解 | 既有非目标（注解属性与 `Signature` 同族，非本 change 范围）；方法体不受影响。jadx 同样省略（源码层注解需要属性读取，属声明层后续 change） |

## 第七十六批（`Sw`，未入库）

源码在 `/tmp/jarde_syntax76/src/Sw.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| `char` 选择器 + 共享标签（`case 'b': case 'c':`） | `lookupswitch` char 键 | 完整写出，字符字面量键 | 对齐（2c.12 已验收工作的确认） |
| 稀疏负键（`-2000000`/`1000000`） | `lookupswitch` | 键与顺序正确 | 对齐 |
| `case 1` 穿透到 `case 2`（`r = r + 1` 共享） | 无 `goto` 直落 | 整段引用：`SwitchArmsOverlap` + `re-entered` + 未覆盖块 | 失败。已有 2c.4（穿透）。本批给出干净的复现与引用结构；1.1/1.2 落地后前缀 `int local1 = 0;` 已是语句（本批文本可见），2c.4 落地时以此 `fall` 为 fixture 候选 |

## 第七十七批（`CharArr`，未入库）

源码在 `/tmp/jarde_syntax77/src/CharArr.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| `new String(char[])` 构造 | `return new java.lang.String(arg0);` | 对齐（2c.27 的参数位消费者已生效） |
| `char[]` 循环求和 | 循环被测试块 `arraylength`（2b.6）拒 | 已有链 |
| `s.toCharArray()` + `cs[k]` | 调用写出，`caload`（2b）引用 | 已有链 |
| 位掩码链 `& 0x0f \| 0x30 ^ 0x20` | 三条 `iand/ior/ixor`（2c.1）各自引用；尾 `return arg0;` 保留 | 已有任务；前缀保留（1.1/1.2）使尾返回正确存活 |

## 第七十八批（`Try78`，未入库）

源码在 `/tmp/jarde_syntax78/src/Try78.java`，`javac --release 8 -g:none`。jarde 与 jadx 均实测。

| 场景 | 字节码 | jarde | jadx | 结论 |
| --- | --- | --- | --- | --- |
| 循环体是 try/catch（`while { try { r+=n } catch { r=-1 } n-- }`） | try 行覆盖体一部分，未覆盖循环头 | **整段拒绝：irreducible over [2, 6, 16]** | 完整还原 | 失败。这正是 spec 已写明的 3.1 循环体接续场景（`caught` 的形状，design 2p 原文引例）；`uncovered` 扫描把处理器块算成活块导致不可约判定。3.1 排队中 |
| `while(true) { try { if (n<0) break; n-- } catch { return -1 } }` | try 在循环内 + break | 引用（异常边 + 未覆盖） | 改写成 `while (i >= 0)`（`if`/`break` 被消除——结构改写，非等价拼写） | 同上 3.1 + 2c.20（catch 的 return）。jadx 的循环条件改写是结构失真，不跟随 |

结论：两例都收敛到 3.1（`build.rs` 队列）与 2c.20；`loopTry` 的 irreducible 判定根因（异常处理器块参与 reducibility）是 3.1 派发提示里要写明的机制点。无新任务。

## 第七十九批（`Stat`，未入库）

源码在 `/tmp/jarde_syntax79/src/Stat.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。附注：`Stat.LOCK = new java.lang.Object();`（`<clinit>` 里）是 2c.26 在树上的效果。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| `synchronized (静态final字段)` + return（2.6 形状） | `getstatic; dup; astore; monitorenter…` | `synchronized (Stat.LOCK) { return Stat.state; }` 完整 | 对齐（2.6 已验收工作的 static-lock 确认；锁定表达式带属主） |
| `synchronized (X.class)`（类字面量锁） | `ldc Class` 做 lock | 引用：`ldc` Class 是 2c.5 的池项 | 已有链：2c.5（`Class` 字面量）落地后 `lock_expr` 即可渲染，**无需改 monitor()**——锁定表达式走同一值渲染路径。记为 2c.5 的验收复查形状 |
| 同一锁嵌套 `synchronized (LOCK) { synchronized (LOCK) … }` | 两对 enter/exit（重入） | 拒绝：`not entered once` | 已有 2.13（嵌套 monitor）。重入情形（同一对象两层）是 2.13 的变体，验收时补 |

## 第八十批（`Ctor`，未入库）

源码在 `/tmp/jarde_syntax80/src/Ctor.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| 构造器链 `this(0)` + 后续副作用（`a = a + 1`） | `this(0); this.a = this.a + 1;` | 对齐。委托后语句存活 |
| 构造器 `super(); this.a = arg1; count = count + 1;` | 三条按序完整 | 对齐（含静态字段更新带属主） |
| 静态方法读写静态字段 `count = count + 10; return count;` | `Ctor.count = Ctor.count + 10; return Ctor.count;` | 对齐 |

结论：构造器链与静态字段族无缺口。

## 第八十一批（`Abs`/`Abs$Shape2`/`Abs$Sq`，未入库）

源码在 `/tmp/jarde_syntax81/src/Abs.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| 抽象类分文件呈现（`abstract class Abs$Shape2`、抽象方法无体标记） | 完整 | 对齐 |
| 多态调用 `sh.area()`（循环内） | `local6.area()` 写出（调用本身无碍） | 循环受阻于 2b（数组）；**调用已可写** |
| `area() * 2`（double 字面量） | `dconst_2`（2c.5）引用 | 已有任务 |
| `instanceof` 守卫 + `(Sq) sh` 下转 | 2c.9 引用 | 已有任务 |
| 子类构造器链（`super(); this.s = arg1;`） | 完整 | 对齐 |

结论：抽象/多态族无新机制需求；卡点仍是 2b/2c.5/2c.9 三个已排队任务。

## 第八十二批（`If82`，未入库）

源码在 `/tmp/jarde_syntax82/src/If82.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| 空 then 臂（`if (n==1) {}`） | 保留空 `if` 块（else 里的 else-if 结构完整） | 对齐——空臂是源码事实，不删除不填充 |
| 空 else 臂（`if (n==3) {…} else {}`） | else 省略，单臂 `if` | 对齐 |
| 三层嵌套 `&&` 形条件（值位） | 嵌套 `if` 完整 | 对齐（值位不折叠 `&&`，非目标） |
| else-if 阶梯（最后一个分支是取反条件） | 嵌套 `if/else`，四个 return 各在位 | 对齐 |

## 第八十三批（`Stat83` 族，未入库）

源码在 `/tmp/jarde_syntax83/src/Stat83.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| 匿名类作调用参数 `register(new Handler() {…})` | `register(new Stat83$1());` | 对齐（2c.27 参数位 + 按类呈现；5.3 内联前的诚实状态） |
| 匿名类作静态字段初始化器 | `<clinit>` 里 `Stat83.field = new Stat83$2();` | 对齐（2c.26 字段位 + 按类呈现） |
| 字段类型是嵌套接口 `Stat83$Handler` | 声明与参数正确 | 对齐 |
| 调用匿名实例方法 `field.handle("x")` | `Stat83.field.handle("x")` | 对齐 |

结论：匿名类四组合（参数位/字段位/接口类型/方法调用）在 2c.26+27 落地后全部对齐；`new Stat83$N()` 的裸构造（无捕获）与带参捕获（批次 48 `new Local$1Helper(this, arg1)`）都已覆盖。5.1/5.2/5.3（内联方向）仍排队，但按类呈现本身已完整。

## 第八十四批（`E84`，未入库）

源码在 `/tmp/jarde_syntax84/src/E84.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| 递归调用在 try 体内（`return level(n-1)`） | `try { return level(arg0 - 1); } catch (StackOverflowError …)` 完整 | 对齐——2.7（try 体调用写出）+ 递归调用的组合 |
| `Error` 与 `Exception` 混排 catch | 两个子句按表序、各自槽名 | 对齐 |
| 前置守卫 `if (n==0) return 0;` + else 包 try | 嵌套结构完整 | 对齐 |

## 第八十五批（`Edge`，未入库）

源码在 `/tmp/jarde_syntax85/src/Edge.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| `0xDEADBEEF` 十六进制字面量 | `ldc -559038737`（池里是十进制语义值） | `return -559038737;` | 对齐字节码真相——进制拼写不是字节码事实（池存位型），十进制是唯一无歧义拼写。jadx 写 `TransmissionCheck` 一类源码级十六进制是猜（它也无从知道），我们的十进制更诚实 |
| `-2147483648`（`Integer.MIN_VALUE` 字面量） | `ldc` 单指令 | 正确 | 对齐——负值不是 `ineg`（范围外不可表示），池直接存 |
| `0x7FFF…FL` long 十六进制 | `ldc2_w` | `return 9223372036854775807L;` | 对齐 |
| `-n`（运行时取反） | `ineg` | 引用 | 已有 2c.11（一元负号） |
| `Integer.MAX_VALUE + 1`（编译期回绕） | 折叠为 `ldc -2147483648` | `return -2147483648;` | 对齐字节码真相——回绕已发生，十进制拼写忠实 |

## 第八十六批（`Str86`，未入库）

源码在 `/tmp/jarde_syntax86/src/Str86.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| 转义串（`\t`/`\n`/`\"`/`\\`） | 逐字符还原 | 对齐 |
| Unicode 字面量（`caf\u00e9 \u4e2d\u6587`） | **按标量字面写出 `café 中文`** | 对齐——6.3 落地的直接收益 |
| 空串 | `""` | 对齐 |
| `a\u0000b`（串中 NUL） | `a\u0000b` | 对齐——6.4 MUTF-8 + 转义链的收益 |
| `'\u4e2d'`（非 ASCII char） | 字面字符 `'\u4e2d'` | 对齐（escape_unit 对非 ASCII 不转义） |
| `'\''`（引号字符） | `'\''` | 对齐 |

结论：字符串/字符字面量边界族（6.3/6.4 已验收工作）确认完整覆盖。

## 第八十七批（`Fmt`，未入库）

源码在 `/tmp/jarde_syntax87/src/Fmt.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| `Integer.parseInt(s, 16)` | `return java.lang.Integer.parseInt(arg0, 16);` | 对齐（4.4 限定 + 双参调用） |
| `new StringBuilder(s).reverse().toString()` | 完整链式写出 | 对齐——2c.27 参数位构造 + 值链方法链 |
| StringBuilder 累加器循环（`sb.append(s)` 循环 + `toString` 收尾） | `local2.append(arg0);` 语句 + `return local2.toString();` | 对齐——批次 58 记录的累加器边界确认：作为普通语句呈现，未被 concat@1 的表达式链误收 |
| `String.format("%x", n)` / `format("%04d", n)`（varargs） | 装箱调用写出，数组链引用 | 已有 2b.7（varargs 初始化链） |

结论：格式化/字符串构建族无新机制需求。

## 第八十八批（`Inc`，未入库）

源码在 `/tmp/jarde_syntax88/src/Inc.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| `int m = --n; return m;`（前缀减） | `iinc` 前置，新值绑定 | `arg0 = arg0 - 1; int local1 = arg0; return local1;` | 对齐（2c.3 前缀侧落地前按加后存储拼写，同程序） |
| `int m = n++; return m + n;`（后缀增，旧值存） | `iload` 旧值 + `iinc` | 存储写出 + 旧值引用，拒绝理由准确（槽名会读到新值） | 失败。已有 2c.3（`local++`）的**存储侧**形状；`m + n` 中的 `m` 是旧值 |
| 混合 `n++/n--/++n` 三连 | 三个旧/新值交错 | 存储链保留、旧值各自引用 | 2c.3 的组合验收形状（旧新值交错）。`return local1 + local2 + local3` 里未声明局部是引用边界的诚实呈现，2c.3 落地消除 |
| `n \|= 0x10; n &= ~0x0f; n ^= 0x55` | `ior/iand/ixor` + `iconst_m1` | 位链引用 | 已有 2c.1（含 `~` 即 `ixor -1` 的非目标边界） |

## 第八十九批（`Obj`，未入库）

源码在 `/tmp/jarde_syntax89/src/Obj.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制（共享树上有 2c.29+30+31 接管 coder 的 WIP）。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| `Objects.requireNonNull(s)`（返回值被丢弃的 `pop`） | `java.util.Objects.requireNonNull(arg0);` 写出、**无 pop 引用** | 2c.31(b) 的实例。注意：可能是接管 coder 在共享树上的 WIP 效果，**不作为已验收结论**；2c.31 验收时以此形状复查 |
| `Objects.equals` / `Objects.hashCode`（作值） | 完整写出 | 对齐（4.4 限定 + 调用值） |
| `new int[n][n]`（`multianewarray`） | 引用 | 已有任务 2b.3（二维创建） |
| `new int[n][]`（部分维度） | 引用 | 同上 2b.3 |

## 第九十批（`Dense`/`Shadow`，未入库）

源码在 `/tmp/jarde_syntax90/src/`，`javac --release 8 -g:none`。文本来自当前 debug 二进制（树上有 2c.29+30+31 接管 coder 的 WIP）。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| 密集 `tableswitch`（case 0–4 连续） | 六臂完整、键值正确 | 对齐 |
| 负基数 `tableswitch`（case -2..0） | 键 `-2/-1/0` 正确 | 对齐 |
| 参数遮蔽字段（`int v` 参数 vs `this.v`） | `return arg1 + this.v;` | 对齐——槽位与字段读取各归其位 |
| 局部块遮蔽字段 | `int local1 = 10; return local1 + this.v;` | 对齐 |

## Behinder 抽查（原目标工件，当前 debug 二进制，树上有 2c.29+30+31 WIP）

方法：从 `/tmp/Behinder_v4.1.t00ls.zip` 内层 `Behinder.jar` 提取两个核心类，单类呈现（整 jar 132MB 超过 `input_bytes` 64MB 默认预算且该维度 CLI 不可覆盖——产品级注记，非本 change 范围）。

| 类 | 成员数 | 整段拒绝 | 局部引用行 | 备注 |
| --- | --- | --- | --- | --- |
| `net.rebeyond.behinder.core.Crypt` | 13 | **0** | 78 | `Encrypt/Decrypt/DecryptForJava/DecryptForNative` 均产出可读语句：`byte[] raw = key.getBytes("utf-8");`、`cipher.doFinal(bs)`；声明带 `throws java.lang.Exception`（6.2 接线在真实工件生效）；局部引用集中在数组/位运算族（2b/2c.1 排队任务） |
| `net.rebeyond.behinder.utils.Utils` | 87 | 3 | 266 | 3.4% 整段拒绝率 |

结论：相对早期「大量整方法引用」的基线，核心加密类已到零整段拒绝；剩余缺口与排队任务族一致。

## 第九十一批（`Multi`/`Shape91`，未入库）

源码在 `/tmp/jarde_syntax91/src/`（Multi 用 `--release 8`，sealed 用 `--release 17`）。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| 多个静态初始化块 | javac 合并为一个 `<clinit>`（源序） | `Multi.a = 1; Multi.b = 2;` 按序 | 对齐字节码真相——块边界在字节码里不存在，不恢复 |
| 多个实例初始化块 | 并入每个 `<init>`（源序） | `this.x = 10; this.y = 20;` | 对齐 |
| sealed 接口的 `permits` | `PermittedSubclasses` 属性（reader 已有 `PermittedSubclassesFacts`） | `public interface Shape91 {}`——sealed 丢失；子类 `final … implements Shape91` 正确 | 声明层缺口，与注解/`Signature` 同类（属性拼写未接），**非本 change 范围**，记录为后续 change 候选 |

## 第九十二批（`Cached`，未入库）

源码在 `/tmp/jarde_syntax92/src/Cached.java`，`javac --release 8 -g:none`。文本来自当时 debug 二进制（2c.29+30+31 交卷前后）。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| `n >= -128 && n <= 127` 边界链 | 嵌套 `if` 完整写出 | 对齐（值位不折叠 `&&`，非目标） |
| `Integer.valueOf(n)`（缓存路径）与 `new Integer(n)`（非缓存） | 两条都按精度规则真实写出 | 对齐——装箱调用保留（P 精度规则） |
| `Integer[]` 装箱循环（`intValue()` 拆箱） | `local1 + local5.intValue();` 语句 + 循环受阻于 2b | 已有链：2b（数组）+ intValue 保留正确 |

## 第九十三批（`Sb93`，未入库）

源码在 `/tmp/jarde_syntax93/src/Sb93.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| `sb.insert(at, what)` / `sb.delete(from, to)`（变异器作语句） | `local3.insert(arg1, arg2);` + `toString` 收尾 | 对齐——变异器是普通语句，不误收进 concat@1 |
| `new StringBuilder(s).append(n).append('!').toString()` | 完整链式，**`'!'` 字符字面量** | 对齐——2c.19（append(char)）收益可见 |
| `s == null \|\| s.isEmpty()`（引用比较+调用复合） | 嵌套 `if` 极性正确（`!= null` 外层、`isEmpty` 内层反转） | 对齐 |
| `substring(0,1).toUpperCase() + substring(1)` | 完整 | 对齐 |

## 第九十四批（`Codec`，未入库）

源码在 `/tmp/jarde_syntax94/src/Codec.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。rot13 完整解码循环（恶意软件高频形态）。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| `for` + `s.length()` 测试 + `charAt` + 范围判断 + `(char)` 转换 + `sb.append(c)` | 循环整段引用：测试块的 `invokevirtual length`（2c.6）先拦 | 卡点顺序确认：2c.6（测试块允许调用）→ 2c.2（`i2c`）→ 2c.1（`irem` 等已在子集）。**2c.6 是这个族的第一闸门**——`rot13` 体本身的指令（`i2c` 一处外）几乎都已可写。jadx 对照未跑，本批聚焦卡点顺序 |

结论：解码循环族的主闸门收敛到 2c.6（循环测试块的方法调用），与 2b.6（arraylength）并列为循环族的两个前置。

## 第九十五批（`Obf`，未入库）

源码在 `/tmp/jarde_syntax95/src/Obf.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制（树上有 2b 数组 coder 的 WIP）。经典混淆三形态。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| 回文双指针循环（`while (i<j)` + 提前 `return false`） | 前缀两条语句保留；循环被 `LoopLeavesEarly`（3.1）拒 | 已有 3.1——提前返回的双指针循环是其核心场景（设计 2p 一族） |
| `new String(char[])` | 完整 | 对齐 |
| `char[]` 重建循环（`cs[i] = s.charAt(i)`） | `arg0.length();` 语句（2c.31(b) 的 WIP 效果，验收时复查）+ `newarray`（2b）+ 循环（2c.6） | 已有链：2b + 2c.6 |
| 位旋转 `(v << n) \| (v >>> (32-n))` | `ishl/iushr/ior`（2c.1）引用 | 已有任务 |

结论：混淆三形态全部收敛到已排队任务（3.1、2b、2c.6、2c.1），无新机制需求——巡查对“剩余缺口=排队任务”的收敛判断再次确认。

## 第九十六批（`Lazy`，未入库）

源码在 `/tmp/jarde_syntax96/src/Lazy.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| `volatile` 字段声明与静态读写 | `getstatic/putstatic` | `static volatile Lazy$Holder h;`、`local0 = Lazy.h;`、`Lazy.h = local0;` | 对齐 |
| 惰性初始化 + null 守卫（局部缓存字段） | 局部赋值、`ifnonnull`、双写 | 结构完整，`new Holder()`（2c.26 收益）、双写都写出 | 对齐到尾部字段读 |
| **尾读 `local0.v`（经局部别名读字段）** | `aload; getfield Lazy$Holder.v` | 引用：「field access … is not one this run proved names the member its own receiver's type declares」 | **新发现**：字段读的接收者证明拒绝了**局部别名**——`local0` 的帧类型是 `Lazy$Holder`（`getstatic` 结果的 SSA 类型），但 field@1 的接收者判定似乎只认 `this` 或字段声明可见性直接可证的形状。诊断在案：需要读 field@1 的接收者判定代码才能定论是类型传播缺失还是规则过窄 |
| null 替代守卫 `if (s == null) s = "";` | `ifnonnull` + 双写 | 完整 | 对齐 |

## 第九十七批（`Cast2`，未入库）

源码在 `/tmp/jarde_syntax97/src/Cast2.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制（树上 2b 数组 coder 的 WIP 在跑）。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| 混合 `Object[] {…}` 初始化（含装箱、null、内嵌数组） | 装箱调用写出为语句（2c.31(b) WIP 效果），初始化链引用 | 已有链：2b.7（初始化链）+ 2b（内嵌 `new int[]`）。混合元素的引用粒度诚实 |
| null 守卫 + `!isEmpty()` 的 for-each 循环 | 循环被 `LoopLeavesEarly`（提前 `return s`）拒 | 已有 3.1（双指针/null 守卫提前返回同族）+ 2c.6/2b.6 |
| 引用三元 `f ? a : b`（引用类型 phi） | 空 `if` + 汇合 phi 引用 | 按规格正确拒绝：不写三元、不发明局部。与 int 三元（批次 35）一致 |

结论：无新任务。`mixed()` 的形状记为 2b.7 验收的组合复查项（装箱/null/嵌套数组三种元素）。

## 第九十八批（`Timer`，未入库）

源码在 `/tmp/jarde_syntax98/src/Timer.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| `if (n < 2) return n;`（int→long 返回） | `i2l`（2c.8）在返回位 | else 臂完整；then 臂 `i2l` 引用 | 已有 2c.8——递归 long 方法的**最常见守卫臂**（`return n` 从 int 参数），2c.8 落地即解封 |
| `Long cached = cache.get(n)` | 装箱 + `checkcast` | `arg1.get(Integer.valueOf(arg0));` 语句写出（2c.31(b) 收益），`checkcast`（2c.9）引用 | 已有链：2c.9 |
| `cached != null` null 守卫 | 引用比较 | `local2 != null` 写出（**局部名正确**——2c.32 的帧槽类型正是这个局部） | 对齐；`return local2.longValue()` 拆箱保留正确 |
| 三元 `n < 2 ? n : memo(...)+memo(...)` | 双递归算术 + phi | 按规格拒绝（不写三元）；`if_icmpge`/`i2l` 引用 | 已有 2c.8 + 非目标 |

结论：备忘录族全落已有链（2c.8/2c.9/2c.31 已生效部分），无新任务。

## 第九十九批（`Enum99`，未入库）

源码在 `/tmp/jarde_syntax99/src/Enum99.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。注意：本源码用了 `switch (this)`，javac 8 编译成 `ordinal()` switch（无合成表——合成 `$SwitchMap` 只出现在**外部类** switch 枚举时；本批的 `apply` 在枚举自身内，字节码即 `switch (this.ordinal())`）。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| 枚举方法 `apply`（含 `switch (this)` 语义） | `switch (this.ordinal()) { case 0: … default: … }` 完整 | 对齐字节码真相（`ordinal()` 是字节码事实，不猜 `switch (this)`——那是源码级猜测） |
| 跨类枚举方法调用 `op.apply(a, b)` | 完整 | 对齐 |
| `op.name() + ":" + op.ordinal()`（枚举内建 + concat） | 完整 | 对齐（concat@1 与 int→String 混拼） |
| `values()`/`valueOf`（合成成员） | `clone()` 语句 + `checkcast` 引用 | 已有 2c.9（与批次 33 一致） |
| 枚举常量声明 | 字段 + `<clinit>`（2c.26 已写出 `new`，见批次 33） | 已验收路径 |

结论：枚举自 switch 与方法分发族无新缺口。`ordinal()` 拼写与 jadx 的 `switch (this)` 差异是源码级猜测边界——字节码只说 `ordinal()`。

## 第一百批（`Wrap100`，未入库）

源码在 `/tmp/jarde_syntax100/src/Wrap100.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。异常包装链——C2 恶意软件常见形态。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| catch 内嵌套 try + 链式调用（`re.getMessage().length()`） | 双层表 | **完整对齐**：外层 catch 子句内 `try { return local1.getMessage().length(); } catch (NullPointerException local2) { return 0; }` | 对齐——2.5 嵌套机制在 catch 体内同样工作 |
| 内层 `return parseInt(cmd)` | 调用在 try 体 | 写出（2.7 收益） | 对齐 |
| `throw new IAE("bad: " + cmd, nfe)`（带原因包装） | concat + `new; dup; invokespecial(String,Throwable); athrow` | 内层 try 结构正确；`new` 链与 `athrow` 引用（「bare throw not claimed」） | 失败。已有 2c.20（throw 语句）的**组合验收形状**：操作数是构造链 + concat + 捕获局部三重组合。2c.26/27 已落地但 throw 位的 `dup` 遗留值消费者尚未接线（2c.20 排队中，其派发提示应含本形状） |
| 外层第二 catch（`iae → -1`） | 前向汇合 | `re-entered` 误诊（批次 53/71 同族） | 已留档的误诊族；1.1/1.2 落地后仍在，待专项小任务 |

结论：无新任务。`dispatch` 是 2c.20 的最佳 fixture 候选（真实包装语义：concat 消息 + 捕获原因）。

## 第一百零一批（`Iface101`/`Base101`，未入库）

源码在 `/tmp/jarde_syntax101/src/Iface101.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。接口继承族全链。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| 接口常量 `int SIZE = 16` | `public static final int SIZE = 16;`（**带初值**） | 对齐——6.6 ConstantValue 在接口字段上的收益首次实测 |
| 接口 `default` 方法体 | `public default int doubled() { return this.size() * 2; }` | 对齐——6.5 拼写 + 抽象方法 `this.size()` 分发 |
| 实现类引用接口常量 | `return 16;`（编译期常量内联，无 `getstatic`） | 对齐字节码真相 |
| 实现类调用 default 方法 | `this.doubled()` | 对齐 |
| 经接口参数虚分发 `b.doubled()` | `arg0.doubled()` | 对齐 |

结论：接口继承族（常量/default/实现/分发）全链对齐，无新任务。6.5/6.6 在接口场景的组合收益确认。

## 第一百零二批（`Inner102`/`Inner102$Access`，未入库）

源码在 `/tmp/jarde_syntax102/src/Inner102.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。内部类私有访问（含 `Outer.this` 显式形式）。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| 内部类读外部私有字段 | `Inner102.access$000(this.this$0);`——合成访问器与 `this$0` 按类忠实呈现 | 对齐（P05 诚实中间态：accessor 保留，4.1 才内联；jadx 恢复 `Inner102.this.secret` 是用类层级知识的改写，两键文本等价） |
| `Outer.this.secret`（显式限定形式） | 同一 `access$000(this.this$0)`——字节码与简写形式**完全相同** | 对齐字节码真相——`Outer.this` 在字节码里就是 `this$0`，两种源码拼写无区分依据，均呈现为访问器形式 |
| 构造器捕获顺序 | `this.this$0 = arg1; super();`（2.10 已排队） | 已有任务 |
| `use()` 内 `new Access()` 跨类构造 | 完整 | 对齐 |

结论：无新任务。`viaOuter`/`read` 的同字节码确认了一个拼写边界：显式 `Outer.this` 无法从字节码区分，访问器形式是唯一诚实拼写。

## 第一百零三批（`Mix103`，未入库）

源码在 `/tmp/jarde_syntax103/src/Mix103.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| switch 混合出口（`break` 到汇合 + 臂内 `return`） | case1 `goto 40`、case2 自带 `ireturn`、default 落入 40；40 = `iload_1; ireturn` | case1 臂内联 `return local1`（**join 被先到的臂吞并**），default 臂 `break` 后 join 被引用为 `re-entered`——**default 路径在文本里没有 return，语义不完整** | 失败。新的 2.16。与第 53/71/100 批同根（`visited` 把多臂汇入判重入），本批升级为语义缺口 |
| `do { } while` 内 `continue`（跳到条件） | `irem`（2c.1）+ 回边 | 循环整段引用 | 已有 2c.1 + 3.1（continue 族） |
| 静态嵌套枚举 + `ordinal()` | 常规 | 完整 | 对齐（批次 33/99 已覆盖族） |

## 第一百零四批（`Num104`，未入库）

源码在 `/tmp/jarde_syntax104/src/Num104.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| `a / b + a % b`（除法+取模混算） | `idiv`/`irem` | 完整写出 | 对齐 |
| `long %`（`lrem`） | `lrem` | 完整 | 对齐 |
| `if ((n & flag) != 0)`（位与作条件） | `iand`（2c.1） | 条件处的 `iand` 引用 | 已有 2c.1 |
| `(bits & (1L << i)) != 0`（long 位测试） | `lshl`/`land` | 位运算族引用 + 汇合 phi | 已有 2c.1（long 移位/与）；`!= 0` 的比较返回是 2c.17 族 |

结论：数值边界最后一批全部落已有任务（2c.1/2c.17），数值族巡查闭合——其卡点收敛为 2c.1/2c.5/2c.7/2c.8/2c.11 五个已排队任务。

## 第一百零五批（`Enc`，未入库）

源码在 `/tmp/jarde_syntax105/src/Enc.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制（树上 2b 数组 coder 的 WIP：`local2.length`、`local2[local4]` 已能写出）。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| `s.getBytes("UTF-8")`（charset 重载） | `return arg0.getBytes("UTF-8");` 完整 | 对齐（throws 声明 6.2 正确） |
| `new String(b, off, len, "UTF-8")`（四参构造） | 完整 | 对齐 |
| `new String(b, StandardCharsets.US_ASCII)`（静态字段作参数） | 构造链引用（`getstatic` 的 Dup 遗留值喂构造参数——与 2c.27 同族但值是字段读取） | 失败。**新观察**：2c.26/27 覆盖了 `new` 构造链的 Dup 遗留值，但这里遗留值来自 `getstatic`（字段读），构造参数位读字段值的组合未被覆盖——归入 2c.32（field@1 接收者/值证明的同一族）验收复查形状，不开新任务 |
| hex 格式循环（`%02x`） | 循环结构保留；`int local5 = local2[local4];` 数组读取已写出（2b WIP），`String.format` 的 varargs 链（2b.7）引用 | 已有链：2b + 2b.7；BCI 32 的「array instruction produces a value nothing reads」是 2b WIP 的中间态，验收时核对清除 |

结论：charset 族对齐；静态常量字段作构造参数记入 2c.32 复查；hex 循环是 2b/2b.7 的组合形状。

## 第一百零六批（`Try106`，未入库）

源码在 `/tmp/jarde_syntax106/src/Try106.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制（树上 2.15 WIP）。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- |---| --- |
| 两个顺序 try 各 catch 同类型 | 两行独立表 | 完整对齐（2.9 的 `steps` 已覆盖；本批是 `IllegalArgumentException` 变体） | 对齐 |
| 双层嵌套（内 catch RE + 外 catch Exception，`n*3` 在两 try 之间） | 表：`[0,4)→RE`、`[0,14)→Exception`（**含中间语句**） | 外层 catch 结构正确（`n = -9; return arg0;` 在子句内——外层 catch 直接到 join 的路径被正确读作子句内 return）；**但 `n = n * 3` 之后 join 被引用为 `re-entered`** | 失败。2.16 的同族形状（第三次出现，非 switch 专属）：外层 catch 的处理器块（17–20）与正常路径都到 21，正常路径先走被判重入。`n * 3` 在 try 之后写出（前缀保留正确），join 的 `return` 丢失 |

结论：2.16 的适用面从 switch 扩大到嵌套 try——`visited` 重入误诊是统一根因。fixture 建议合并本批 `deepCatch`（非 switch 变体）。

## 第一百零七批（`Mc107`，未入库）

源码在 `/tmp/jarde_syntax107/src/Mc107.java`（`javac --release 8 -g:none`；初版 `RuntimeException` 先于子类被 javac 拒——「已捕获到异常」，修正后合法），文本来自当前 debug 二进制。multi-catch 深水变体。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| 三类型 wide multi-catch | `catch (A | B | C local1)` 按表序完整 | 对齐 |
| 子类→父类层次顺序（合法源序） | 两个子句按表序 | 对齐 |
| multi-catch 与普通子句混排（子类联合在前、父类在后） | `catch (A | B local1)` + `catch (RuntimeException local1)` | 对齐 |

结论：multi-catch 全变体（宽度/层次/混排）对齐，2.1/2.4 已验收工作在深水区稳定。无新任务。附注：javac 拒绝父类先于子类的 catch 顺序（编译期已捕获检查），呈现层无需处理该非法形状。

## 第一百零八批（`Sync108`，未入库）

源码在 `/tmp/jarde_syntax108/src/Sync108.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。monitor 深水组合。

| 场景 | 字节码 | jarde | 结论 |
| --- | --- | --- | --- |
| `synchronized (this.lock)`（**字段锁**）+ 块内 while + return | `aload; getfield lock; dup; astore; monitorenter…` | 引用：`getfield` 的 Dup 遗留值喂 `monitorenter` 的锁定表达式——与 2c.26/27 同族（构造/字段值的 Dup 消费者）但消费者是 monitor 头 | 失败。**新形状**：monitor 锁定表达式经 `getfield`（phi 合并已修但这里是单值直达）引用 BCI 4/5。批次 79 的 static-lock 对齐（`getstatic` 值），本批是**实例字段** `getfield`——2c.26 的「字段读取值喂 dup 消费者」未覆盖 monitor 位。归入 2.12/2.13 的派发提示（monitor 族统一收尾时一并接线），不开新任务 |
| `synchronized (this)` 内 try/catch + return | 表：用户行 `[4,14)→17` + monitor 自身行 `[4,29)→30 any`（**行交叉**） | 引用：「protecting row does not end where this shape requires」 | 失败。**真 crossing**：monitor 的 any 行 `[4,29)` 覆盖用户行 `[4,14)` 且不同端——`crossing_records` 前置整段拒绝（合理：两形状声明范围真实交叉）。深水组合需要 monitor+catch 的联合证明（2.12 的多退出推广 + 行分区），记录为 2.12 的远期形状，非本轮可派 |

结论：monitor 族两个深水组合（字段锁、monitor×catch 行交叉）都超当前任务覆盖，分别归入 2.12/2.13 的派发提示与远期记录。

## 第一百零九批（`Outer109`/`Outer109$Inner`，未入库）

源码在 `/tmp/jarde_syntax109/src/Outer109.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。嵌套静态类跨访问族。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| 外类访问嵌套静态成员 `Inner.STATIC + Inner.stat()` | `Outer109$Inner.STATIC + Outer109$Inner.stat()`——4.4 限定用内部名（`$` 保留） | 对齐字节码真相（池属主是 `Outer109$Inner`）；源码简写 `Inner.X` 与二进制名拼写同义，不猜源码层导入 |
| `new Outer109.Inner()`（源码限定构造） | `return new Outer109$Inner();` | 对齐（同上，`$` 是字节码事实） |
| 嵌套类内自引用静态 `STATIC` | `return Outer109$Inner.STATIC;`——**同类成员带限定** | 拼写差异记录：4.4 规则是「属主==当前类时裸名」，但该判定用的是**类源名**（装配层 `Outer109`），而这里是嵌套类自身的 `<clinit>` 场景、成员名不冲突时源码也可裸写。呈现忠实、非错误；源码级简化属非目标。记录在案，不改 |
| 嵌套类静态初始化 `<clinit>` | `Outer109$Inner.STATIC = 5;` 写出 | 对齐（计算型初始化留 `<clinit>`，6.6 边界） |

结论：嵌套静态类跨访问族对齐，无新任务。

## 第一百一十批（`Ctor110`，未入库）

源码在 `/tmp/jarde_syntax110/src/Ctor110.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制（树上 2b 数组 coder 的 WIP）。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| 构造器委托链（`()→this(0,0)`、`(int)→this(x,x)`） | 均完整 | 对齐（init@1） |
| **varargs 构造器声明 `Ctor110(int... xs)`** | `Ctor110(int... arg1);`——6.7 在构造器上正确生效 | 对齐（6.7 验收时已含构造器规则） |
| varargs 构造器体（空） | `super(); return;` | 对齐 |
| `this(new int[] { a, b })`（委托参数是数组初始化链） | 引用（`newarray` + 初始化链是 2b/2b.7 的 WIP 形状） | 已有链：2b + 2b.7——BCI 2 的「array instruction … nothing reads」噪声与批次 105 同一 WIP 中间态，2b 验收复查 |
| `new Ctor110(1, 2)`（调用 varargs 构造器） | `return new Ctor110(1, 2);` | 对齐——**字节码就是装箱数组+调用**？否：javac 对 `new Ctor110(1,2)` 发 `newarray; dup; …; invokespecial`——即调用点是 2b.7 的 varargs 装箱形状。此处完整写出说明树上 WIP 已能处理或该形状恰好走了直译。验收 2b.7 时以本方法为复查形状 |

结论：构造器委托/varargs 族对齐或落 2b/2b.7；`make()` 记为 2b.7 的验收复查形状（varargs 调用点）。

## 第一百一十一批（`Cond111`，未入库）

源码在 `/tmp/jarde_syntax111/src/Cond111.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。三元深水：臂是方法调用。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| 三元臂为调用（`len > 3 ? big(s) : small(s)`） | `if/else` 结构完整、两臂调用**作语句写出**（`big(arg0);`）+ 汇合引用 | 已知组合：2.16（join 归结构）——汇合 phi 的 return 被 join 归属问题吃掉。**注意**：臂内调用被写成丢弃返回的语句是 2c.31(b) 与 join 归属的交互中间态；2.16 落地后应为臂内 `return big(arg0);`（两臂各自 return）或 join `return …`。记为 2.16 的验收复查形状 |
| 布尔三元（`n > 0 ? isEven(n) : isOdd(n)`） | 同上结构 + phi 引用 | 同 2.16 |
| `n % 2 == 0` 比较（含 `irem`） | 空 `if` + phi 引用 | 2c.17（`irem` 已在算术子集，比较返回是 2c.17 族） |

结论：三元臂为调用的形状收敛到 2.16 + 2c.17，无新任务。本批三方法都记入各自验收复查清单。

## 第一百一十二批（`Disp`，未入库）

源码在 `/tmp/jarde_syntax112/src/Disp.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。混淆器 dispatch 形状。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| 位状态机（`state << 4 \| (op & 0xf)`） | 前缀/尾语句保留（1.1/1.2 收益），位运算链（2c.1）引用 | 已有任务；静态字段读写正确（`Disp.state = local1; return Disp.state;`——注意 `local1` 未声明是引用边界的诚实呈现） |
| 字符串 switch（`hashCode` 两段式） | **完整对齐**：`switch (hashCode)` + 臂内 `equals` 守卫 + 第二个 `switch` 返回 | 对齐（既有确认，批次 30 同族；第二个 switch 的 `default` 直落正确） |
| 字符串计数循环（`charAt == ' '`） | 循环被 2c.6（测试块 `length()`）拦 | 已有链：2c.6 → 2c.30（`charAt` 比较返回 char 字面量——`(char)` 收窄在比较位） |

结论：无新任务。字符串 switch 双段式在真实 dispatch 形状上稳定。

## 第一百一十三批（`Cmp113`/`Cmp113$1`，未入库）

源码在 `/tmp/jarde_syntax113/src/Cmp113.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。Comparator/桥方法族。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| 匿名 Comparator 常量（`static final`） | `Cmp113.BY_LEN = new Cmp113$1();`（`<clinit>`，2c.26 收益）+ 按类呈现 | 对齐（5.3 前诚实状态） |
| 具体方法 `compare(String,String)` | `return arg1.length() - arg2.length();` | 对齐 |
| **桥方法** `compare(Object,Object)`（erasure 转发） | 引用（两次 `checkcast`——桥@1 只对 `bridge_owns` 的调用点生效，方法体内的 checkcast 是 2c.9） | 已有任务：2c.9；桥方法保留（P05 非目标确认） |
| `new ArrayList<>(xs)`（diamond） | `new java.util.ArrayList(arg0);` | 对齐字节码真相（泛型擦除，`Signature` 非目标） |
| `out.sort(BY_LEN)` + 迭代器循环 | 调用写出；循环被 2c.6 拦（`hasNext` 在测试块） | 已有链：2c.6 |
| `Integer` 装箱循环 | `n = n + x;` 中 `x.intValue()`（装箱值进算术） | 已有链：`intValue` 保留是精度规则 |

结论：无新任务。桥方法体内的 checkcast 与调用点桥拼写是两个不同层的 2c.9 形状，都已在任务内。

## 第一百一十四批（`Lbl114`，未入库）

源码在 `/tmp/jarde_syntax114/src/Lbl114.java`，`javac --release 8 -g:none`。jarde 与 jadx 均实测。

| 场景 | jarde | jadx | 结论 |
| --- | --- | --- | --- |
| 双层循环 + `continue outer` + `break outer` 混用 | 整段引用（`LoopLeavesEarly`——两条带标签边） | `loop0` 标签 + **把 `continue outer` 改写进内层条件 `j != 2`**（结构改写） | 已有 3.1（fixture 已备批次 70 的纯局部版）；jadx 的条件并合是我们明确的非目标。记 3.1 验收复查形状：本方法同一外层既有 continue 又有 break |
| 单层 while 的 `continue outer`（标签冗余） | **完整对齐**：`if (n == 5) {} else { r += n; }`——javac 把单层 continue 编成空 then（无 goto），现有文本就是这个程序 | `if (i != 5) i2 += i;`（条件反转——语义同） | 对齐；与 More19 `skip` 的既有结论一致：**不得改成 `continue`、不得反转条件**。jadx 的反转再次确认是非目标 |

结论：无新任务。3.1 的 fixture 家族补上「同一外层 continue+break 混用」变体；单层标签 continue 的对齐再次确认。

## 第一百一十五批（`Tbl115`，未入库）

源码在 `/tmp/jarde_syntax115/src/Tbl115.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制（树上 2b 数组 WIP 生效）。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| `switch (op.ordinal())` + 共享标签（`case 0: case 1:`）+ 三类出口 | 完整对齐：共享标签、`n+1`/`n*2`/default | 对齐（批次 99 的 ordinal 真相 + 批次 76 共享标签组合稳定） |
| 枚举数组 for-each（`for (Op op : ops)`）+ 循环内调用 | `local3 = local2.length; while (local4 < local3) { … eval(local5, local1); }`——**数组 for-each 的下标展开完整对齐**（2b WIP 的 `.length`/元素读取已能写出）；`Object local5` 声明是引用边界呈现 | 对齐为下标 while（不猜 for-each，非目标）；循环+调用组合完整 |

结论：无新任务。枚举 ordinal switch 与数组 for-each 的组合在 2b WIP 树上完整呈现——2b 验收复查清单再添一形状（`viaTable`）。

## Behinder 第二次抽查（2.14/2.15/2c.29+30+31/6.5/6.6 之后）

同一批类文件、同一单类入口。与第一次抽查（2c.26+27 刚落地时）对比：

| 类 | 第一次 | 第二次 | 变化 |
| --- | --- | --- | --- |
| `Crypt` | 13 成员 0 整段拒绝、78 局部引用行 | 13 成员 0 整段拒绝、76 行 | 局部引用 −2（2c.29 去掉发明 cast 一族） |
| `Utils` | 87 成员 3 整段拒绝 | 87 成员 3 整段拒绝 | 持平（3 个拒绝在数组/循环族——2b 在途） |

`Decrypt` 主分派方法体复查：`type.equals("php")` 嵌套分派 + try/catch（`e.printStackTrace()` 完整）——真实分派结构完整呈现。剩余局部引用仍集中在数组/位运算/浮点族（全部对应排队任务）。

结论：整段拒绝率维持零（核心类），局部引用随会合位拼写批小幅下降；大的解锁仍系于 2b（数组）与 2c.6（循环测试调用）——与巡查收敛画像一致。

## 第一百一十六批（`Asrt`，未入库）

源码在 `/tmp/jarde_syntax116/src/Asrt.java`，`javac --release 8 -g:none`。文本来自当前 debug 二进制。完整 `assert` 形态（带消息与不带）。

| 场景 | jarde | 结论 |
| --- | --- | --- |
| `assert n > 0 : "…" + n`（带消息） | 结构骨架完整：`if (!$assertionsDisabled) { if (n <= 0) { … } }` + 前缀/尾语句保留；体内三处引用（构造链、concat 的 Dup、`throw`） | 已有链：2c.20（throw）+ 2c.26/27（throw new 的 Dup 消费者）——`throw new AssertionError(msg)` 是这两任务的组合形状。不拼写 `assert`（既定非目标）。`$assertionsDisabled` 字段与 `<clinit>` 的 `desiredAssertionStatus()` 呈现正确 |
| `assert o != null`（无消息） | 同骨架，`new AssertionError()` 无参构造引用 | 同上 |
| `<clinit>` 的 `$assertionsDisabled` 初始化 | 引用（`ldc Class` + `invokestatic`——2c.5 的 Class 字面量） | 已有链 |

结论：无新任务。`assert` 的完整形态 = 2c.20 + 2c.27 + 2c.5 三任务组合，骨架（守卫 if + 条件反转）已正确。

## 数组切片（2b.1/2/3/5）验收复查形状（架构师复测）

- 批次 95 `toChars`：`char[] local1 = new char[arg0.length()];`——**创建+类型声明完整**（此前引用）；循环仍拒（2b.6/2c.6 域）。✓
- 批次 105 `hex`：`local3 = local2.length;`（`.length` 后缀）、`int local5 = local2[local4];`（字节元素读取无 cast）——**元素读取与长度落地**；BCI 32 噪声仍在（`%02x` 的 varargs 链，2b.7 域）→ 归 2b.7 验收复查。✓（部分：本批范围外项正确保持引用）
- 批次 97 `mixed()`：BCI 1 噪声从「not part of the provable subset」变为「array instruction … nothing reads」——引用语义升级（数组指令已建模、声明了未消费），2b.7 初始化链落地时消除。记录在案。✓
- `chained` 形状（fixture 内）：`// @bytecode 8 9` 替代未声明局部——2b.2 的「使用留在引用里」正确执行。

Behinder 第三次抽查（数组切片后）：`Utils` 整段拒绝 3→待测（时间关系本轮未跑全量，下轮补）。
