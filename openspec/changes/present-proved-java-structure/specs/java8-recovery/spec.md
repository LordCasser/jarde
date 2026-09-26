## ADDED Requirements

### Requirement: A proved prefix survives a local gap

一次方法恢复里，走法已经证明为语句的块 MUST 作为语句进入产物。当前形状证明不了时，fallback MUST 只引用该形状自己的块与未认领的臂，MUST NOT 把已证明前缀的指令放进 fallback 的 BCI 集合，也 MUST NOT 因此停止对已证明汇合点之后的走法。没有证明过的汇合点时，走法 MUST 停止扩展，其余活块仍由既有的未覆盖引用命名。

每个活块 MUST 恰好属于一个区域：结构化区域或一条 fallback 引用，二者不得重叠，也不得遗漏。图没有覆盖、也没有标成不可达的指令，MUST 继续走既有的整方法引用，MUST NOT 因本要求变成半段语句。`content` MUST 仍只有 `not_produced`、`explanation_only`、`contains_statements`。产物里至少有一条语句时 content 为 `contains_statements`，同时存在 fallback 时 quality 保持 `Fallback`。只有引用、没有语句的方法 MUST 仍是 `explanation_only`。

MUST NOT 用空的 `if`、空的 `catch`、空的 `switch` 或空的方法体代替一条引用。MUST NOT 引入字节码和常量池里不存在的类型或常量。

#### Scenario: A branch that cannot be proved keeps the statements before it

- **WHEN** 一个方法在一条证明不了的分支之前已经有可呈现的赋值或调用，且该分支没有证明过的汇合点
- **THEN** 产物含这些语句，fallback 的 BCI 集合不含这些语句的 BCI，`content` 为 `contains_statements`，`quality` 为 `Fallback`（P01）

#### Scenario: An unaccounted body stays quoted whole

- **WHEN** 解码出的指令既不属于任何规范块，也没有被标成不可达
- **THEN** 产物 MUST NOT 把其余块呈现成看起来完整的方法体，引用 MUST 指名这些指令的 BCI（P01）

#### Scenario: An empty structure is not success

- **WHEN** 一个 `switch`、`catch` 或 `if` 的臂没有被证明为语句，也没有被证明为空臂
- **THEN** 该区域 MUST 是带 BCI 的引用，MUST NOT 呈现为 `switch (…) {}`、空的 `catch` 或没有条件的空 `if`（P01）

### Requirement: A one-armed branch and a proved exiting arm are if statements

两路分支的一个后继就是该分支的直接后支配点时，恢复 MUST 写成没有 `else` 的 `if`，空的那一臂 MUST NOT 再以 `jre_region_arms_do_not_meet` 拒绝。另一臂的区域证明每条路径都是 `return` 或 `athrow`、且没有落到第三个块时，恢复 MUST 写成 `if/else`，并且该语句之后 MUST NOT 再有汇合来的语句。证明不了整臂退出时，MUST 按局部缺口处理该分支，前缀语句仍然保留。

#### Scenario: One successor is the post-dominator

- **WHEN** 分支的一个后继就是它的直接后支配点，另一臂是可呈现的语句
- **THEN** 文本含 `if` 且不含 `else`，`content` 为 `contains_statements`，该分支不再是 `ArmsDoNotMeet`（P02）

#### Scenario: A forward tail reached from both arms is the join

- **WHEN** 分支的一个后继不是循环头，它的每条入边都来自该分支支配的前向块，且另一臂也能只经前向边走到它
- **THEN** 该块写在这个 `if` 之后且只出现一次，两臂里走到它的边不再写成 `else` 中的第二份，也 MUST NOT 以可重入循环拒绝；文本 MUST NOT 为此把两个条件收成 `&&` 或 `||`（P02）

#### Scenario: An arm that is not a proved exit stays a gap

- **WHEN** 一臂到达汇合点，另一臂既不是汇合点，也没有被证明为每条路径都 `return` 或 `athrow`
- **THEN** 产物 MUST NOT 含把该臂写成 `else` 的 `if`，分支前已证明的语句 MUST 仍在（P02）

### Requirement: A proved Boolean value chain preserves short-circuit evaluation

一个值位的分支链已经由 `ShortCircuitValue` 及 CFG/SSA Phi 证明、两个唯一生产叶严格为 `iconst_1` 和 `iconst_0`、返回消费者的 Java 类型已证明是 `boolean` 时，恢复 MUST 按原分支极性及求值顺序写出布尔值。可等价化简的返回链 MUST 使用 `&&`、`||` 或原条件本身，MUST NOT 继续把这类精确 0/1 值写成整数三元式后附 `% 2 != 0`。普通语句位 `if` 的共享前向汇合不受此要求影响，MUST 保持原语句结构；不得仅凭字节码声称找回作者原来的源码拼写。生产叶不是严格 0/1、返回类型不是已证明的 `boolean`，或链与 Phi 证明不闭合时，MUST NOT 按布尔常量简化；任意整数进入 `Z` 消费位时 MUST 保持 JVM 低位语义。其它已证布尔消费位由下一要求处理。

#### Scenario: Pure and effectful Boolean returns

- **WHEN** Java 8 编译的 `BoolValue` 有 `a > 0 && b > 0`、`a > 0 || b > 0` 以及两项依次调用 `positive` 的对偶返回表达式，且每个消费者是 `ireturn Z`
- **THEN** 四个返回值分别含 `&&` 或 `||`、无整数三元和 `% 2 != 0`；`positive` 的调用顺序和次数与原 class 相同，测试、生产叶、汇合和返回 BCI 仍可由 source map 查询，整类 Java 8 重编和执行对照一致（P02/P11）

#### Scenario: An unproved Boolean leaf is not simplified

- **WHEN** 同形状分支链的一个生产叶改成 `2`，或消费者是 `int`，或 Phi/边的唯一性证明失败
- **THEN** 文本 MUST NOT 使用该链的布尔 `&&`/`||` 规范化；若能继续呈现原始整数值进入 `Z` 消费位，MUST 保留 `% 2 != 0`，否则按原拒绝路径引用 BCI（P11）

### Requirement: A proved Boolean value chain remains Boolean at typed stores and calls

同一 `ShortCircuitValue` 的 CFG/SSA Phi、两个严格 1/0 生产叶、唯一消费者与布尔类型均已证明时，静态或实例 `Z` 字段写入、`boolean[]` 元素写入、`(Z)V` 调用参数和已判定为布尔的局部声明 MUST 复用返回位的短路布尔表达式。可等价化简的链 MUST 保留原极性与调用次数，MUST NOT 再套数值三元或 `% 2 != 0`。实例接收者、数组引用和下标仍按字节码先于右值求值；短路右项只在原分支允许时执行。生产叶不是严格 1/0、消费者类型或唯一性未证明时，MUST 保留已有低位转换或局部引用，不得依据目标声明猜测布尔链。

#### Scenario: Proven Boolean sinks keep their source evaluation order

