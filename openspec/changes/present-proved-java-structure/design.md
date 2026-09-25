## Context

恢复侧是一条单向链：正常流视图 → 事实 → 区域 → AST/发射。正常流视图只保留普通转移，异常边留在视图之外。这是本 change 要保住的边界。

今天的失败点是区域走法的返回约定，不是缺一张新图：

| 位置 | 事实 | 后果 |
| --- | --- | --- |
| `region.rs` `region_at_inner` 的 `ArmsDoNotMeet`、守卫拒绝、`leaving_edge` | 返回一个包含前缀的 `Fallback`，后继 `None` | 前缀不再是语句；主循环停止 |
| 同函数末尾的未覆盖扫描 | 只从异常边到达的活块另成一条 `jre_region_uncovered_blocks` | 整方法变成 `explanation_only` |
| `ast.rs` `StmtKind::If` 与 `emit.rs` | `else_body` 为空时不写 `else` | 单臂 `if` 的发射已经存在，走法拒绝了它 |
| `pass.rs` 的 `MONITOR` / TWR | 证明不了就引用，因为语句会丢掉出口或 close 顺序 | 不能用普通 `catch` 代替这次拒绝 |
| `classfile.rs` `AttributeFacts` | `exceptions`、`inner_classes`、`enclosing_method` 已解析 | `class_source` 与 `jarde-java` 的方法事实都还没接这些字段 |
| `emit.rs` `escape_string` | 非 ASCII 一律 `\uXXXX`；测试把 emoji 钉成代理项转义 | 中文字面量不可读，但这是有意的旧契约，本 change 显式替换它 |

`content` 规格已经允许「有语句 + 仍有未恢复区域」：`contains_statements` 且 `quality=Fallback`。本 change 让走法能够产生这种产物，不新增第四个 content 值。

## Goals / Non-Goals

**Goals:** 在不新增符号、不把失败的 TWR/monitor/`finally` 写成另一种语句的前提下，把已证明的 `if`、命名 `catch`、循环、`for`、accessor 字段访问、lambda/匿名类使用点，以及包名、`throws`、varargs、Unicode 标量写进类源码文本。

**Non-Goals:** 不把异常边并进 `NormalFlowView`。不实现 `finally`。不解析 `Signature` 与注解。不隐藏 bridge。不自动递归嵌套 jar。不改 bulk 汇总口径。不修数组槽宽与 T5。不用「与 jadx 文本相同」做断言。

## Decisions

### 1. 缺口是方法内的一段区域，不是整方法拒绝

`region_at` 在失败路径上可以返回一段区域序列再加一个后继。成功路径通常是单个区域；嵌套分支中的顺序语句与已证转移也可组成多个区域，由有序 `Sequence` 留在原分支内。不借用 `visited` 把前缀再送回去，否则下一次进入会命中 `jre_region_loop`。

| 情形 | 区域 | 后继 |
| --- | --- | --- |
| 前缀非空，当前形状证明不了 | `Straight(前缀)`，再加只含当前块及其未认领臂的 `Fallback` | 有证明过的汇合点就从那里继续；没有就 `None`，其余活块仍由未覆盖扫描引用 |
| 前缀为空 | 只发该 `Fallback` | 同上 |
| 守卫规则拒绝 | 拒绝范围只含该规则声明自己拥有的块 | 规则给出了 join 就从 join 继续，否则 `None` |

不变量：每个活块恰好属于一个区域，要么在结构化区域里，要么被一条 fallback 引用。图没有覆盖的指令仍走现有的 `jre_region_unaccounted_instruction`；该整方法引用时继续整方法引用。fallback 的 BCI 集合不得包含已经作为语句发出的指令。

### 2. 单臂 `if` 与「一臂退出」用已有的 `If` 节点

处理方式对照 jadx 1.5.x 源码里的 `IfRegionMaker.process`（`jadx-core/.../regions/maker/IfRegionMaker.java`），只取判断，不取它的后续改写，也不把 jadx 的文本当结果。

它的判断是：`else` 块为空，或者 `else` 块已经是这条 `if` 的出口（`stack.containsExit(elseBlock)`），就把 `elseRegion` 设成 `null`，然后从出口继续。对应到我们的图，就是两后继之一等于直接后支配点。空臂用已有的 `Region::Straight { blocks: Vec::new() }`，`switch` 对「臂就是 join」已经这么做。不新增区域种类。

同一文件和 `IfRegionVisitor` 里还有三件我们不采用：

- `mergeNestedIfNodes` 把连续的单臂 `if` 收成 `&&`。`nestedIf` 的源码是两层 `if`，javac 也编成两条跳到同一出口的分支。恢复成嵌套 `if` 就和源码一致，不收成 `&&`。
- 条件方向对调（`IfInfo.invert`，每个块只做一次）和 `removeRedundantElseBlock`（`then` 以 `throw`/`return` 结束时把 `else` 挪到 `if` 后面）。`armExits` 的源码是 `else` 里 `throw`、汇合后 `return`。jadx 把它收成 `then` 直接 `return`。那是改写，不是这条分支的结构。
- `TernaryMod` 在区域已经是 `if` 之后，把两臂都是纯值的 `if` 收成 `? :`。`ternary` 属于这一步，不属于单臂语句。本轮不收。

两后继之一就是该分支的直接后支配点时，另一臂是空区域。发射器已有的空 `else_body` 路径负责不写 `else`。不另造节点。

一臂离开方法，当且仅当该臂区域证明每条路径都是 `return` 或 `athrow`，且没有落到第三个块。这时沿用现有的 `If.join = None`：`if` 之后没有代码。证明不了「整臂退出」时，这条分支留在局部 `Fallback` 里，前缀仍然保留。不把「看起来像 throw」当成退出。

### 2a. `throw new` 是已有构造规则的消费者，不是新管线

`init.rs` 的 `new@1` 已经能证明 `new`、紧随的 `dup`、其间的 `ldc`，以及同类型的 `<init>`。字符串常量是 `Operation::Push`，落在 `StatementFree` 允许的范围内。`throw new IllegalArgumentException("bad")` 仍被整段引用，是因为 `renders_its_reads` 不包含 `Operation::Throw`：构造结果唯一的读取者是 `athrow`，规则就认为这个实例没有会被写出的消费者，于是分配、复制和构造一起引用。

处理仍在现有语句构建里。`athrow` 所读的值能渲染时，写成 `throw <expr>`，并让 `renders_its_reads` 承认 `Throw`。这样 `new@1` 不必知道 `throw`，它只看到一个会写出该值的消费者。不新增 `Region`。读不到值、或值来自本子集不写的指令时，保持今天的引用。

### 2b. 普通数组读写复用已有的下标表达式，不新开区域

`decode.rs` 只把 `iaload`（`0x2e`）收成 `Operation::ArrayLoad`，注释写明是给枚举 `switch` 的分派表。`aaload`、`arraylength`、`newarray`、`iastore` 仍是 `Operation::Other`。`build.rs` 对 `ArrayLoad` 的语句路径只在 `enumswitch@1` 认领时跳过，否则整段引用；取值路径也只渲染被认领的分派表，节点却已经是 `ExprKind::Index`。

所以 `arrayInit`、`forEach`、以及 monitor 体里的 `box[0] = box[0] + 1` 失败，不是缺一种区域。要补的是解码事实和现有下标节点的使用范围：

- 数组、下标、元素类型都来自该指令自己的操作数和描述符，不从「这是枚举表」推断。
- 枚举 `switch` 已认领的 `iaload` 保持今天的分派表拼写，普通读不得抢这条证明。
- 引用掉的 `istore` 不得让后面的使用写成没有声明的 `local5`。定义被引用吃掉时，使用处也必须留在同一条引用里，不能单独拼出一个名字。

`arraylength` 与 `new T[n]` / `{1, 2, 3}` 同属这一层。不为此增加 `Region` 变体。`multianewarray` 是同一层的另一条指令：`grid` 的 `new int[2][3]` 是 opcode `0xc5`，维度数在操作数里，元素类型在 `[[I`。它不是 `newarray`，也不要拆成两次一维创建。维度只取该操作数，类型只取池里的数组类。`anewarray` 仍不在这一刀。

发射器今天没有这三种拼写：`ExprKind::New` 印的是 `new T(args)`，`Assign` 只写一个名字，`Field` 会插入 `.`。所以这一层要加三个节点，不是复用错的节点：`NewArray { element, length }`、`IndexAssign { array, index, value }`、`ArrayLength { array }`。`array.length` 不得走字段节点，否则会和 `field@1` 的成员证明混在一起。

### 2c. 取负和 `instanceof` 也是操作数，不是区域

`arithmetic()` 只覆盖 `0x60..=0x73` 的二元运算。`ineg`（`0x74`）落到 `Other`。`ternary` 的 `-n` 因此在 `else` 臂里被引用，即使单臂判断修好也写不出取负。此项已拆成 `recover-unary-negation`：使用一个一元 `Operation::Negate` 与 `ExprKind::Neg`，保持 `ArithmeticOp` 和它到二元运算的全映射，不新增区域。

`instanceof`（`0xc1`）同样是 `Other`。`checkcast` 已解码，但只在 `bridge@1` 证明是擦除时才写。普通 `(String) value` 与 `instanceof String` 是同一类操作数：类型来自该指令的常量池类名。证明不了类型就保持引用。这也不需要新的区域种类。