- **WHEN** 冻结 Java 8 类 `MixedBooleanField`、`MixedShortCircuitField`、`MixedArrayValue`、`MixedBooleanArgument` 和 `MixedBooleanLocal` 分别将 `(a && b()) || c()` 或 `(a || b()) && c()` 消费于已证明的字段、数组、调用和局部位置
- **THEN** 相应语句 MUST 含 `&&`/`||` 而不含整数 1/0 三元与 `% 2 != 0`；字段接收者、数组引用、下标和 `b()`/`c()` 的调用次数、顺序、异常行为与原 class 一致；测试、生产叶及消费 BCI 有 source map，完整类经 Java 8 重编的 Runner 输出与原 class 一致（P02/P11）

#### Scenario: An unproved store leaf retains its low-bit interpretation

- **WHEN** 同形状链的一个生产叶改为 `2`，或字段描述符、数组元素、调用参数、局部决定并非已证明布尔，或 Phi 的唯一消费证明失败
- **THEN** MUST NOT 输出该链的逻辑 `&&`/`||` 投影；原路径能呈现时保持整数低位转换，否则引用未证明的 BCI（P11）

### Requirement: A branch that exits a loop is represented before the loop is published

循环体内的分支有边指向该循环的已证明出口时，该边 MUST 由循环条件、显式的 `break` 或局部引用保留。区域覆盖所有自然循环块并不足以证明出口语义；若没有一种 Java 结构能保留这条边，MUST 引用该循环，MUST NOT 输出无缺口标记却改变退出行为的 `while`。

#### Scenario: The second conjunct exits the loop

- **WHEN** `while (a > 0 && b > 0)` 编译出的 BCI 3 与 7 均向 BCI 23 循环出口跳转，且体内更新只在两个条件都为真时执行
- **THEN** 对 `(a=1,b=0)`，若方法被标成 `Structured`，恢复源码经 Java 8 重编后 MUST 立即返回原 class 的 `0`；未证明复合头或显式退出时 MUST 保留引用，MUST NOT 写成没有 `break` 的 `while (a>0) { if (b>0) { 更新 } }`（P02/P04）

### Requirement: An ordinary array access is an index expression

`iaload`、`iastore`、`arraylength` 与 `newarray` MUST 按该指令自己的操作数呈现，MUST NOT 只在枚举 `switch` 的分派表被证明时才允许出现。读取 MUST 写成 `array[index]`，写入 MUST 写成 `array[index] = value`，`arraylength` MUST 写成 `array.length`，`newarray` MUST 写成 `new T[length]`，元素类型取该指令的 `atype`。`enumswitch@1` 已经认领的 `iaload` MUST 保持今天的分派表拼写，普通读取不得抢这条证明。

一条存储被引用而其后的读取仍被拼成名字时，产物 MUST NOT 出现没有声明的局部名。该读取 MUST 留在同一条引用里，或者该存储必须作为语句留下。`aaload` 及其他元素宽度不在本要求内，仍按既有 `Other` 引用。

#### Scenario: An int array read and write are index expressions

- **WHEN** 字节码对一个 `int[]` 做 `iaload` 或 `iastore`，且该 `iaload` 不是被 `enumswitch@1` 认领的分派表
- **THEN** 文本含 `array[index]` 或 `array[index] = value`，不含「不是分派表」的引用（P10）

#### Scenario: A quoted store does not leave a bare local name

- **WHEN** 某个槽位的 `istore` 被引用，其后同一槽位的 `iload` 仍在产物里
- **THEN** 文本 MUST NOT 含该槽位一个没有声明的 `localN`（P10）

`laload`、`faload`、`daload`、`aaload`、`baload`、`caload`、`saload` 以及对应的 `*astore` MUST 使用与 `iaload`/`iastore` 相同的下标表达式，MUST NOT 新增区域或节点。`baload` MUST NOT 凭 opcode 写成 `boolean`；只有数组引用的类型已证明为 `[Z` 时，该值才可作为 `boolean` 证据。数组类型未证明时，返回 `boolean` 的成员 MUST 保持拒绝。`byte`、`char`、`short` 的读取 MUST NOT 添加字节码中不存在的转换指令。`anewarray` MUST 复用同一个数组创建节点，元素类型只取该指令常量池中的类，写成 `new T[n]`。只有创建之后的存储下标正好是 `0..长度-1`、这些 `dup` 没有别的使用者、留下的值就是这个数组时，才 MUST 写成 `new T[]{...}`。这个表达式可以被返回、存进局部，或作为调用参数。可变参数调用点 MUST 写成 `args(new int[]{1, 2, 3})`，MUST NOT 猜成 `args(1, 2, 3)`。其它创建 MUST NOT 写成花括号。`aaload` 与 `aastore` MUST 复用下标表达式。`aaload` MUST NOT 呈现成 `int`；数组类型已证明为 `T[]` 时元素类型是 `T`，否则 MUST NOT 声称类型。

循环测试中的 `arraylength` 和上述读取是条件所读的值，MUST 可以留在测试里。存储和 `iinc` 仍 MUST 拒绝该测试。此放宽 MUST NOT 新增循环形状，也 MUST NOT 把计数循环写成 `for-each`。

#### Scenario: A boolean array read is not guessed from baload

- **WHEN** `baload` 的数组引用类型没有被证明为 `[Z`
- **THEN** 文本 MUST NOT 把该值写成 `boolean`；返回 `Z` 的成员保持拒绝（P10）

#### Scenario: A latch test that only reads an array length is do-while

- **WHEN** 循环的唯一测试在闩锁上，回边指向头部，且测试中除分支外只有数组读取和 `arraylength`
- **THEN** 文本含 `do` 与 `while`，条件中含 `.length`，且 MUST NOT 含 `for (`（P10）

### Requirement: A named catch is written only when the region is not a refused guard

异常表记录的 `catch_type` 非 0，且守卫检查认定该区域不是 try-with-resources 或 monitor 时，恢复 MUST 把保护范围写成 `try`，把处理器写成 `catch`，类型 MUST 是该 `catch_type` 的类名，不得改成其父类或 `Throwable`。同一保护范围的多条此类记录 MUST 按异常表顺序出现。两条记录指向同一个处理器入口时 MUST 写成一个 `catch (A | B …)`，处理器体只走一遍，MUST NOT 写成两个子句。指向不同入口的记录仍是多个子句。某一条处理器体证明不了时，该 `catch` MUST 仍出现，其体为指名 BCI 的缺口，MUST NOT 省略这条 `catch`。两条记录从同一条指令开始但结束位置不同、且较窄记录的处理器落在较宽的范围里时，MUST 写成嵌套的 `try`：外层的体是内层 `try`，MUST NOT 收成同一个 `try` 的并列 `catch`。范围互相交叉时 MUST NOT 写 `try`。

`catch_type` 为 0 的记录 MUST NOT 写成 `catch` 或 `finally`。try-with-resources 或 monitor 规则声明拥有的区域，无论它领取该区域还是按其形状拒绝，MUST NOT 被改写成 `catch`。普通 `try/catch` 的正面形状对不上 TWR 时，守卫检查 MUST 结论为不是守卫，使本要求适用。无法证明的 TWR close/suppress 顺序 MUST 继续按既有 TWR 场景降级，而不是变成普通 `catch`。

#### Scenario: A typed handler becomes catch

- **WHEN** 异常表有一条非 0 的 `catch_type`，保护范围与处理器体都可按正常流呈现，且该区域不是 TWR 或 monitor
- **THEN** 文本含 `try` 与 `catch (该类型 …)`，处理器体里的语句出现在该 `catch` 中（P03）