`iand`/`ior`/`ixor`（`0x7e`/`0x80`/`0x82`）和 `ishl`/`ishr`（`0x78`/`0x7a`）也在 `0x60..=0x73` 之外。`Ops.bits` 与 `Ops.shifted`（`javac --release 8 -g:none`）因此整段是 `Other`。`BinaryOp` 已有 `+ - * / %` 和比较，没有位运算与移位。补的是这个枚举和 `arithmetic()` 的 opcode 表，发射器的 `spell()` 已是按枚举分支写的。不新增区域。

`i2b`（`0x91`）同样是 `Other`。`ExprKind::Cast` 和 `Type::Byte` 已经存在。`(byte) n` 是把这条转换接到已有转换节点，元素类型只由 opcode 决定，不从值的范围猜。`i2c`/`i2s` 同理。字节码里真有 `i2l` 等加宽指令时同样写成转换，目标类型只由 opcode 决定。没有这条指令的加宽（`int` 赋给 `long` 局部、javac 不发射转换）仍不补。`ineg`/`lneg`/`fneg`/`dneg` 是同一个一元 `Neg`，不是四个节点。`return n < 0 ? -n : n` 在取负修好之后仍缺汇合值，那是延后的三元，不拿它验收取负。

`if (value == null)` 已经写成 `if (arg0 == null)`。`return !flag` 写成了 `if (!arg0) {}`，汇合处的 `ireturn` 丢了两臂的常量。`!` 本身在，缺的是两臂都是纯值、汇合处只读这个值时收成一个表达式。这仍是延后的三元，不是再修一次单臂 `if`。

### 2f. `i++` 的返回值不能再用被改过的槽位名

`return i++` 在 `javac --release 8 -g:none` 下是 `iload`、紧接着的 `iinc 1`、然后 `ireturn`。`ireturn` 读的是 `iload` 压下的旧值，槽位已经在中间被改过。`build.rs` 因此拒绝用这个槽位的名字（「the slot does not hold it」），方法停在 `local1 = local1 + 1` 后面的引用。`return ++i` 是先 `iinc` 再 `iload`，名字仍然指同一个值，已经写成 `local1 = local1 + 1; return local1;`。

这不是新的区域。`iinc` 仍是一条语句。返回值按它的生产者写：下一条指令就是对同一槽位的 `iinc ±1`，并且这个使用读的就是那次 `iload` 的值时，写成 `local++` 或 `local--`。其它「槽位在使用前被改写」的情形不套这个糖，写成在改写之前绑定的那个值的名字，改写语句留在原位。不得把 `return i++` 收成 `return i` 而丢掉 `iinc`，也不得在 `iinc` 之后返回槽位的新值。

### 2g. `switch` 穿透是少写一个 `break`，不是两臂共用一块

`fall` 的 `case 1` 落到 `case 2` 的入口。`region.rs` 把从 28 走出的区域和从 30 走出的区域都算成占有块 30，于是 `SwitchArmsOverlap`，整段引用。发射器对非 `return` 的臂一律补 `break`，所以即便区域切开了，不改发射器仍会变成两个互不穿透的臂。

不新增 `Region`。先按现在的汇合边界走一遍。臂里若出现另一个 case 的入口，停在这些入口里字节码偏移最小的那一个，用它做边界再走一遍，那一块及它后面的代码只属于后一个臂。`SwitchArm` 记下这一臂是穿透，发射器就不补 `break`。跳过中间 case、直接进入更后一个 case 时，停在实际进入的那个入口。空臂和以 `return` 结尾的臂保持今天的 `break` 写法。两臂共用的块不是任何一个 case 入口时，仍是 `SwitchArmsOverlap`，不得把同一块写进两个臂。

新增的非数值顺序真实对照见 `../../evidence/java-syntax-2026-09-22/switch-fallthrough-order/analysis.md`：`lookupswitch` key 表按 1、4、9 排列，但代码 BCI40 的 `case 9` 先于 BCI43 的 `case 1` 并自然穿透。现有 group 依 key 顺序输出，整类虽编译通过，却把 91 写成 90。同目录 `tableswitch/` 独立证实密集键亦如此：表内 1/2→BCI39、3→45、4→36，实际先执行 case 4 的 BCI36，再自然落入共享标签 case 1/2 的 BCI39；当前整类可编译但两项 4→41 错成 40，并给 BCI39 留下一处引用。穿透被证明后必须按目标代码入口 BCI 输出这些臂及其标签，而不是沿 key 表顺序；default 的真实入口同样按代码位置排列，共享 target 的标签仍合并，join-only 空臂保持独立 break。只有按代码顺序发射，去掉前臂 break 才仍是原控制流。

实现时还要注意 `Walker::visited` 是全局认领集合，`region_at` 一旦走过后一个 case 的入口就已改变它。上文的“先走一遍再切开”是证明顺序，不允许直接在同一个已污染 walker 上重走并信任其重入结果；可先从 CFG/目标入口确定受限臂边界再认领，或在有界试探后准确恢复认领状态并为试探计费。某臂若可能进入多个非相邻 case 入口，单凭最小 BCI 也不足以证明 Java 顺序穿透，应继续引用而非排列出看似可编译的错误语义。

本地 JADX `2fb1b1638694` 的 `SwitchRegionMaker.addCases` 可直接借鉴两个算法步骤：先按目标块合并多个 key/default，再用 CFG 支配前沿与 case 入口集合找穿透目标；构建前把该目标设为臂的出口，后继臂独占其代码。它在 `insertBreaksForCase` 与 `SwitchBreakVisitor` 中依据 `FALL_THROUGH` 和真实出口分别处理 `break`。但 `reOrderSwitchCases` 用相邻关系 comparator 排序，遇到不能修好的顺序会清掉穿透关系并告警；这不是 Jarde 可以沿用的语义证明。这里改为从已证唯一入口边建立有向 case 链，要求无分叉、无环且链边均连接输出时相邻的 case；与其它独立臂按真实代码 BCI 排列。若链无法满足 Java 的相邻穿透语义，保留引用，不用复制代码或猜测 `break`。只对通往 switch 自身 join 的路径补 `break`，不可把通往循环出口、`return` 或另一个 case 的边一律当作 `break`。

`More30.fall` 现在整段是 `explanation_only`。`r = 0` 和 `lookupswitch` 在同一块，所以这不是 1.2 的前缀块；穿透切开之后，块里 `switch` 之前的存储仍由已有的块内语句写出。数组 `for-each` 已经是下标 `while`，迭代器 `for-each` 停在 2c.6 的 `hasNext`。两者都不新写成 `for-each`。字符串 `switch` 已经是 `hashCode` 加 `equals`，不得收成 `case "ab"`。

字段写入读到的值是一条已证明构造链的 `dup` 遗留值时，那条链就是这个字段的初始化表达式。`this.f = new T(args)` 和静态 `F.f = new T(args)` 的字节码都是 `new; dup; invokespecial <init>; putfield/putstatic`：`new` 的一份复制做构造器接收者，另一份就是要存的值。局部存储的同形状已经由 new@1 呈现（`astore` 读到 `new` 就写 `local = new T(...)`），被拒的只有字段写入这一路，原因是 field@1 不认 `Duplicate` 生产者。修法在现有两个规则之间：字段写入的值经 `dup` 追溯到已验证的构造点时，用 new@1 已有的 `New` 表达式做右值，接收者写法沿用 field@1（实例带接收者、静态带属主类型）。不新增表达式种类。枚举 `<clinit>` 的常量赋值（`Color.RED = new Color("RED", 0)`）和任何 `INSTANCE = new X()` 都是这条。写成语句后也不把常量合并回 `RED, GREEN, BLUE;` 声明——那是把 `<clinit>` 的效果挪进字段声明的另一种改写，只有 ConstantValue 属性那种工件自述的初始化才上提。

同一个遗留值还有两个单消费者位置：`areturn` 直接读它（`return new T(args)`），和一条调用的参数表读它（`take(new Box())`）。它们和局部 `Store`、字段写入一样，都是「一份复制给构造器接收者、另一份给唯一消费者」的形状，右值同用 new@1 已有的 `New`，不发明局部，也不把构造拆成独立语句。多个消费者同时读这个遗留值时仍引用——两份消费没有单一的 Java 拼写。这不是新的表达式种类，是 new@1 已验证构造点在值位置的复用。

`ACC_VARARGS`（0x0080）是方法自己的声明事实：最后一个参数写成 `int... arg1`。它只改签名的拼写面，数组语义不动，体内该槽仍是数组，调用点的可变参数装箱仍由 2b.7 的初始化链处理。主工作区的接口签名缺 `default` 关键字是 6.5 在声明 worktree 未并入的状态，不是新任务；并入时与 6.7 同在签名的装配处，合并顺序上先 6.5 后 6.7 可以共用同一批校验。

构造器中 `invokespecial` 前的合成捕获字段写入必须保留原时序，不能为了让独立的二进制名类源码通过 Java 8 编译，就改写成 `super(); this.val$captured = arg1;`。这不只是拼写问题：`anonymous-super-dispatch` 的 `Base()` 在基类构造尚未返回时虚调用匿名覆写；原 class 的 `val$captured` 先于 `Base.<init>` 写入，因此覆写读到 `captured-value`。换序后它会读到默认 `null`。单独呈现匿名/内部类时，可保留前置写入及其真实 BCI，并明确整段不声称可编译；只有类级投影已证明 `new Base(...) { ... }` 的捕获语义与原 class 一致时，才交给 Java 编译器重建这一合成初始化顺序。其它自定义构造字节码也不能凭字段名推断可重排。证据见 `../../evidence/java-syntax-2026-09-25/anonymous-super-dispatch/evidence.md`。

赋值、字段写入与返回的隐式加宽由既有 `meeting_position` 保持，不额外包 cast；concat 则依赖操作数类型选择转换，继续显式陈述。调用参数是不同的消费位置：重编译会重新进行重载选择，缺少 cast opcode 不等于源码不需要静态类型约束。Object、char→int、byte/short 常量及泛型返回的可执行反例已推翻此前将调用与赋值一并处理的范围；具体安全构造和未知引用关系拒绝见 [preserve-invocation-argument-types](../preserve-invocation-argument-types/design.md)。不为匹配 descriptor 合成任意可失败 checkcast，也不把这项修改传播到返回和赋值。

`throw` 语句（2c.20）的落点在 `build.rs` `instruction()` 的 `Operation::Throw` 臂：今天它无条件 fallback（「a monitor or a bare `throw` is written only where a rule claimed the statement around it」）。改法：守卫规则没认领的 `athrow` 写成 `StmtKind::Throw`，操作数走与返回位相同的值渲染。`throw new T(...)` 的操作数恰是 2c.27 正在落地的「构造链 `dup` 遗留值给单一消费者」——所以 2c.20 必须排在 2c.27 之后派发，届时 `throw new java.lang.IllegalArgumentException()` 复用同一个消费者机制，不再另写识别。TWR 处理器的重抛、`finally` 副本、monitor 的异常路径已被守卫规则认领，不经过这个臂，天然不重复写。

数组切片（2b）不文本合并，按规则清单在当前主工作区重实现。数组 worktree 基于旧 commit（`a83b557`），改了 10 个源文件，与主工作区后来的 forward_join、嵌套 try、2.6/2.7、2c.25 以及在跑的各任务全部冲突；它的价值是已验证的规则集，不是可直接套用的 diff。重实现时的清单：解码侧 `0x2e–0x35`（读）按元素类型收成 `Operation::ArrayLoad`、`0x4f–0x56`（写）同理、`0xbe` 是 `ArrayLength`、`0xbc/0xbd` 是 `NewArray`；AST 侧 `ExprKind::NewArray { element, length }`、`ExprKind::ArrayLength { array }`、`StmtKind::IndexAssign`；构造侧 `ArrayBindings`（局部持有哪次创建、哪些值是它的副本、`subsumed` 判定）、`boolean_array_read`（`[Z` 证据）、嵌套 `g[i][j]` 的下标复用；发射侧 `IndexAssign` 的 `[` 作后缀、`new T[n]` 作 Primary、`.length`。重实现顺序排在 2c.27 之后：`2c.23`（`a[i] = a[i] + 2`）和 `2b.7`（初始化链）都直接依赖这些下标表达式，一起派可以共用 fixture。census 与 fingerprint 到时按新 fixture 重测。

声明切片（6.1/6.2/6.5/6.6）同样不文本合并，重实现于当前主工作区。声明 worktree 也基于 `a83b557`，改动落在 `src/class_source.rs`、`src/facade.rs`、reader 的 `classfile.rs`，外加 `emit.rs`/`region.rs` 少量；主工作区这些文件已各自前进（6.3 标量转义、2.7 的 region、6.8 正在改装配层）。规则清单：6.1 是 `this_class` 的 `/` 定包（无 `/` 是默认包、不写 `package` 行），成员表用简单名、`$` 保留；6.2 是 `throws` 只来自 Exceptions 属性、同名成员拼描述符；6.5 是 `default` 只在类自身 `ACC_INTERFACE` 且方法非 static、非 abstract、有 `Code` 时出现；6.6 是字段 `ConstantValue` 作声明初值。6.7 的 varargs 机制该 worktree 已解决过一轮：`ACC_VARARGS` 只及最后一个参数、且仅当该参数是数组——`T[]` 写 `T...`，`[[B` 写 `byte[]...`（元素类型加 `[]` 再加 `...`），重实现沿用这个边界。并入顺序：6.8 停手后先重实现 6.1/6.2/6.5/6.6（同一装配层，一批 fixture），6.7 随后同层补上。

`return switch (n) { case 1 -> 2; default -> 0; }` 不是新的区域。javac 把它收成普通 `lookupswitch`：每臂留下一个值再 `goto` 同一个 `ireturn`，默认臂直接落到这个 `ireturn`。汇合处读到的是栈上的 phi。现有渲染把栈 phi 说成「没有生产者」，所以 `ireturn` 被引用，每臂只剩空的 `break`。不读这个 phi，也不新增 switch 表达式。汇合块只有这一条 `return` 时，看每臂转到汇合前留下、且臂内没有人再读的那一个栈值，把 `return` 这个值写进该臂，汇合处不再写。臂里已有的存储留着：`int local1 = arg0 + 1; return local1`，不得收成 `return arg0 + 1`。某一臂留下的不是恰好一个栈值，或汇合处除了 `return` 还有别的指令，保持今天的引用，不得发明局部变量来接这个结果。这不是穿透，汇合块也不是下一个 case 的入口。

`record` 不另写成 `record` 头。类文件里它是 `final class` 继承 `java.lang.Record`，字段、构造器和访问器都是普通成员，已经按这些成员写出。`equals`、`hashCode`、`toString` 是 `java.lang.runtime.ObjectMethods.bootstrap`，不是 `LambdaMetafactory`，保持拒绝。不得删掉这些成员来换成一个 `record` 声明。

### 2h. 浮点字面量是常量解码，循环条件里的调用不是新区域

`half` 的 `ddiv` 已在 `0x60..=0x73` 里，会被收成 `Divide`。整方法失败是因为 `ldc2_w` 的 `double 2.0` 在 `constant()` 里被丢成 `Other`。`tiny` 的 `ldc float 1.5f` 同样。`ConstantValue` 只有 `Int`/`Long`/`String`/`Null`。补 `Float` 与 `Double`，字面量必须能还原同一组位，不另造区域。`fconst_0`/`fconst_1`/`fconst_2` 和 `dconst_0`/`dconst_1` 不是池项，值就是 0、1、2，写成 `0.0f`、`1.0f`、`2.0f`、`0.0d`、`1.0d`。`fdiv` 已经在同一段算术里。`Class` 常量（`More3.class`）同属这一处：池项已经是类名，写成 `Type.class`。没有池项时保持引用。

`sum` 的头部是 `iterator.hasNext()`。`test_expr` 已经能把 `Invoke` 放进条件，但 `test_is_pure` 只允许 `Push`/`Load`/`Arithmetic`，于是 `loop@1` 以 `StatementFree` 拒绝整段。调用写在条件里时，每次测试执行一次，并没有被挪到循环外。允许的是：测试块里除分支外的每条指令都是这个条件读到的值，包括调用。存储和 `iinc` 仍拒绝。不把 `while (it.hasNext())` 猜成 `for (Integer value : list)`：手写的迭代器循环字节码相同，`for-each` 只有在 `iterator`/`hasNext`/`next` 三条都按这个形状对上时才有资格，本轮不作为完成条件。

`assert n >= 0` 在 class 里是 `if (!$assertionsDisabled && n < 0) throw new AssertionError()`。release 二进制把它认成可重入，是因为两条分支都落到同一个 `ireturn`，这正是已经落地的单臂 `if`。剩下的 `throw new` 是 2a。不另写 `assert` 糖：字节码没有 assert 指令，`if` 写对了就是这个程序。

`interface` 的默认方法体已经是 `return arg1 + 1`，但声明没写 `default`。方法标志 `0x0001`、所在类型是接口、不是 `static` 也不是 `abstract` 时，声明前加 `default`。静态接口方法已经写成 `static`，不加 `default`。

`enum Color` 的关键字、`super(name, ordinal)` 和 `this.ordinal()` 已经对齐。常量仍是字段，`$values` 用了未建模的 `anewarray`，`<clinit>` 里的 `new Color` 仍是 2a 的「消费者被引用」。不新增枚举区域。把常量挪进枚举头要等构造能写出来，再证明每个 `ACC_ENUM` 字段就是一次 `new 本类(名字, 序号)`。

### 2i. `super`、比较指令和常量字段都是已有拼写

`More5.value` 的字节码是 `invokespecial Base5.value`。现在写成 `this.value()`，这是另一个程序：它调用自己，而不是父类。`call_expr` 对非 `static` 调用一律把第一个操作数渲染成接收者。2026-09-22 的 SpecialProbe 已以重编译执行确认 StackOverflowError，并发现接口 `I.super.value` 同样误写；jadx 在接口例也返回错误的 7（原值 11）。此项独立拆成 `preserve-special-call-dispatch`，以同源直接父类/接口声明、池项类别和 SSA 入口 this 证明来选择 `super`/`I.super`，不能只比较 owner。当前类 private 须由成员 flags 证明并保留实际 receiver；同类其它对象的 private 调用仍可写，不能一律拒绝。`<init>` 保持 `init@1` 规则。其它不能证明的特殊调用引用，不偷偷改成虚分派。