#### Scenario: A complete field assignment before a typed catch is not a resource

- **WHEN** 一条 `putstatic` 字段赋值及其全部操作数在非 0 `catch_type` 的保护范围之前作为单条语句结束，范围入口没有该赋值的遗留栈值，且保护范围与处理器体均可呈现
- **THEN** 文本 MUST 在 `try` 前写该字段赋值一次，并写出原异常表命名的 `catch`；MUST NOT 报 `jre_guard_resource_init` 或把该赋值放进资源头（P03）

#### Scenario: A range that cuts through resource initialization remains refused

- **WHEN** 保护范围从资源初始化链中段开始，以致前一指令不是局部 Store，但前缀不能证明为完整字段赋值语句
- **THEN** TWR 规则 MUST 保留原有资源头拒绝，MUST NOT 把合成处理器改写成用户 `catch`（P03）

#### Scenario: A refused try-with-resources is not spelled as catch

- **WHEN** TWR 规则识别出资源与 close 链，但 close/suppress 顺序证明不了
- **THEN** 该区域保持引用，文本 MUST NOT 把这些处理器写成用户的 `catch`（P03）

#### Scenario: Two types on one handler are one multi-catch

- **WHEN** 同一保护范围的两条非 0 记录指向同一个处理器入口，且类型不同
- **THEN** 文本含一个 `catch (A | B …)`，两条类型按表序出现，处理器体只出现一次（P03）

#### Scenario: A narrower range inside a wider one is a nested try

- **WHEN** 两条非 0 记录从同一条指令开始，结束位置不同，且较窄记录的处理器落在较宽的范围里
- **THEN** 文本是外层 `try` 的体里再有一个 `try`，两个 `catch` 各属一层。MUST NOT 把它们写成同一个 `try` 的并列子句（P03）

#### Scenario: A zero catch type is not finally

- **WHEN** 异常表记录的 `catch_type` 为 0
- **THEN** 文本 MUST NOT 含对应的 `finally` 或 catch-all `catch`（P03）

### Requirement: A call inside a presented try is written

`try` 体里的一条语句 MUST 被写出，即使它所在的块还有异常边。这个条件只在每条异常边的处理器都是覆盖该块的命名 `catch`（`catch_type != 0`）时成立。异常边 MUST NOT 被并进 `NormalFlowView`，也 MUST NOT 被当成后继走进处理器。`catch_type == 0` 的边、子程序入口、没有被这些 `catch` 覆盖的边，以及不在 `try` 里的块，MUST 保持今天的整块引用。

#### Scenario: The call a nested try protects is written inside it

- **WHEN** 内层 `try` 的体是 `invokeinterface java/lang/Runnable.run`
- **THEN** 文本在内层 `try` 里含 `arg0.run()`，且不含 `@bytecode 0`（P03）

### Requirement: An unclaimed throw is a throw statement

没有被 try-with-resources 或 monitor 领走的 `athrow` MUST 写成 `throw`。操作数是已证明的值时，文本 MUST 是 `throw` 这个值。该值是一次没有别的使用者的 `new` 加构造时，MUST 写成 `throw new T(...)`。被守卫规则领走的 `athrow` MUST NOT 再写一条用户 `throw`。MUST NOT 从 `athrow` 推断 `throws`。

#### Scenario: A rethrow of the caught value is throw

- **WHEN** `catch` 的处理器是 `astore` 之后的 `aload; athrow`，且这条 `athrow` 没有被守卫规则领走
- **THEN** 文本含 `throw local1`，MUST NOT 把这条 `athrow` 留成引用（P03）

#### Scenario: A constructed exception is throw new

- **WHEN** `athrow` 的操作数是同一次 `new` 与构造器，且 `dup` 没有别的使用者
- **THEN** 文本含 `throw new java.lang.IllegalArgumentException()`（P03）

### Requirement: A switch whose join only returns is a return in each arm

`switch` 的汇合块只有一条 `return`，且每臂转到该汇合前恰好留下一个值时，该臂 MUST 以 `return` 这个值结束，汇合处 MUST NOT 再写一次。默认臂直接落到这条 `return` 时同样写进该臂。臂里已有的存储 MUST 保留。MUST NOT 写成 `return switch`，MUST NOT 发明局部变量。某一臂没有留下恰好一个值，或汇合处还有别的指令时，MUST 保持今天的引用。这不是 case 穿透。

#### Scenario: Each arrow arm returns its own constant

- **WHEN** `return switch (n) { case 1 -> 2; case 2 -> 3; default -> 0; }` 被收成每臂一个常量再汇合到同一条 `ireturn`
- **THEN** 文本含 `return 2`、`return 3` 和 `return 0`，且这是仅有的三条 `return`，不含 `return switch`（P11）

#### Scenario: A stored arm value is returned after the store

- **WHEN** 一臂先把 `arg0 + 1` 存进局部再把该局部留到汇合的 `ireturn`
- **THEN** 文本含 `int local1 = arg0 + 1` 和 `return local1`，MUST NOT 是 `return arg0 + 1`（P11）

### Requirement: A stated catch is never dropped silently

异常表声明的行 MUST 是图的事实：保护范围覆盖了块时，MUST 有从被覆盖块到该行处理器的 `Exception` 边，处理器入口 MUST 是块边界，与范围里有没有能抛的指令无关。`throw_sites` MUST 仍只记真实抛点。一个 `catch` 子句要么写出来，要么带着原因引用，MUST NOT 无声消失。

#### Scenario: A try whose body cannot throw is still a try

- **WHEN** 两个顺序的 `try`/`catch`，两个保护范围里都没有能抛的指令
- **THEN** 两个 `try`/`catch` 都呈现，方法不是整段引用（P03/P07）

#### Scenario: A catch that contains its own try keeps both

- **WHEN** 一个 `catch` 子句的体内还有自己的 `try`/`catch`
- **THEN** 外层 `catch` 呈现，内层 `try` 在其中，没有子句消失（P03）

### Requirement: An existing local can be the resource

资源槽由一条局部读取的存储填入，正常路径对该槽做 `ifnull` 后 `close`，且 `ifnull` 的目标就是 `close` 的下一条时，文本 MUST 是 `try (T local = 原来的局部)`。体 MUST NOT 再存储原来的槽，否则 MUST 拒绝。MUST NOT 展开成 `catch (Throwable)` 或 `addSuppressed`。`close` 之后仍是 `goto` 的资源头 MUST 保持今天的写法。

#### Scenario: A copied local is closed without a goto

- **WHEN** `try (r)` 的字节码把 `r` 存进另一个槽，`close` 之后直接 `return`
- **THEN** 文本含 `try (java.io.Reader local1 = arg0)`，`return` 在该语句之后，且不含 `addSuppressed`（P03）

### Requirement: A synchronized return stays inside the statement

正常路径的 `monitorexit` 下一条是 `return`，且返回的值是退出前留在栈上的那个值时，文本 MUST 是 `synchronized (lock) { return value; }`。MUST NOT 在块外再读一次同一字段，也 MUST NOT 发明局部变量。退出后是 `goto` 的已有形状 MUST 保持今天的写法。退出后既不是 `return` 也不是 `goto` 时 MUST 保持拒绝。该处理器的 `catch_type == 0` MUST NOT 写成 `finally`。

#### Scenario: A return immediately after the normal exit is inside synchronized

- **WHEN** `monitorexit` 的下一条是 `ireturn`，其操作数是退出前的 `getfield`，处理器退出后 `athrow`
- **THEN** 文本含 `synchronized (this)`，`return this.n` 在该块内，且没有第二次 `this.n`（P03）

目标就是某循环出口块的普通边 MUST 写成 `break`。目标就是该循环头部或已证明的更新闩锁的普通边 MUST 写成 `continue`。目标是包围它的外层循环的出口时，MUST 写成指向该层的 `break`；目标是外层循环的头部或已证明的更新闩锁时，MUST 写成指向该层的 `continue`。这两种外层边 MUST 带能区分层次的标记。无标记的 `break` 或 `continue` 只作用于最内层，不得用来表示外层边。目标既不是本层出口、也不是任何包围循环的出口、头部或已证明更新闩锁时，MUST 保持缺口，且 MUST NOT 因此把整个循环收成引用。循环体里一个结构的后继仍在该循环内时，MUST 继续呈现后继，MUST NOT 把循环收成 `LoopShape`。`switch` 的汇合是该 `switch` 的 `break`。从 `switch` 的臂跳到循环出口时，MUST 写成该循环的带标记 `break`，因为无标记 `break` 会被 `switch` 截住。跳到循环头的 `continue` MUST 保持无标记。回边 MUST NOT 写成 `return`。

当 `switch` 的部分臂离开包围它的循环、其它臂仍留在循环内时，MUST 分别证明外层循环出口和后者的局部 `switch` 汇合；全方法后支配点若是循环出口，MUST NOT 直接当成 `switch` 的正常完成汇合。只有仍能正常完成的 Java case 臂才可由发射器补写 `switch` 的 `break`；已经以循环 `break`、`continue`、`return` 或 `throw` 终止的臂，以及两条 `if` 路径均终止的臂，MUST NOT 再追加不可达的 `break`。无法证明出口归属或臂完成性时，MUST 保留局部缺口。

头测循环同时具有循环前恰好一条无副作用的局部初值、只依赖该局部与循环不变量的头部测试、以及闩锁上恰好一条对该局部的更新，且将初值和更新移入头部不改变所有入边、退出后的局部使用与词法作用域时，MUST 写成 `for`。这些条件缺任何一条 MUST 保持 `while` 或 `do-while`，MUST NOT 猜成 `for`。

更新既可是一条 `iinc`，也可是一条完整、无额外效果的「读取归纳局部与循环不变量、加法、写回同一局部」SSA 链。后者的更新入口若是内层 `continue` 的目标，投影 MUST 使该边执行更新一次，同时跳过更新入口前的体尾效果。链不完整、操作数另有用途、步长在循环内被修改、更新块夹带其它效果、入边或作用域证明不全时，MUST NOT 将这次写回搬入 `for` 头部。

#### Scenario: A break targets the loop exit

- **WHEN** 循环体里一条普通边的目标就是该循环的出口块，且体的其余部分可呈现
- **THEN** 文本在该位置含 `break`，循环体的其它语句仍在，该边不再把整段循环变成 `LoopLeavesEarly` 引用（P04）

#### Scenario: A break targets an enclosing loop exit

- **WHEN** 内层循环体里一条普通边的目标是外层循环的出口块，而不是内层自己的出口
- **THEN** 文本含指向外层的 `break`，两层循环的其余语句仍在；无标记的 `break` 不得出现在这条边上（P04）

#### Scenario: Switch cases exit their enclosing loop or complete locally

- **WHEN** `SwitchLoopExits.run(IIZ)I` 的 case 0 和 default 有指向 BCI 64 外层循环出口的边，其余正常路径在 BCI 58 汇合后更新并回到循环测试
- **THEN** case 0 的条件路径和 default 写成带循环标签的 `break`，正常 case 在 BCI 58 之后继续循环；完整类用 Java 8 重编成功、与冻结原 class 的七行 `-Xverify:all` 输出相同，不含 `@bytecode` 或不可达的第二条 `break`（P04）

#### Scenario: A catch rejoins inside its enclosing loop

- **WHEN** `LoopTryHandlerEntry.loopTry(II)I` 的 `RuntimeException` 处理器由循环内 `[6,14)` 的受保护范围进入，处理器与正常路径在 BCI 20 汇合，之后递减并回到 BCI 2
- **THEN** 处理器的异常根入口不被误判为普通循环第二入口；文本完整保留 `while`、`try/catch`、汇合后的递减与返回，Java 8 重编后正常/抛出两行 `-Xverify:all` 与冻结 class 相同（P04）

#### Scenario: A continue targets an enclosing loop header

- **WHEN** 内层循环体里一条普通边的目标是外层循环的头部，而不是内层自己的头部或闩锁
- **THEN** 文本含指向外层的 `continue`，两层循环的其余语句仍在；无标记的 `continue` 不得出现在这条边上（P04）

#### Scenario: A counted loop is a for

- **WHEN** 头测循环满足初值、测试与单点更新三条
- **THEN** 文本含 `for`，初始化、条件与更新分别对应这三条，而不是一条 `while`（P04）

#### Scenario: An add/store latch is a for update

- **WHEN** `ForAddStoreSimple.run(II)I` 的头测循环由 BCI 13–16 的 `iload`、`iload`、`iadd`、`istore` 更新归纳局部，步长槽在循环内只读，且 SSA、闩锁入边与退出后作用域满足上述证明
- **THEN** 文本把 `local3 = local3 + arg1` 写在 `for` 的更新位置而不留在体内；完整类用 Java 8 重编，原 class、JADX 和 Jarde 在五组输入下 `-Xverify:all` 输出相同（P04）

#### Scenario: A labeled continue executes an add/store update

- **WHEN** `ForAddStoreLatch.run(III)I` 的内层边从 BCI 33 跳到外层更新入口 BCI 45，跳过 BCI 42 的 `afterInnerCount++`，而正常体尾也到达 BCI 45
- **THEN** 若外层循环完整呈现，文本必须让该边执行 BCI 45–48 的更新恰好一次、跳过 BCI 42 的语句；不得用跳过更新的 `while`/`continue` 冒充恢复；完整恢复时原 class、JADX 和 Jarde 的五组 `-Xverify:all` 输出相同（P04）

#### Scenario: A changed or impure step stays outside a for header

- **WHEN** 同形闩锁的步长槽在循环内被修改，或 `Add`/`Store` 链有额外使用者或效果，或更新块的其它效果会因 `continue` 搬动
- **THEN** 文本 MUST NOT 将该更新写入 `for` 头部；仍能完整呈现时保持 `while`，否则指明拒绝的 BCI（P04）

#### Scenario: An unproved header stays a while

- **WHEN** 头测循环的测试或闩锁不满足上述三条
- **THEN** 文本 MUST NOT 含把该循环写成的 `for`；能呈现时保持 `while` 或 `do-while`（P04）

#### Scenario: One store in the test is an assignment expression

- **WHEN** 循环测试里恰好一次 `Store`，存入的值与分支读到的值是同一次 `dup` 的两边，且除此之外只有条件所读的值
- **THEN** 条件含 `(local = expr)`，循环体的其余语句仍在，且文本 MUST NOT 改写成 `while (true)`（P04）

#### Scenario: A try is not the end of the loop body