比较条件由独立 [recover-numeric-comparison-conditions](../recover-numeric-comparison-conditions/design.md) 承接。五种比较事实保留数值种类与 NaN 偏置，只组合唯一、同块、紧邻零分支的结果；先定 taken/fall-through 极性，再映射既有 Binary/Not。NaN 为真的关系可以写成 `!(a < b)`，不再要求一概留下比较结果；也不能直接写成含义不同的 `a >= b`。不新增区域，不恢复一般比较结果值、三元或布尔汇合。1,309 组原程序/jadx 对照已证明 34 个 NaN 错值，实施以原 class 为基线。

`static final int N` 带 `ConstantValue: int 3`。字段声明应写成 `= 3`，值只来自这个属性，不从 `<clinit>` 推断。`named()` 的字节码是 `iconst_3`，不是 `getstatic`，所以保持 `return 3`。jadx 写成 `return N` 是把内联还原了，不跟随。`volatile`、`static synchronized`（锁在方法标志 `0x0020` 上，方法体里没有 `monitorenter`）、`return null`、`%` 和空方法的 `return` 已经对齐。

协变桥 `Object get()` 的体是 `invokevirtual get()Ljava/lang/String;`。现在写成 `return this.get()`，读起来是自调用。桥方法保留，不加 `@Override`，也不像 jadx 那样删掉。同名而描述符不同的调用，文本里必须出现被调用的描述符，不能只写与所在方法相同的简单名。

`continue outer` 和 `break outer` 是同一条规则：边的目标是外层头部就带标记 `continue`，是外层出口就带标记 `break`。`outerContinue` 里 `goto 2` 指向外层头，现在把外层整段引用。无标记的 `continue` 会回到内层，那是错的。

外层 `while` 的体只有 `break` 时，字节码没有回到外层头的边，外层就不是循环，而是 `if`。不得把这个 `if` 还原成 `while`。内层的 `break` 和紧跟着的外层 `break` 会被 javac 收成同一条到出口的边，文本只写这一条边，不拆成两层 `break`。`m` 只被赋值一次时不得删掉，jadx 把它折进被修改的外层局部，不跟随。

### 2j. 其它元素宽度仍是下标表达式

`More6` 的 `baload`/`bastore`、`caload`、`saload`、`laload`、`faload`、`daload` 现在都是 `Other`。栈效果按 opcode 算，不按 `Operation` 算，所以这些指令可以收进已有的 `ArrayLoad`/`ArrayStore`，文本仍是 `array[index]` 和 `array[index] = value`。不新增节点，也不新增区域。`aaload`/`aastore` 同样是这个表达式；`anewarray` 仍不在这里，引用数组的创建另算。

`return new int[]{1, 2, 3}` 没有局部变量。字节码是一次 `newarray`，然后每个元素 `dup`、下标、值、`iastore`，最后把留下的数组返回。下标正好是 `0..n-1`、而且这些 `dup` 没有别的使用者时，这是一个表达式 `new int[]{1, 2, 3}`。同一个表达式可以返回、存进局部，或传给调用。`args(1, 2, 3)` 就是先造这个数组再 `invokestatic`，写成 `args(new int[]{1, 2, 3})`，不改成可变参数调用。一个元素的 `{value}` 也是这个序列，不再拆成先创建再 `local[0] =`。缺下标、下标不是这个序列、或者 `dup` 还有别的使用者时，保持引用。不发明局部。

`baload` 不区分 `byte` 和 `boolean`。只有数组引用的类型已经证明是 `[Z`（参数、字段或 `newarray` 的 `atype` 4）时，这个值才是 `boolean` 证据，`flag` 才能写成 `return arg0[arg1]`。证明不了就保持今天的拒绝，不能看见 `baload` 就写成 `boolean`。`[B`、`[C`、`[S` 的读取文本不加 `(byte)`、`(char)`、`(short)`，字节码里没有这些转换指令。

`sumChars` 的测试在闩锁上，回边指向头部。这是已有的 `DoWhile`。现在失败是因为测试里的 `arraylength` 和 `caload` 不是值，`test_is_pure` 只认 `Push`、`Load` 和 `Arithmetic`。这两条和 2c.6 的调用一样，是条件读到的值，可以留在测试里。存储和 `iinc` 仍然拒绝。不要因为 jadx 把 `sumBytes` 写成 `for (byte b : bArr)` 就猜 `for-each`。

`readOuter` 已经是 `return arg1.outer.value`。字段链对齐，不加任务。

### 2k. 两条前向边汇合不是循环

`both` 的字节码是 `if (arg0 <= 0) goto 10; if (arg1 <= 0) goto 10; return 1; return 0`。BCI 10 被外层条件和内层条件各走一次，两次都是向前的。现在第二次进入被写成 `FallbackReason::Loop`（「can be re-entered」），`return 0` 丢了。这不是循环：该块没有回边，也不是循环头。

正确写法是把这个块当成汇合，写在两层 `if` 后面，而且只写一次：

```java
if (arg0 > 0) {
    if (arg1 > 0) {
        return 1;
    }
}
return 0;
```

`||` 同样。不要收成 `&&` 或 `||`，也不要收成三元。汇合仍不是直接后支配点，因为为真的那条路提前 `return` 了。单臂 `if` 只处理「一个后继就是后支配点」。这里要补的是：一个后继不是循环头、它的每条入边都从前向块来、而且这些前向块都由这条分支支配时，它就是这条 `if` 的汇合。另一臂走到它就停。已经是循环头的块仍走循环。真正的回边仍是 `Loop`，不得因为这条规则被当成汇合。

`pick` 仍是延后的三元：两臂都是纯值，汇合处只读这个值。空的 `if` 不是这条汇合规则能补上的。

`this(1)`、`super()`、`synchronized (arg0)`、`arg0.run(arg1 + 1)`、当前类上的 `invokespecial hidden`（`this.hidden`）已经对齐。无 `default` 的 `switch` 由 javac 补了 `default`，写出 `default` 与字节码一致。

### 2l. 字段的旧值自增不是新局部

`next` 是 `aload_0; dup; getfield count; dup_x1; iconst_1; iadd; putfield count; ireturn`。返回的是加之前的字段值。`dup_x1`（`0x5a`）现在是 `Other`，整段引用。局部变量的 `i++` 是 `iload; iinc`，这条不是。

只有这一条栈形能写成 `return this.count++`：复制的是同一个对象，读和写是同一个字段，加上的是 `1`，留下的栈值是读到的旧值。其它 `dup` / `dup_x1` 仍引用。不发明局部变量。jadx 写成 `int i = this.count; this.count = i + 1; return i`，值对，但是多出来的变量。不跟随。

`anewarray` 是 `new T[n]`，元素类型是池里的类，不是 `atype`。复用已有的 `NewArray`，不新增节点。`aaload` / `aastore` 复用下标表达式。`aaload` 不得呈现成 `int`。数组类型已证明是 `T[]` 时，元素类型是 `T`；证明不了就不声称类型。不要写成 `new String[]{s}`：那是初始化糖，字节码是一次创建加一次存储。

### 2m. 测试里的一次赋值，以及 `long` 上的同一批运算符

`drain` 的测试是 `iload; invokestatic; dup; istore; ifle`。比较读的就是刚存进参数的那个值。`dup` 已经是 `Duplicate`。这不是新循环：条件写成 `(arg0 = step(arg0)) > 0`，存储仍在每次判断之前发生。只允许这一条：测试里恰好一次 `Store`，存的值和分支读的值是同一次 `dup` 的两边。存的是别的值、有两次存储，或存储后面还有别的效果，仍按 `StatementFree` 拒绝。不要照 jadx 改成 `while (true)` 再把赋值搬进循环体。

`lshl`/`lshr`/`lushr` 和 `land`/`lor`/`lxor` 是 `ishl` 那一组的 `long` 形式，同一个 `BinaryOp`。`n << 1` 写成 `arg0 << 1`，`n & 3L` 写成 `arg0 & 3`。不新增节点。`iinc` 的 `n += 3` 已经是 `arg0 = arg0 + 3`，不折成没有存储的 `return`。

`switch (char)` 的键在字节码里是整数。选择表达式的类型已经是 `char` 时，键写成字符字面量，转义规则与字符串相同。选择表达式是 `int` 时保持数字，`case 97` 不得改成 `case 'a'`。

`blockBreak` 被 javac 收成 `if (n < 0) return -1; else return n`。两边都是 `return`，与字节码一致。不把已经消失的标记补回去。

### 2n. 别的类上的静态调用必须带类型

`invokestatic` 的接收者现在是 `None`，发射器就不写点号前面的类型。`Integer.valueOf` 因此变成 `valueOf(arg0)`。当前类里没有这个方法，文本是另一个程序。

池里的属主不是当前类时，调用写成 `Owner.name(args)`，类型用点号，`$` 保留。属主就是当前类时，保持现在的无限定名：`step(arg0)` 已经指向本类。不要把装箱收成 `return n`，也不要把 `intValue()` 收成拆箱。那两条是真实调用。

`counted` 已经是 `while (local2 < arg0)`，更新是 `local2 = local2 + 1`。这就是计数 `for` 要认的那三条，不需要新的循环区域。`return ++n` 已经是先加再返回。`return n++` 把加一写成了新值，旧值的返回被拒绝。`switch` 穿透仍是 `SwitchArmsOverlap`。这三处都是已有任务，不再加。

### 2o. 接收者的 `dup`、数组元素的后增，以及比较的 0/1

`this.n += k` 是 `aload; dup; getfield; iadd; putfield`，两次使用的是同一个接收者。`dup` 不是表达式。字段读写已经能写 `this.n`。写成 `this.n = this.n + arg1`，不写成 `+=`，也不当成字段的 `++`：留下的值不是旧值，加上的也不是常量 1。其它 `dup` 仍引用。

`a[i]++` 是 `dup2; iaload; dup_x2; iconst_1; iadd; iastore`，留下的值是旧元素。下标表达式已经有了之后，这个值写成 `arg0[arg1]++`。不发明局部变量。jadx 写成 `int t = a[i]; a[i] = t + 1; return t`，不跟随。其它 `dup2` 仍引用。数组初始化的 `dup` 不是这个形状。

`if ((n = n - 1) > 0)` 与循环里的 `(n = step(n)) > 0` 是同一次 `dup` 的两边。条件规则不限于循环。

`return a == b` 被 javac 收成比较、`iconst_1`、`goto`、`iconst_0`。两臂只有这两个常量、汇合只返回这个值时，文本就是 `return` 那个比较，极性跟分支一致。汇合把这个值存进一个局部或一个字段时，同样是这个比较，不是空的 `if`。`$assertionsDisabled = !desiredAssertionStatus()` 就是后一种，字段仍是字段，不写成 `assert`。两臂是别的值时仍是推迟的三元，不写 `? :`。

已经证明的 `StringBuilder` 链里，`append(int)` 会收成 `+`，`append(char)` 不会。`append(C)` 写的是这一个字符，左操作数已经是字符串时就是 `"x" + c`。`append([C)` 和 `append(CharSequence)` 仍不收：它们和 `+` 不是同一个程序。不新增区域。

Java 9 以后的 `a + "/" + n` 不再是这条链。它是一条 `invokedynamic`，引导方法是 `java/lang/invoke/StringConcatFactory.makeConcatWithConstants`，配方 `\u0001/\u0001` 里的 U+0001 是按顺序填入的操作数，其余字符是字面量。这和 `lambda@1` 读的是同一张引导表，但引导方法不是 `LambdaMetafactory`，所以现在整段被 A04 拒绝。文本仍用已有的 `+`，不新增表达式，也不把这个站点写成 lambda。U+0002 或配方之外的引导参数表示还没证明的常量，整条拒绝。`"" + n` 仍写成 `"" + arg0`，不得像 jadx 那样丢掉转换、写成 `return arg0`。`--release 8` 的 `StringBuilder` 链保持今天的写法。

`n += 1L` 已经是 `arg0 = arg0 + 1L`。`tableswitch` 与 `lookupswitch` 都已经是 `switch`。两边都是 `return` 的 `if` 保持两个 `return`。字符串 `switch` 的 `hashCode` 分派不还原成 `switch (字符串)`；`equals` 为真才赋值、两边都到下一个 `switch`，是已有的单臂 `if`。用来对照的 release 二进制早于那次修复，不能把旧输出当成现在的失败。

`return ++this.n` 与 `return this.n++` 都先 `dup` 接收者。后增是 `getfield; dup_x1; iconst_1; iadd; putfield`，留下旧值。前增是 `getfield; iconst_1; iadd; dup_x1; putfield`，留下新值。前者是已有的 `field++`。后者写成 `++field`，因为返回的是这次加出来的值，不是再读一次字段。不发明局部变量。这个形状只在最后一条是 `ireturn` 时成立。`int x = this.n++` 的第八条是 `istore`，仍会引用，并且后面的 `iload` 可能写出没有声明的 `local1`。那一次存储就是声明：`int local1 = this.n++; return local1`。不得再写 `this.n =`。

静态字段没有接收者。`return N++` 是 `getstatic; dup; iconst_1; iadd; putstatic; ireturn`，`dup` 留下旧值。`return ++N` 是 `getstatic; iconst_1; iadd; dup; putstatic; ireturn`，`dup` 留下新值。接收者沿用静态字段已经写出的属主类型，所以是 `More23.N++`，不是没有类型的 `N++`，也不是 `+=`。其它 `dup` 仍引用。

`synchronized (this) { return this.n; }` 的正常路径在 `monitorexit` 之后直接 `ireturn`，没有 `goto`。现有规则要求退出后必须是 `goto`，所以整段被拒绝。返回的值是退出前 `getfield` 留在栈上的那个值，不是退出后再读一次字段。写成 `synchronized (this) { return this.n; }`。不发明局部，也不照 jadx 把返回挪到块外。退出后是 `goto` 的形状保持今天的写法。退出后是别的指令时仍拒绝。处理器的 `catch_type == 0` 仍不是 `finally`。

静态字段 `N += k` 没有接收者 `dup`，已经是 `More15.N = More15.N + arg0`。`else if` 已经是嵌套 `if`。稀疏 `lookupswitch` 已经是 `switch`。`n <<= 1` 等移位恢复后仍是 `arg0 = arg0 << 1`，不写 `<<=`。两个不同槽位的更新保持 `while`，不写成 `for`。

### 2p. 循环体在 `switch` 之后还要往下走

方法级的恢复会顺着区域的后继继续走。`header_tested_loop` 把体的后继丢了。循环体开头是 `switch` 时，`switch` 的汇合还在循环里面，包括后面的更新和回边。这些块没被走到，`covers` 失败，整个循环变成 `LoopShape`。

循环体用已有区域的有序列表按方法同一条规则接住后继：后继在循环范围内就接着写；后继是循环头就结束循环体；后继是循环出口才是 `break`。分支臂原先只能容纳单一区域，为让条件块之后的跳转仍留在该臂内，局部使用有序 `Sequence` 包住多个已证区域；它不认领额外 CFG 块，也不替代 fallback 的证据。`switch` 自己的汇合仍是 `switch` 的 `break`，不是循环的 `break`。

`break` 写在 `switch` 里面时，无标记形式断开的是 `switch`。字节码跳到循环出口时必须带循环的标记，即使只有一层循环。跳到循环头的 `continue` 不被 `switch` 截住，保持无标记。jadx 把 `out` 的回边写成了循环里的 `return`，不跟随。

[可执行的 switch/loop 多出口反例](../../evidence/java-syntax-2026-09-24/switch-loop-exits/analysis.md)把问题进一步分开：BCI 64 是 switch 的全方法后支配点，也是外层循环出口；BCI 58 才是仍留在循环内的两个 case 的正常完成汇合。现有 `switch_region` 直接用 immediate post-dominator 当 join，会让不同臂共同认领循环尾并报 `SwitchArmsOverlap`。在现有 `Frame.loop_targets` 和 arm 走访中，先将精确指向外层循环出口的边当作带目标身份的终止边；仅对其余能正常完成的臂寻找位于循环范围内的共同汇合，核验每条入边与后继，并在该点停止各臂。条件臂也要允许一条路径终止于已证明的 loop break、另一条走到 switch 局部汇合；证明不了时保留引用。不能把全方法后支配点或 BCI 顺序代替这项证明。

Emitter 对每个 case 追加的 switch `break` 只适用于仍能正常完成的臂。臂若已经以 `break`、`continue`、`return`、`throw` 终止，不再追加；最后是 `if` 时按两条路径判断 Java 可达性，避免写出不可达语句。本地 JADX 的 `SwitchRegionMaker.insertBreaksForCase` 会在嵌套 region 与整个 case 上追加合成 `break`，再靠 `SwitchBreakVisitor` 清理；同一合法 class 在 JADX 1.5.6 中留下连续两条 `break;`，`javac --release 8` 拒绝。Jarde 保留自己的目标身份与完成性证明，不借这种后清理补救。

[循环内真实可抛 try/catch](../../evidence/java-syntax-2026-09-24/loop-try-handler-entry/analysis.md)另揭示一个独立的循环入口判定：异常边已从 `NormalFlowView` 排除，但处理器 BCI 17 的普通后继 BCI 20 汇回循环，方法入口沿普通边又无法到达 17。全图 SCC 把 17→20 误当成循环的第二个普通入口，于是先于 `try_region` 报 `Irreducible [2,6,20]`。修复应继续用现有 handler row 的受保护块、处理器身份与 loop scope，证明这条后继确是**循环内部**的 catch 汇合；只在该证明下把处理器的异常根入口从普通循环入口判定中排除。处理器体及 BCI 20 仍须由 try/catch 区域覆盖，正常和抛出路径都执行 BCI 20 的递减。来自循环外或跨不相容保护范围的处理器不能获得豁免，仍拒绝。JADX 在主区域构建后再处理异常处理器和 try/catch 的顺序可参考，但不复制其区域重排作证明。

实际 class 还须区分“`continue` 目标是头部”和“目标是体尾更新闩锁”。前者可由现有 `while` 直接表达；后者若把更新仍写在 `while` 体尾，Java 的 `continue` 会跳过更新，形成死循环或少一次副作用。`Grid.labeledContinue` 的目标恰是内层自然出口兼外层更新入口，单凭字节码无法区分源码的外层 `continue` 与内层 `break`；[BCI 对照](../../evidence/java-syntax-2026-09-24/loop-transfers/analysis.md)不以它验证外层标签。带有内层循环后可观察语句的独立样本把外层更新入口与内层出口分开；只有当 3.2 把已证明的更新放进 `for` 头，或另有同样可证明、会在 `continue` 时执行该更新的表示，才可将这条边写成 `continue outer`。否则保留缺口，不输出虽能编译却少更新的 `while`。这使 3.1 的闩锁 `continue` 验收依赖 3.2 的对应投影；循环体续接、精确出口 `break` 与头部 `continue` 可先独立闭合。