- **WHEN** 循环体里的 `try` 汇合之后还有语句，并且回边回到循环头
- **THEN** 文本含 `while`、`try` 与 `catch`，汇合之后的赋值仍在循环内，且 MUST NOT 是 `LoopShape` 引用（P04）

#### Scenario: A switch is not the end of the loop body

- **WHEN** 循环体里的 `switch` 汇合之后还有语句，并且回边回到循环头
- **THEN** 文本含 `while` 与 `switch`，汇合之后的赋值仍在循环内，且 MUST NOT 是 `LoopShape` 引用（P04）

#### Scenario: A loop exit nested in a switch is labeled

- **WHEN** `switch` 的一条臂跳到它所在循环的出口，而不是 `switch` 自己的汇合
- **THEN** 该臂含指向这个循环的带标记 `break`，MUST NOT 是无标记 `break`，也 MUST NOT 把回边写成 `return`（P04）

### Requirement: A proved accessor call site spells the field

公开恢复入口在检查 accessor 候选时，MUST 按既有场景按需读取 callee 的声明与 Body。`accessor@1` 证明该调用是纯字段转发时，调用点 MUST 写成对应的字段读取或字段写入，MUST NOT 写成 `access$N(...)`。证明不了时 MUST 保留原调用与拒绝原因，MUST NOT 发明字段名。accessor 方法本身 MUST 仍出现在类源码视图里。

#### Scenario: A pure forward becomes a field access

- **WHEN** 调用点所在方法含语句，且 callee 被证明只转发到某一个字段
- **THEN** 调用点文本是该字段的读或写，不是 `access$` 调用（P05）

#### Scenario: An unproved accessor stays a call

- **WHEN** callee 的 Body 未能按需读到，或体里除字段转发外还有别的效果
- **THEN** 调用点保持 `access$N(...)`，并带有拒绝原因；类文本仍含该 accessor 方法（P05）

### Requirement: A proved lambda or anonymous body is shown at its use site

类源码视图装配一个类时，MUST 使用各方法已经产出的语句，MUST NOT 再跑一套方法恢复。lambda 合成方法的 `content` 为 `contains_statements` 且正文 `quality=structured`、无 fallback/引用缺口、完整 AST 可取得，只是内联候选；同次 bootstrap/调用点证据还 MUST 证明捕获参数与 SAM 参数按实现描述符逐槽对应、必要的类型适配与方法目标不变、捕获值在创建时的求值不被推迟或复制。全证时使用点 MUST 在 lambda 箭头内呈现该体，MUST NOT 写成对该 `lambda$` 方法的转发调用；多条语句只能在箭头块体中出现，MUST NOT 把体里的语句搬到使用点之前。任一证明缺失时使用点 MUST 保持转发调用。合成方法的物理身份、正文及可追溯报告 MUST 仍保留；只有同一物理类中所有有 `Code` 的方法完整扫描、指向该合成方法的每个 lambda 使用点均已原子改写，才带 `// jarde:` 标记说明体已在使用点呈现；有未扫描或未改写的使用点时 MUST NOT 加标记。候选不能从按需 `RuleDetails` 或已发射文本逆推。

类源码保留原 `lambda$...` 方法声明且另发出箭头时，`javac --release 8` 可能为箭头生成同名 helper，致使完整类编译失败。对同类 synthetic `lambda$` 实现方法，只要类源码发出指向它的箭头，类级呈现 MUST 为原物理方法确定性分配无冲突的源码别名，并将已证的同类直接调用按物理成员身份一致改名；无须预测编译器给箭头分配的序号。源映射和报告仍指向原方法。无法证明全部相关调用可同步改名时 MUST 明示编译限制，MUST NOT 声称完整类可重编。方法不得只因 synthetic 标志而隐藏。

匿名类的 `InnerClasses.inner_name` 为空、`EnclosingMethod` 指向当前方法，且调用者类所有有 `Code` 的方法中只有一处分配点调用该候选类的构造器，只是候选；所选物理输入范围内候选站点以外的构造或类身份使用（包括调用者自身与其它类）也 MUST 被完整排除。范围扫描不完整、可能指向候选却无法解析、兄弟 class 缺失或身份/分配点不唯一时不得按当前类的文本猜。每个待内联方法 MUST 是已产出 Java 语句、`quality=structured`、无 fallback/引用缺口的完整正文；`contains_statements` 单独不足以证明完整。唯一构造器的每个参数 MUST 划入两类之一：按原描述符和顺序传给直接基类的唯一 `invokespecial <init>`，或在创建时恰好写入一个合成捕获字段；除此之外的构造器指令、字段写入或实例初始化效果若不能按原时序表达，MUST 拒绝内联。每个捕获字段的全部读取 MUST 映射回该参数和分配点词法范围内同一稳定值，不能只凭字段名或字段类型推断；创建时求值、异常顺序不得改变。证明齐全时方法体 SHALL 出现在匿名类花括号中。`new Base(args) { ... }` 的实参 SHALL 仅为真实基类构造实参；实现接口时 `new Interface() { ... }` MUST NOT 携带合成捕获实参。非静态成员基类的构造器描述符包含限定外层实例；若不能另外证明 `outer.new Base(args) { ... }` 的限定接收者及其空值检查，MUST 拒绝内联，MUST NOT 把该实例印成普通 `Base` 实参。缺少任何证明或正文时 MUST NOT 内联：使用点保持该类的二进制名，该类单独装配。`Outer.this` 仅在分配点值流证明该值就是当前词法外层接收者、且正文读该捕获时可用，MUST NOT 凭字段名或同型对象发明。

合成捕获字段若在字节码里先于基类构造调用写入，MUST 保留该可观察时序，MUST NOT 为使独立二进制名类的源码通过 Java 8 编译而无条件把字段写入搬到 `super(...)` 后。基类构造期间可能虚调用匿名覆写并读该捕获；只有类级匿名语法及完整值流证明能让 Java 编译器重建等价前置合成写入时，才 SHALL 将使用点投影为 `new Base(...) { ... }`。不能证明时保留分开的类与使用点，并声明独立构造器文本的编译限制（P02/P06）。

唯一性计数 MUST 包含同次解码中所有指向该匿名类的 `Allocate`，包括 `new@1` 未验证的候选和被其它规则预留的分配；调用者任一有 `Code` 的方法未完整扫描时 MUST 拒绝唯一性，MUST NOT 把空的已验证 site 列表解释为零分配。所选物理输入范围内调用者和其它类的 Code/metadata XRef 覆盖 MUST 完整，并按同一解析环境确认目标物理身份；出现另一处分配、构造器句柄或可观察类身份引用时 MUST 拒绝内联，覆盖不完整或目标可能相同而未决时也 MUST 拒绝。此证书只说明所选输入范围，不宣称排除范围外的开放世界调用者。

这些类级事实 MUST 来自 reader 已经解析的 `InnerClasses` 与 `EnclosingMethod`，MUST NOT 由单方法 IR 重建，也 MUST NOT 重新扫描属性字节。

#### Scenario: A recovered lambda body replaces the forward call

- **WHEN** 使用点的 lambda 合成方法正文完整且 `quality=structured`，捕获/SAM 参数逐槽、目标及适配均已证明
- **THEN** 使用点文本含该体，不含对该 `lambda$` 方法的调用；该合成方法仍在类文本中（P06）