本地 JADX `2fb1b1638694` 的 `LoopRegionMaker` 将 loop exit edges 与条件块的出口分开处理：对循环体的其他出口沿 CFG 寻找可插入 `break` 的边；对指向 loop end 的前驱再判断是否是 `continue`，并防止同一位置已有 `break`。这个边分类次序可复用。其 `canInsertBreak` 遇到入口到目标路径中存在 `switch` 时直接拒绝，`addBreakLabel` 也只处理一部分多层循环；Jarde 不应靠这种全路径猜测。以当前 Region 的嵌套栈和实际目标身份决定 `break`/`continue` 属于哪层结构：循环出口是目标、switch 汇合是另一目标，跨过 switch 的循环出口必须带循环标记；不能把一个 `goto` 因离开当前块就变成无标记 `break`。

### 2d. 计数 `for` 是已有 `while` 的拼写，不是新的循环区域

`header_tested_loop` 已经把「头上测、一体一出」收成 `LoopForm::While`。`build.rs` 按这个枚举只写 `while` 或 `do/while`。javac `--release 8` 的 `for (int i = 0; i < n; i++)` 就是这条 `while`，前面多一次对同一槽位的无效果初值，体末是对该槽位的一次 `iinc` 或等价的加后存储，回边回到头。

写成 `for` 时，初值和更新从语句位置挪进头部。这只有在三件事同时成立时才不改变求值次数：初值在循环前恰好执行一次且没有别的效果；条件读的是这个槽位加上不变量；更新是体里最后一条效果，回边只经过它。缺任何一条就保持 `while`。不为此增加 `LoopForm`。`break` 仍只指向这个循环的出口，`continue` 只指向这条更新或头。

`OuterContinue.run` 把闩锁目标具体化：内层的 BCI 21 直跳外层 BCI 36 `iinc 2,1`，正常走完内层则先在 BCI 33 执行 `total += 100`，再到同一闩锁。若把 BCI 36 留在 `while` 体尾，`continue outer` 会跳过这次更新；若把 BCI 33 也挪进 `for` 更新，又会让该 `continue` 多执行 `+100`。因此投影只认领 BCI 36 的唯一更新，BCI 33 留在 body；条件跳转和所有正常回边须到达同一更新入口，独立出口不穿过它，跨层 `continue` 指向这个入口。更新及初值的真实 SSA 定义—使用、例外效果、词法作用域和所有入边必须与 header 的 Java 执行顺序一致；若把声明放入 `for` 初始化，退出后仍读取该局部的合法 class 必须拒绝这种缩小作用域的拼写，或先证明一个外层声明加头部赋值的等价形式。证明不全时保持原 `while` 或引用，不能仅凭 `iinc` 邻近猜测。

本地 JADX `LoopRegionVisitor.checkForIndexedLoop` 在建好 `LoopRegion` 后取 loop end 的最后指令，要求其结果仅进入一个双输入 phi、初值结果只有该 phi 一次使用、phi 结果参与条件，且归纳变量只在 loop 内使用；通过后隐藏 init/increment 并选 `ForLoop`。Jarde 可借其 SSA 使用次数和归纳变量身份约束，但还须用原 CFG 验证闩锁的**全部入边**及 `continue` 的精确目标，再原子地让 AST 头部认领那两条指令。JADX 对这一点的类型/区域启发式不能替代真实边和效果证明。

`i = i + step` 的 `javac --release 8 -g:none` 更新不是 `iinc`，而是闩锁里连续的 `iload i; iload step; iadd; istore i; goto header`。[独立冻结样本](../../evidence/java-syntax-2026-09-24/for-add-store-latch/analysis.md)中，最小类已完整恢复成行为正确的 `while`，说明运算和赋值的 AST 已够用；缺口在 `region.rs::for_header_candidate` 只接受 `Increment`。扩展现有 `ForHeader` 证明，不新增循环种类：更新 `Store` 的栈操作数须唯一来自同块的 `Add`，`Add` 两边须分别来自同块对归纳槽和只读步长槽的 `Load`；SSA 链与头部双输入 phi 的回边值须一致，链中无额外消费或效果；步长槽加入循环不变量集合，循环内有写入便拒绝。更新块除这条纯链和末尾回边不得夹带其它效果，全部入边仍只从循环体来；初始化、退出后作用域及头部测试继续走既有证明。`ForHeader.update_bci` 指向 `Store`，既有 Builder 在头部写 `i = i + step` 并在体内只认领这次更新，避免引入另一套表达式构建器。

复杂样本的外层更新入口 BCI 45 接受正常体尾和内层 `continue outer` 两条入边；BCI 42 的 `afterInnerCount++` 必须留在体内并被跳过。当前先在外层 BCI 8 报 `LoopShape`，随后整段因 quoted fallback 的局部作用域门禁拒绝。扩展更新证明后，还须验证现有区域走法是否能把 BCI 45 作为外层 `continue` 目标，输出可执行的 `for` 和带标签 `continue`；不能为绕过此结构问题而把标签边误写成 `while` 的普通 `continue`。这一复杂正例若仍失败，应记录独立的 3.1 区域缺口，3.2 的 add/store 子集仅按最小类和反例验收，不虚报完整 3.2。

### 2e. U+0000 坏在 Modified UTF-8 解码，不在转义

`controls` 的常量池是 `61 C0 80 62`，即一个 U+0000。`decode.rs` 的 `lossy()` 用 `String::from_utf8_lossy`，把 `C0 80` 读成两个 U+FFFD，`escape_string` 再写成 `\ufffd\ufffd`。标量转义改对了也救不回这个值。修复在 `lossy()`：按 JVMS 的 Modified UTF-8 解出标量，再交给现有转义。不改 reader 里的原始字节。`正在` 是合法 UTF-8，只受 `escape_string` 影响，可以先于这一处落地。

### 3. 命名 `catch` 与 TWR/monitor 互斥

typed `catch` 只适用于同时满足下面四条的异常表记录：

- `catch_type` 非 0，类型是该常量池 `CONSTANT_Class` 的内部名，不另选父类或 `Throwable`。
- 守卫检查对该区域的结论是「不是守卫」。TWR 或 monitor 规则只要声明自己拥有这块（领取或按其形状拒绝），本 change 就不把同一区域写成 `catch` 或 `synchronized`。
- 保护范围在正常流上是一段连续区域，处理器体是从处理器入口走出的一段区域。
- 同一保护范围上的多条命名记录按异常表顺序写成多个 `catch`。写不出体的那一条留作该 `catch` 内部的缺口，不省略这条 `catch`，也不写成空块冒充成功。

`catch_type == 0` 既可能是 `finally` 也可能是 catch-all。本 change 不区分它，一律保持今天的引用。这样不会把复制出去的 `finally` 写成一个看起来完整的 `finally`。

用户写的 `throw` 不是守卫。`athrow` 现在只在 TWR 或 monitor 证明了它是重抛时才被领走，其它一律引用，所以 `throw e` 和 `throw new` 都没有语句。操作数已经是一个值时，写成 `throw` 这个值。这个值是 `new`、`dup`、构造器、而且 `dup` 没有别的使用者时，写成 `throw new T(...)`。被守卫规则领走的 `athrow` 不再写第二条。不从这里推断 `throws`。这是一条语句，不是新的区域。

普通 `try/catch` 被误认成 TWR 时，修的是分类。`resources()` 现在只在保护范围前根本没有指令时跳过该行；每一行都是这样时返回 `NotGuarded`。`Guarded.one` 把范围从 `[6, 9)` 扩到 `[5, 9)` 之后，范围前的指令是 `invokestatic`，不是 `astore`。`initialisation()` 仍以 `ResourceInit` 拒绝，测试要求这个拒绝留下。所以「范围前不是 `Store`」不能一律当成不是资源头：范围被扩进初始化时，前面正好不是 `Store`，而这仍是没写完的 TWR。

范围前有一条普通赋值时，那一行不是资源头。只有这次 `Store` 的值来自同一条直线语句里的 `new` 或调用，才继续按资源检查。往前看语句时，上一条语句的结尾只是边界，不是「读不懂」。`open(); astore` 仍是资源头。范围被扩进初始化、前面不是 `Store` 的 TWR 仍拒绝。

`putstatic` 是另一种普通赋值边界，不能把它当作资源槽初始化。冻结的 [字段赋值对照](../../evidence/java-syntax-2026-09-24/catch-after-field-store/analysis.md)中，`fieldAssignment` 的 `bipush 7; putstatic field` 在受保护范围 `[5,9)` 之前完整结束，当前 `resources()` 却因前一条不是局部 `Store` 而报 `jre_guard_resource_init`；同形的 `localAssignment` 已完整恢复 catch。2.4b 只在现有 `single_statement` 能证明前缀是一条完整字段赋值、赋值链的栈值在范围入口不再存活、且范围从该语句之后开始时，让此行交给 `catches()`。字段写入仍由既有字段语句构造器在 try 之前写出，不复制进 try。若范围从赋值链中间开始，或前驱链不能界定为完整语句，保留原有 TWR 拒绝。特别是 `a_handler_range_that_swallows_the_initialisation_is_refused` 把范围扩到初始化中间，范围前的 `invokestatic` 虽也不是局部 Store，却必须继续拒绝；不能把所有非 Store 前驱一律跳过。此边界只复用现有 Guard 的语句/SSA 事实，不新增 catch 或资源机制。