#### Scenario: A retained helper name collides with javac's lambda symbol

- **WHEN** 同一类既保留 `lambda$build$0(int,int)` 的源码声明又在 `build` 中呈现一个 lambda 箭头，Java 8 编译器生成的符号与该声明冲突
- **THEN** 类级源码为原物理 helper 及其全部已证直接引用使用同一无冲突别名，原方法的正文与物理身份仍可追溯；整类 Java 8 重编不得因同名 helper 失败（P06）

#### Scenario: An unrecovered lambda stays a forward call

- **WHEN** 合成方法的 content 不是 `contains_statements`、正文虽含语句却有 fallback，或捕获/适配对不上
- **THEN** 使用点保持转发调用，且类文本里没有「体已在使用点呈现」的标记（P06）

#### Scenario: Lambda body effects stay deferred until the SAM is invoked

- **WHEN** lambda 的捕获值在创建时求得，而合成方法体含两条以上有序语句与一次可能抛异常的调用
- **THEN** 创建 lambda 时只发生原捕获求值，箭头块体内的效果只在 SAM 调用时按原顺序执行；不能把体提前到创建点，也不能重复捕获值的求值（P06）

#### Scenario: One unprojected use prevents the all-sites marker

- **WHEN** 同一合成方法的两个 lambda 使用点中一个被证明并改写，另一个因适配或正文缺口未改写
- **THEN** 合成方法仍保留，类文本 MUST NOT 标记其正文已在所有使用点呈现；未改写处保留转发调用（P06）

#### Scenario: An anonymous class is inlined only when every method was recovered

- **WHEN** 匿名类满足无名、唯一 `EnclosingMethod`、唯一 `new`，但其中一个方法是 `explanation_only`
- **THEN** 使用点 MUST NOT 含该类的方法体花括号，该类 MUST 仍单独装配（P06）

#### Scenario: Two allocation BCIs in one caller method target the same anonymous class

- **WHEN** 同一物理调用者方法的两处 `new` BCI 都指向同一个匿名类，且两个构造调用也指向同一物理构造器
- **THEN** 唯一分配点证明 MUST 拒绝，两个使用点 MUST 保持同一物理目标身份；不得按“只在一个方法里使用”将它们分别投影成两个源码匿名类（P06）

#### Scenario: Another class in the selected input uses the candidate identity

- **WHEN** 调用者类内部只扫到一处分配，但同一所选物理输入范围内另一个类也构造该物理匿名类、持有其构造器句柄或直接使用其类身份；或者该范围的 XRef 扫描未完成
- **THEN** 唯一性证明 MUST 拒绝，不能把调用者的一处分配内联为新的源码匿名类并改变它与另一个使用者之间的运行时类身份（P06）

#### Scenario: A mixed-quality anonymous body is not complete

- **WHEN** 候选匿名类的某个方法有 Java 语句但 `quality=fallback`，正文还引用了未恢复的 BCI
- **THEN** 即使该方法 `content=contains_statements`，使用点 MUST NOT 内联该类（P06）

#### Scenario: Synthetic constructor captures are not base constructor arguments

- **WHEN** 唯一匿名类的构造器从 `new` 读取外层接收者与局部捕获，类实现一个接口，已恢复方法读取这些合成字段
- **THEN** 只有捕获字段能按真实值流映射到稳定词法值且原创建顺序保留时才 SHALL 写 `new Interface() { ... }`；MUST NOT 写带捕获实参的 `new Interface(this, value) { ... }`，不能把匿名体内的 `this` 冒充外层接收者；证据不足时 SHALL 保留未内联类及其使用点（P06）

#### Scenario: Base constructor arguments and captures have different roles

- **WHEN** 一个匿名类继承基类，分配点分别求值 `choose()` 作为基类构造实参和局部 `captured` 作为合成捕获
- **THEN** 仅在构造器的 `super` 调用、捕获字段存储及全部读取逐一证明后 SHALL 写 `new Base(choose()) { ... captured ... }`；`choose()` 恰执行一次，捕获值 MUST NOT 被传给 `Base` 或重新求值（P06）

#### Scenario: A captured value is visible to virtual dispatch during base construction

- **WHEN** 匿名构造器在 `Base.<init>` 前写入捕获字段，而 `Base()` 在尚未返回时虚调用读取该字段的匿名覆写
- **THEN** 内联后的源码若生成，重编执行 MUST 在虚调用期间读到原捕获值；若无法证明等价内联，独立构造器 MUST 保留原指令顺序及编译限制，MUST NOT 改成 `super(); this.val$captured = arg1;`（P02/P06）

#### Scenario: An equal-typed object is not the lexical outer receiver

- **WHEN** 分配点捕获一个与当前类同型的参数 `other`，匿名方法读取的是 `other.state`
- **THEN** 仅在真实值流可保留时 SHALL 内联并保持 `other` 语义；MUST NOT 因类型相等而改写为 `Outer.this.state`（P06）

### Requirement: Class source spells package, throws, and varargs from the class file

类源码视图的类声明 MUST 使用 `package` 与简单名：二进制名在最后一个 `/` 之前的部分写成包名（`/` 改为 `.`），最后一段写成简单名；简单名中的 `$` MUST 保留。二进制名不含 `/` 时 MUST NOT 写 `package`。MUST NOT 再把完整二进制名写在 `class` 或 `interface` 关键字后面。

方法的 `throws` MUST 等于该方法 `Exceptions` 属性中的类名，顺序与属性一致，内部名写成点分名。属性不存在时 MUST NOT 写 `throws`，也 MUST NOT 从 `athrow` 或处理器类型推断。

方法 access flags 的 `0x0080` MUST 按 `ACC_VARARGS` 解读。该标志置位且描述符的最后一个参数是数组时，声明的最后一个参数 MUST 写成 `T...`，MUST NOT 写成 `T[]`。标志置位但最后一个参数不是数组时，MUST NOT 写 `...`，MUST 按描述符声明参数，并加一条 `// jarde:` 标记。字段的 `ACC_TRANSIENT` 拼写 MUST 不变。

这些事实 MUST 来自 reader 已经解析的方法属性与 access flags，MUST NOT 重新解码属性。

#### Scenario: A packaged class uses a simple name

- **WHEN** 类的内部名是 `a/b/Outer$Inner`
- **THEN** 类源码文本含 `package a.b;`，类声明的名字是 `Outer$Inner`，不是 `a.b.Outer$Inner`（P07）

#### Scenario: Throws comes only from the Exceptions attribute

- **WHEN** 方法的 `Exceptions` 属性列出 `java/io/IOException`，且方法体里还有别的 `athrow`
- **THEN** 声明含 `throws java.io.IOException`，且不含从 `athrow` 额外推断出的类型（P07）

#### Scenario: Varargs is the last array parameter

- **WHEN** 方法置位 `ACC_VARARGS`，描述符末参是 `[[B`
- **THEN** 声明的最后一个参数写成 `byte[]...`，不是 `byte[][]`（P07）

### Requirement: A proved Unicode scalar is written as the character