这条赋值和 `try` 经常在同一个基本块里。分类成「不是守卫」还不够：块的前半段要写在 `try` 前面，不能写进保护范围，也不能再写一遍。这是同一条 `try` 的 lead，不是新的区域种类。

两条记录从同一条指令开始、结束位置不同，是嵌套，不是一条 `try` 的两个子句。`catches()` 现在直接返回 `None`，外层处理器就变成没人走的块。`Region::Try` 的体本身可以再是一个 `try`，不需要新的区域种类。较窄的范围是内层，它的处理器落在较宽的范围里面；外层的体就是这个内层 `try`。范围互相交叉、谁也不包含谁的，仍不写 `try`。同一范围的多条子句保持今天的写法，不得把嵌套收成并列的两个 `catch`。

`try` 体里的调用仍被整块引用，因为这块还有异常边，而 `NormalFlowView` 不包含异常边。这条边已经是外面那个 `catch` 的入口，不是没人认领的出口。块里的语句照写，边不跟进去。不把异常边并进正常流。`catch_type == 0` 的边仍引用，子程序入口仍引用。不在 `try` 里面的块仍引用。

Java 9 的 `try (r)` 没有 `new`。它先把已有局部复制到另一个槽，保护范围里是方法体，正常路径用 `ifnull` 跳过 `close`，`close` 之后直接落到 `ifnull` 的目标，没有 `goto`。当前库不把这次复制当成资源头，于是命名的 `Throwable` 行被写成用户 `catch`，处理器里的 `close` 和 `addSuppressed` 也被写出来。这不是源码。必须在进入 `catch` 之前认成原来的资源头：`try (java.io.Reader local1 = arg0)`，表达式是这次复制读到的局部，`arg0.read()` 在体内，`return` 在语句之后。`close` 作用在被复制的槽上。体里若再写原来的槽，就关的不是同一个对象，必须拒绝。不得留下 `catch (Throwable)` 或 `addSuppressed`。复制局部但后面不是这条关闭形状时，仍是普通 `catch`，不得整段拒绝。已有的 `goto` 形资源头保持不变。

### 2j. 异常表声明的行不依赖抛点

`cfg.rs` 只在 `may_throw` 的指令上建异常边，`canonical.rs` 的 `handler_rows` 又只从抛点读 `protected`。`try { n = n * 2; } catch (IllegalStateException e) { n = -2; }` 的保护范围里一条能抛的指令都没有，这条行在图里就不存在。后果取决于偶然的块布局：第二个顺序 `try` 的处理器字节恰好没有任何块覆盖，`unaccounted_instructions` 把整个方法引用；`catch` 里嵌 `try` 的那个方法，处理器字节恰好被某个不可达块盖住，`uncovered` 跳过不可达块，`catch` 子句无声消失——文本只写正常路径，没有任何标记。后者是正确性问题：呈现声称了一个不是字节码程序的程序。

修复在图这一层，不加新区域。异常表是工件自己的声明：每一行保护范围覆盖了块，就有一条从被覆盖块到处理器的 `Exception` 边，处理器入口是块边界，与范围里有没有能抛的指令无关。`throw_sites` 仍是运行时事实，只记真实抛点，不为无抛点的行发明条目。`handler_rows` 的 `protected` 改从边读——图对保护范围的声明只有一个来源。帧和 SSA 的处理器入口态本来就从 `Exception` 边读（`exception_inputs`），边存在后整套机制照常工作，`catches` 读的是解码表，不需要改。真实抛点的行为不变：有抛点的行今天怎么建边，之后也怎么建。费用与计数维度会动（`IrEdges` 增多），按实测重钉。

`StmtKind::Try` 目前只有 `resources` 和 `body`。命名 `catch` 要在这个节点上加 catch 子句，空的 `resources` 表示不写 `try (`。不另造一套异常图。`catch_type == 0` 仍不写成 `catch` 或 `finally`。

### 4. 循环出口与 `for` 都要有单独的证明

`break` 只对应目标就是该循环出口块的普通边。`continue` 只对应目标就是该循环头部或闩锁的普通边。目标在循环外但不是出口的边，仍是缺口，并且不得把整段循环收成引用。

`for` 只在头测循环上写，且同时成立：循环前恰好一条对某个局部的无副作用初值；头部测试只读该局部与循环不变量；闩锁恰好一条对该局部的更新。缺任何一条就写现有的 `while` 或 `do-while`。不从 `while` 猜 `for`。

### 5. 使用点改写是类装配，不是第二套恢复

单方法恢复继续按方法跑。accessor 调用点的改写沿用 `accessor@1`：公开入口按需读取 callee 的声明与 Body，证明是纯字段转发才写成字段读写；否则保留 `access$N(...)` 和拒绝原因。不删除 accessor 方法。

lambda 与匿名类的拼接放在 `class_source` 的装配，材料是各方法已经产出的语句，加上 reader 已解析的 `InnerClasses` / `EnclosingMethod`。不在单方法载荷里重建这些类级事实。

- lambda：本地 JADX 1.5.6 的 `CustomLambdaCall` 值得借鉴两步：按当前站点的 `LambdaMetafactory` bootstrap 链识别函数形状；复制合成方法体时把工厂捕获实参和 SAM 参数按实现方法的槽位重新绑定，体在箭头内生成，不在创建时执行。但它对同类 synthetic 实现方法直接 `DONT_GENERATE`，没有类级全使用点及完整正文证明；Jarde 不隐藏该方法。单方法 `lambda@1` 已严格保存 bootstrap/适配事实，不过可选的 `LambdaRecord` 仅在 `RuleDetails` 下物化，装配不能依赖它。需从同次恢复另交接每个 indy 站点、被拒站点、目标方法和可复制的 AST/值流，类内所有有 Code 的方法完整扫描后按目标分组，逐槽证明捕获/SAM 参数、适配和创建时求值；合成方法正文还须 `quality=structured`、无 fallback/引用缺口。现有 `ExprKind::Lambda` 只装一个表达式，单表达式正文可沿用；多语句体确实需要最小的 lambda 块体 AST，不能从已发射字符串剪贴。所有该目标的站点能原子改写才加「体已在使用点呈现」标记；任何站点未改写则不加。方法物理身份、正文和来源始终保留；绝不因为它叫 `lambda$` 或 synthetic 就先删除。

  `lambda-body-inline` 的完整 Jarde 源码还揭示独立的命名边界：哪怕箭头只是调用原 helper，javac 也会给它生成同名 `lambda$build$0`，与保留的方法声明冲突。类装配只要为同类 synthetic `lambda$` 实现方法发出箭头，就在发射前按物理方法身份分配确定性的无冲突源码别名，统一改写该方法声明与所有已证同类直接调用；无须预测 javac 对箭头的编号，不从字符串做替换，也不改变方法恢复报告里记录的原名。`ClassSourceMethod` 当前持有已发射声明/正文字符串，`ExprKind::Call` 只存写出的名字而非物理目标；枚举 switch 的同次 AST 重发路径可供参考，但这里还须交接调用指令 BCI 到真实 `Methodref`/物理目标的映射，在 AST 节点来源 BCI 上核对后改写，不能碰同名其它重载、其它 owner 或字符串文字。别名表按物理方法身份键控，与该类全部可写方法名检查冲突，再原子重发受影响成员；任一目标或来源无法对应就不局部改名。原物理 helper 仍有正文和可追溯性。若某调用无法按同一身份重写，则留下明确编译限制。JADX 把 synthetic helper 隐藏可以绕过冲突，但其完整性判据不足，不能照搬。
- 匿名类：`inner_name` 为空、`EnclosingMethod` 指向当前方法，只是候选身份。调用者类全部方法的同次 BCI 扫描只是唯一性的第一层：还须在所选物理输入范围和解析环境内排除所选范围内其它位置（包括调用者自身和其它类）对同一物理匿名类的构造与可观察类身份使用。范围的 XRef/Code 覆盖不完整、符号目标无法确认或兄弟 class 身份不唯一时拒绝；不把当前类生成文本或一个方法的使用者集合当成全局唯一性。候选方法必须 `quality=structured` 且无引用缺口；`contains_statements` 只是「有语句」，不能当成完整。再核对唯一构造器的实参流：每个参数必须恰好进入按原描述符调用的直接基类 `super(...)`，或恰好写入一个只作捕获的合成字段。构造器除此之外的效果、额外字段写入和实例初始化必须有等价位置，否则拒绝；合成字段的每一次读取都要映射回该参数及分配点词法范围内同一个、可稳定读取的捕获变量/接收者，不得按 `val$`/`this$0` 名字或类型猜。匿名语法的 `new Base(args) { ... }` 只携带原基类构造实参，接口实现的 `new Interface() { ... }` 不携带实参；捕获值在创建时的求值和异常顺序不能被删、复制或推迟。基类、构造重载、捕获替换、成员正文、非捕获字段或实例初始化任一未证明，则不内联，使用点和该类各自保持诚实呈现。只有当分配点值流证明被捕获值确实是当前词法外层接收者、且新源码的词法外层就是它时，内联正文才能用 `Outer.this` 表达其字段读取；否则不得发明它。

  这里需要的机制是**同次恢复的类装配侧车**，不是第二套方法恢复，也不是对已经发射的 Java 文本做搜索替换。`recover_for_class_source` 已经为接口初始化和枚举 `switch` 暂存同次 AST/证明，类装配可按这个边界暂存候选 `new` 的精确 BCI、构造调用的 SSA 实参及调用者未发射的结构化语句。不能从 `RecoveryReport.news` 推断分配点：那组记录仅在请求 `RuleDetails` 时物化，改变证据选择不应改变源码。兄弟类经现有环境解析得到唯一物理身份后，其成员只走现有恢复入口各一次；若完整正文或所需值流拿不到，就拒绝本次投影，不再从它的字符串正文逆解析字段访问。逐个 `Fieldref` 读取和构造器参数槽建立映射，再在 AST 上替换已证的捕获读取；调用者的 `new` 也在同次 AST 上替换成匿名类表达式，由现有 emitter 一次写出。若 Java 匿名类花括号无法用现有表达式节点表示，只增这一种语法节点；不为此增区域、IR 表或通用跨类重写框架。类源码方法文本可沿用枚举 `switch` 的原子投影路径，保留原物理方法的恢复报告，不把改写后的文本冒充原单方法报告或原 source map。

  唯一性证明要数同轮 `Operations::Allocate` 中所有指向该兄弟类的分配，包括 `new@1` 拒绝的候选和被其它规则预留的分配。`Sites` 中只有一个已验证项只能说明「至少有一个可写使用点」，不能说明全类唯一。调用者任一有 `Code` 的方法未完成解码/扫描，计数就是未知，整个候选拒绝；空 sidecar 只有在该方法完整扫描后才是「确实没有分配」。这一步独立于匿名正文是否能恢复，避免把未见到的第二处分配误当不存在。之后复用 P1 已有、带覆盖信息的 Code/metadata XRef，对所选物理输入范围里调用者及其它类核对候选站点以外的 `new`、构造器句柄及类身份引用；其目标须在同一解析环境中绑定到候选物理类，任何实际第二构造或身份使用都拒绝，扫描停止、范围不完整或可能指向该类却无法解析时也拒绝。不为此建立新的全局索引或假称排除输入范围外的开放世界调用者。

  现有 `class_source` 只为描述符可拼写且有 `Code` 的成员运行正文；唯一性证明却要覆盖调用者**所有**有 `Code` 的物理成员。因而类级聚合须把未运行、准备失败、描述符不可拼写、扫描 `complete=false` 分别视为未知，不能把 sidecar 中没有记录解释为零。下一步解析兄弟匿名 class 时复用现有 `resolve_symbol` 的环境/物理身份核对与 `read_definition` 的唯一确认读；当前通用 `resolve_class_source_dependency` 只回传 `ClassMemberFacts`，会丢掉解析 `InnerClasses`/`EnclosingMethod` 所需的原始字节。匿名专用读取应从同一次 `ConfirmedRead` 的字节经 reader 的 typed attribute parser 取得这两项，按现有预算计费，不从 `$1` 命名或已发射文本逆推，也不为此复制一套解析器。

  `outer.new Base(1) { ... }` 还有另一层参数语义：当 `Base` 是非静态成员类，`javac --release 8` 给其二进制构造器加外层实例参数；匿名子类构造体可先对该实例调用 `Objects.requireNonNull`，再调 `Base.<init>(Outer, int)`。这不是源码里的第一个普通实参，也不是匿名体捕获字段。5.3 当前只承诺无需限定接收者的 `new Base(args)`，遇到这种基类就拒绝；待单独证明限定接收者语法与空值检查的求值点，才能扩展。

  本地 JADX 1.5.6 的 `ProcessAnonymous` 检查唯一构造器/使用者；`AnonymousClassVisitor.getArgsToFieldsMapping` 用 SSA 单次使用找到合成字段存储或 `super` 调用，可借鉴这种“构造参数有用途证书”的顺序，但其 `startArg` 仅因首参类型等于外层类就跳过，未证明这个值在内联正文如何保留。root 的 Java 8 `AnonymousProbe` 对照中，原 class 输出 `10`；JADX 生成 `new Action(this) { ... }` 且 `this.this$0 = this`，`javac` 报接口匿名类不能有参数及接收者类型不匹配。Jarde 当前保持 `new AnonymousProbe$1(this, local2)`，未把它冒充为已内联；即使把同一组原 class 放在 classpath，`javac` 对源级匿名类仍隐藏合成构造参数，不能由“二进制构造器存在”推导出可编译源码。需要本条捕获/基类证书后才满足 5.3，不能照搬 JADX 的改写文本。

  另外，JADX 的 `ProcessAnonymous.checkUsage` 读的是 `ctr.getUseIn()` 的调用**方法集合**；`UsageInfo` 用 set 收集它，`size()==1` 不能证明同一方法里只有一条构造调用。这里需要逐分配 BCI 的计数，不能借用 JADX 的调用者数量判据。

### 6. 声明与字面量只改拼写来源

- 类的二进制名含 `/` 时，最后一段之前是 `package`（`/` 写成 `.`），最后一段是简单名。简单名里的 `$` 保留。默认包不写 `package`。
- `throws` 的类型与顺序等于该方法 `Exceptions` 属性里的类名。属性不存在则不写 `throws`。不从 `athrow` 推断。
- 方法 access flags 的 `0x0080` 是 `ACC_VARARGS`，不是字段的 `ACC_TRANSIENT`。末参类型是数组时写成 `T...`。标志置位但末参不是数组时，不发明 `...`，按描述符写，并加 `// jarde:` 标记。
- `escape_string` 对已解码的 Unicode 标量：不是引号、反斜杠、行终止符（LF、CR、U+2028、U+2029）、不是 U+0000–U+001F 与 U+007F 的，按字符写入。孤立代理项仍 `\uXXXX`。这是对本 crate 现有 emoji 测试的有意替换。

不新增第三方库。jadx 1.5.6 只在验收对照里作为外部见证，不进入生产依赖，也不进入「文本必须与 jadx 相同」的断言。

### 7. 验收对照的三条文本

每个 P01–P08 场景都是本仓库里的一份 Java 源码，由本 change 编写，不从 Behinder 或其它第三方工程裁剪。`javac` 的 `--release` 与 jadx、jarde 的版本写进对照记录。同一组 class 文件分别交给 jadx 与 `class-source`。

对照记录对每个场景保留四样东西：源码摘录、jadx 摘录、jarde 摘录、结论。结论只允许三种：

- **jarde 对齐源码**：要求写出的结构在 jarde 文本里与源码一致（允许没有 import、接收者写成 `this`、简单名这些规格内的拼写差异）。
- **jarde 按规格留下缺口**：该场景的要求明确允许缺口，jarde 的引用指名了对应 BCI，并且没有写成空结构。
- **失败**：要求写出的结构缺失，或 jarde 抄了 jadx 相对源码的偏离（外部符号、空 `switch`、把 TWR 写成用户 `catch`、把 `catch_type == 0` 写成 `finally`）。

jadx 与源码不一致时，记录差异，不据此要求 jarde 跟随 jadx。Behinder 上的计数可以附在 `verification.md`，不能代替这组场景。

## Risks / Trade-offs

- 大量既有文本断言会变。更新时只接受本 change 写明的差异（前缀留下、单臂 `if`、命名 `catch`、`for`、字段访问、使用点的体、`package`/`throws`/`...`、标量字符）。不得放宽「不可约 CFG 仍整段引用」「TWR 失败不写成 `catch`」「空结构不算成功」这些断言。
- 类装配才看得到 `Exceptions` 与 `InnerClasses`。若从单方法恢复文本里要求 `package` 或匿名类花括号，会迫使恢复层拥有类级事实。规格把这两件事限定在类源码视图。
- 内联 lambda 若重排语句，会改变求值顺序。装配只替换使用点表达式，不把体里的语句搬到调用点之前。

## Migration Plan

按 tasks 的顺序做：先让缺口局部化的 fixture 变红再变绿，再加 `catch`、循环、accessor、装配期拼接、最后改拼写。每一步都不依赖下一步才能单独验证。不保留旧拼写开关。

## Open Questions

无。TWR 失败不降级为 `catch` 已按既有规格定死，不留给实现时再选。

`while (a > 0 && b > 0)` 的字节码是头部一条分支链：每个假边都离开循环，最后一条的真边进体。这个形状只有一个结构化拼写——复合条件本身；`while (true) + break` 是另一种字节码，不会被误认。jadx 实测也不还原它（写成 `while(true)+if break` 且条件取反，或把 `||` 循环改成提前 return），所以恢复复合条件即反超。与非目标「不折叠 `&&`」的界线在位置：值位与 `if` 位的多分支折叠仍禁止——那里同一种字节码有多种源码拼写，选 `&&` 是猜；循环测试位的分支链没有第二种拼写，选它不是猜。`||` 对偶：每个真边都进体，最后一条的假边离开。混用两种算子、任一条件证不了、或某条分支两边都不离开链时保持整段引用——优先级与括号没有字节码事实支撑。条件表达式为此增加 `&&`/`||` 两个二元操作，只在这个位置使用；`do-while` 的闩锁同样适用（`doWhileComplex` 的链在闩锁上）。