方法与类源码文本里的字符串字面量 MUST 把已解码的 Unicode 标量按字符写入，当且仅当该标量不是 `"`、`\`、LF、CR、U+2028、U+2029，也不是 U+0000–U+001F 或 U+007F。这些例外 MUST 继续用既有转义。不能组成标量的孤立代理项 MUST 继续写成 `\uXXXX`。转义与否 MUST NOT 改变 `content` 分类，也 MUST NOT 改变该字面量对应的 BCI。

#### Scenario: A non-ASCII letter is not a unicode escape

- **WHEN** 字节码的字符串常量解码为「正在」
- **THEN** 文本含 `正在`，不含 `\u6b63\u5728`（P08）

#### Scenario: A control character stays escaped

- **WHEN** 字符串常量含 U+0000 或换行
- **THEN** 文本仍使用 `\u0000` 或 `\n` 一类既有转义，字面量不被换行拆开；一个 U+0000 MUST NOT 变成一个或多个 U+FFFD（P08）

### Requirement: A post-increment and a switch fall-through keep the value and the block

`iload` 的下一条是对同一槽位的 `iinc`，且随后的使用读的是这次 `iload` 压下的值时，文本 MUST 返回改写前的值。`iinc` 为 `+1` 或 `-1` 且中间没有别的指令时，MUST 写成 `local++` 或 `local--`。其它改槽位后再使用的情形 MUST 写成改写前绑定的值，MUST NOT 套用 `++`。文本 MUST NOT 在 `iinc` 之后返回该槽位的新值，也 MUST NOT 丢掉这次 `iinc`。

`switch` 的前一个 case 的代码只走到后一个 case 的入口时，被走到的块 MUST 只出现在后一个臂，前一个臂 MUST NOT 以 `break` 结束。两个臂真正共用一块、且不能按 case 入口切开时，MUST 保持 `SwitchArmsOverlap`，MUST NOT 把同一块写进两个臂。选择表达式的类型已是 `char` 时，键 MUST 写成字符字面量，转义与字符串相同。选择表达式是 `int` 时键 MUST 保持数字。

#### Scenario: A returned post-increment yields the old value

- **WHEN** 字节码是 `iload`、紧接着 `iinc 1`、然后 `ireturn` 读这次 `iload` 的值
- **THEN** 文本含 `++` 或一个在加一之前绑定的旧值，并且返回的是旧值（P11）

#### Scenario: A field pre-increment yields the new value

- **WHEN** 字节码是 `aload`、`dup`、`getfield`、`iconst_1`、`iadd`、`dup_x1`、`putfield`，且返回的是这次加法的值
- **THEN** 文本含 `return ++this.n`，MUST NOT 再读一次该字段，也 MUST NOT 发明局部变量（P11）

#### Scenario: A char switch spells its keys as characters

- **WHEN** `switch` 的选择表达式类型是 `char`，键是该字符的码位
- **THEN** 文本含 `case 'a'` 这种字符字面量，而不是该码位的十进制数字；`switch (int)` 的 `case 97` MUST 保持数字（P11）

#### Scenario: A case that reaches the next case does not break

- **WHEN** `case 1` 的目标块顺序落到 `case 2` 的目标块
- **THEN** 文本含两个 `case`，`case 1` 与 `case 2` 之间没有 `break`，`case 2` 的语句只出现一次（P11）

### Requirement: A field write of a constructed value is one assignment

`putfield` 或 `putstatic` 读到的值是一条已证明构造链（`new` 加其 `<init>`）的 `dup` 遗留值时，MUST 写成一次赋值 `field = new T(args)`：实例字段带接收者，静态字段带属主类型，沿用既有字段写入的接收者写法。MUST NOT 发明局部保存这个实例，MUST NOT 把构造调用写成独立语句后丢失赋值。局部 `Store` 读到同一个 `new` 的既有写法保持不变。此要求约束单方法恢复正文；在尚无完整类级证明时，它 MUST NOT 自行把 `<clinit>` 里的枚举常量赋值合并回枚举声明。独立的类源码装配只有取得 `recover-proved-enum-constants` 的完整类级证明，才可在保留该方法原始报告的同时投影出枚举常量；证明不完整时仍按本单方法规则保留逐项赋值。

#### Scenario: A field assigned a new instance in a method

- **WHEN** 方法体是 `new; dup; invokespecial; putfield`，字段是 `this.a`
- **THEN** 文本含 `this.a = new java.lang.Object()`，MUST NOT 含 `Duplicate` 或该链的引用（P11）

#### Scenario: A static field assigned a new instance

- **WHEN** 静态方法体是 `new; dup; invokespecial; putstatic`
- **THEN** 文本含 `More34.s = new java.lang.Object()`，带属主类型（P11）

#### Scenario: An enum constant is assigned in the static initializer

- **WHEN** `<clinit>` 是三个 `new; dup; ldc; iconst; invokespecial; putstatic` 再 `Color.$VALUES = $values()`，且本次只做单方法恢复、没有完整类级证明
- **THEN** 该单方法的恢复正文依次含 `Color.RED = new Color("RED", 0)` 一类的赋值与 `Color.$VALUES = $values()`，`<clinit>` 不再含 `Duplicate` 引用；该单方法 MUST NOT 自行合并成 `RED, GREEN, BLUE;`（P11）。完整类级证明允许类源码装配另行投影常量列表，未知或不完整证明仍保留逐项赋值。

### Requirement: A static call names the class the instruction names

`invokestatic` 的常量池属主不是当前类时，调用文本 MUST 写成 `Owner.name(args)`。属主就是当前类时，MUST 保持无限定名。装箱与拆箱 MUST 保持 `valueOf` 与 `intValue` 这两次真实调用，MUST NOT 收成没有调用的 `return`。

#### Scenario: Integer.valueOf is not a bare name

#### Scenario: One dup stored into two locals is written once

- **WHEN** 一条 `dup` 的下一个指令和下一条指令都是局部 `Store`，中间没有别的指令
- **THEN** 先存储的局部写下被复制的表达式，后存储的局部读取这个局部；文本 MUST NOT 把该表达式写两遍，也 MUST NOT 让后面的读取使用没有声明的局部（P11）

#### Scenario: A duplicated receiver is one field assignment

- **WHEN** `putfield` 与它前面的 `getfield` 读的是同一次 `dup` 复制的同一个接收者，且写的是同一个字段
- **THEN** 文本含 `this.n = this.n +` 这种一次赋值，MUST NOT 含 `+=`，也 MUST NOT 把这次更新写成 `++`（P11）

#### Scenario: An array element post-increment yields the old element

- **WHEN** 字节码是 `dup2`、`iaload`、`dup_x2`、`iconst_1`、`iadd`、`iastore`，且随后使用的是这次 `iaload` 的旧值
- **THEN** 文本含 `arg0[arg1]++`，且 MUST NOT 另写 `arg0[arg1] =`，也 MUST NOT 发明一个保存旧值的局部（P11）

#### Scenario: A comparison materialized as 0 and 1 is the comparison

- **WHEN** 一条比较的两臂分别只是 `iconst_1` 和 `iconst_0`，汇合点只返回这个值，或只把它存进一个局部或一个字段
- **THEN** 文本含这个比较，返回时是 `return arg0 == arg1` 或极性相反的 `!=`；存字段时字段赋值仍在。MUST NOT 含空的 `if`，MUST NOT 含 `?`，也 MUST NOT 写成 `assert`（P11）

#### Scenario: append(char) is one character of a proved concatenation

- **WHEN** 一条 `StringBuilder` 或 `StringBuffer` 链的某个 `append` 的参数类型是 `char`
- **THEN** 该链收成 `+`。第一个操作数还不是字符串时，文本以 `"" +` 开始，所以单独的 `append(char)` 是 `"" + arg0`，而不是裸的 `arg0`。前面已有字符串时是 `"x" + arg0`。MUST NOT 把 `char` 收成整数加法。`append(char[])` 与 `append(CharSequence)` MUST 保持构造器调用（P11）

#### Scenario: Integer.valueOf is not a bare name

- **WHEN** 指令是 `invokestatic java/lang/Integer.valueOf:(I)Ljava/lang/Integer;`，当前类不是 `java.lang.Integer`
- **THEN** 文本含 `java.lang.Integer.valueOf(arg0)`，且 MUST NOT 是单独的 `valueOf(`

### Requirement: A string-concat invokedynamic is the same concatenation

引导方法是 `java/lang/invoke/StringConcatFactory.makeConcatWithConstants`，配方只含字面量与个数等于栈操作数的 U+0001 时，文本 MUST 是已有的 `+`。U+0001 MUST 按配方从左到右对应这些操作数，其余字符 MUST 是字符串字面量。第一个部分不是字符串时 MUST 仍写成 `"" +`。配方含 U+0002、引导参数超出这一条配方、或占位个数与操作数不一致时，MUST 保持今天对这条 `invokedynamic` 的拒绝，MUST NOT 写成 lambda。`makeConcat` 且没有引导参数时，操作数 MUST 按顺序连成同一条 `+`。`--release 8` 的 `StringBuilder`/`StringBuffer` 链 MUST 保持今天的文本。

#### Scenario: A recipe with a literal between two arguments is one concatenation

- **WHEN** 配方是 U+0001、`/`、U+0001，操作数是一个 `String` 和一个 `int`
- **THEN** 文本含 `return arg0 + "/" + arg1`，且不含 `StringConcatFactory`（P11）

#### Scenario: A single argument recipe keeps the string conversion

- **WHEN** 配方只有一个 U+0001，操作数是 `int`，返回类型是 `String`
- **THEN** 文本含 `return "" + arg0`，MUST NOT 是 `return arg0`（P11）

### Requirement: A special call, a compare, and a constant field keep the program they came from

`invokespecial` 的方法名不是 `<init>`、池项引用的是当前类声明的直接父类、且接收者已证明为入口实例 `this` 时，调用 MUST 写成 `super.name(...)`。池项引用的是直接父接口且接收者同样已证明时，MUST 写成 `接口类型.super.name(...)`。当前类已声明为 private 的目标 MUST 保留实际接收者，包括其它同类实例。其余证据不足的特殊调用 MUST 引用，MUST NOT 仅凭属主不同写成 super 或默认写成虚调用；独立验收由 `preserve-special-call-dispatch` 承担。

`lcmp`、`fcmpg`、`fcmpl`、`dcmpg`、`dcmpl` 的结果唯一供给同块紧邻零分支、且外围结构与操作数可恢复时，文本 SHALL 组合成保持 NaN 偏置及分支方向的条件。无序时为真的关系 SHALL 使用正确的取反关系，MUST NOT 直接换成 NaN 上为假的相反关系。其它消费者 MUST 保留有来源的缺口；独立验收由 `recover-numeric-comparison-conditions` 承担。

字段的 `ConstantValue` 属性 MUST 写成该字段的初值。没有该属性时 MUST NOT 从 `<clinit>` 或从使用点的字面量反推初值。使用点的字节码若是字面量而不是读取该字段，MUST 保持字面量。

同名重载的调用参数呈现 SHALL 保留池目标的参数类型选择；没有足够安全转换证据时 MUST 拒绝，不得合成会新增异常的检查。该范围由 `preserve-invocation-argument-types` 独立验收。桥方法 MUST 保留，MUST NOT 写成对所在方法自身的调用，也 MUST NOT 添加 `@Override` 或删去该成员；其声明和返回类型区分仍是独立范围。

#### Scenario: An invokespecial of the superclass is super

- **WHEN** 实例方法用 `invokespecial` 调用父类的同名实例方法，接收者是 `this`
- **THEN** 文本含 `super.该方法`，MUST NOT 含把这次调用写成的 `this.该方法`（P02 之外的调用拼写）

#### Scenario: A long comparison becomes the relational operator

- **WHEN** `lcmp` 的唯一消费者是一条单操作数条件分支，且该分支就是源码的 `<`
- **THEN** 文本含这个关系运算，不再把 `lcmp` 引用成 `Other`

#### Scenario: A NaN-true float comparison keeps a negated relation

- **WHEN** `fcmpg` 后面的分支在操作数为 NaN 时成立，而对应的 Java `>` 或 `>=` 在 NaN 时不成立
- **THEN** 在唯一、同块、紧邻消费条件下，文本 SHALL 使用 `!(a <= b)` 或 `!(a < b)`，MUST NOT 把该比较写成 `>` 或 `>=`

#### Scenario: A constant field is not inferred at a use

- **WHEN** 字段带 `ConstantValue`，而某个方法体里是与该值相同的字面量指令、不是对该字段的读取
- **THEN** 字段声明含该初值，方法体仍是字面量，不是字段名

### Requirement: Constructed sources are accepted by a three-way comparison

P01–P08 的每个场景 MUST 有一份由本 change 编写的 Java 源码，MUST NOT 从既有应用语料裁剪。该源码 MUST 用记录了版本与 `--release` 的 `javac` 编译。同一组 class 文件 MUST 分别由 jadx 1.5.6 与 jarde 类源码视图反编译。验收记录 MUST 对每个场景同时留下源码摘录、jadx 摘录与 jarde 摘录。

通过条件 MUST 以本规格的结构证明和原 class 的执行行为、异常顺序、classfile 属性为准，而不是以 jadx 文本为准。手写源码证明该结构可由 Java 表达，不要求从可能对应多种源码的字节码逐字还原；jarde 写出的结构 MUST 满足相应证明，能完整重编的场景其运行结果 MUST 与原 class 一致。要求不写的结构，即使出现在 jadx 文本里，jarde MUST NOT 照抄。与 jadx 逐字相同 MUST NOT 作为通过条件。规格明确允许缺口的场景，jarde MUST 给出指名 BCI 的引用，而不是空结构。既有应用语料上的计数 MUST NOT 代替这组场景。

#### Scenario: A structure the source has must appear in jarde

- **WHEN** 源码含一条本 change 要求呈现的 `if`、`catch`、`for`、`break`、字段访问、lambda 体或声明，且对应证明条件成立
- **THEN** jarde 文本含该结构；若 jadx 文本与源码不同，记录以本规格的证明条件及原 class 行为判定 jarde，不要求 jarde 跟随 jadx（P09）

#### Scenario: A structure the spec forbids is not copied from jadx

- **WHEN** jadx 把失败的 try-with-resources 写成用户 `catch`、把 `catch_type` 为 0 的记录写成 `finally`、把未证明的 `switch` 写成空 `switch`，或引入 class 文件里不存在的类型
- **THEN** jarde MUST NOT 出现同一偏离；验收记录同时留下三份摘录，并标明 jadx 与源码的差异（P09）
