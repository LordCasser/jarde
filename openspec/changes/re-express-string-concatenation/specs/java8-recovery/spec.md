## ADDED Requirements

### Requirement: A presented concatenation keeps each append's own conversion

一条被判为拼接链（`StringBuilder`/`StringBuffer` 链）的呈现 MUST 逐片段保留该 `append` overload 自己的转换语义：文本 MUST 计算与该链相同的值，且每个片段的转换 MUST 发生在该片段自己的位置。链的呈现 MUST 从一个**字符串上下文**开始：需要经 `String.valueOf` 转换的片段（int/long/float/double/boolean/对象）出现在第一个字符串操作数之前时，字符串拼接 MUST 从这些片段处开始（例如在它们之前插入一个空字符串片段，或用同等语义的 `String.valueOf` 形态），MUST NOT 先把相邻的数值片段加成数值、再对求和结果做一次转换——`append(a).append(b).append("!")` MUST NOT 呈现为 `arg0 + arg1 + "!"`（输入 `(1, 2)` 时原 class 返回 `"12!"`，该文本返回 `"3!"`）。

MUST 按 `append` 的参数 descriptor 拼写每个片段：参数类型为 `boolean` 的片段里的 `0`/`1` 字面量 MUST 呈现为 `true`/`false`，MUST NOT 呈现为 `1`/`0`（`append(true)` 的 `iconst_1` 在文本里不是数值 `1`）；`null` 片段 MUST 呈现为 `null` 字面量；对象片段 MUST 按其 overload 的转换呈现（对象经 `String.valueOf` 等价的转换）。呈现 MUST 保持每个片段的求值次数与左到右顺序（不重复调用、不移动可能抛异常的操作、不改变异常次序），每个片段 MUST 保留自己的 BCI/source-map 锚点，插入的空字符串片段 MUST 只携带由链派生的锚点而不是一条它没有执行的指令。

片段序列 MUST NOT 被任意平衡、合并或重排：把相邻片段合并、把片段移出它自己的求值位置、或为「少一层」改变某个片段被转换的时机都会改变值或副作用，MUST NOT 出现。片段自己的表达式 MUST 保持自己的分组，MUST NOT 被拆进链的加法：链 `append(int a).append(int b)`（**两个**片段）MUST 呈现为 `"" + arg0 + arg1`（两个转换各自发生，输入 `(1, 2)` 得 `"12!"`），而链 `append(int a + int b)`（**一个**片段，其表达式本身是加法）MUST 呈现为 `"" + (arg0 + arg1)` 的形态（该加法先按数值求出再转换，同一输入得 `"3!"`）；把后者的片段拆成两个片段、或把前者两个片段合并成一次数值加法，都改变值，MUST NOT 出现。链的识别与本规则接受的 overload 集合 MUST NOT 因此改变。某个片段的转换无法用呈现表达时，该区域 MUST 按既有 refusal 契约拒绝并保留 bytecode 与 origin。representation/quality/content/execution 与「两侧都能编译」MUST NOT 被当作本要求的证据：本要求的证据是原 class 与呈现文本在同一输入集合上的执行对照。

#### Scenario: A numeric fragment keeps its conversion where the chain converts it

- **WHEN** IR 证明确认的链是 `append(int a).append(int b).append(String "!")`（源码形状 `new StringBuilder().append(a).append(b).append("!").toString()`），且该链被呈现为表达式
- **THEN** 文本 MUST 在第一个数值片段处进入字符串拼接，MUST NOT 是 `arg0 + arg1 + "!"`；输入 `(1, 2)` 时原 class 返回 `"12!"`，呈现文本 MUST NOT 返回 `"3!"`（验收 A13）

#### Scenario: A boolean fragment is spelled by its append descriptor

- **WHEN** 链的某个 `append` 的参数类型是 `boolean`，且该片段的值是 `0`/`1` 字面量
- **THEN** 文本里该片段 MUST 是 `true`/`false`，MUST NOT 是 `1`/`0`；两侧以 `true` 与 `false` 两个输入执行时返回值 MUST 相同（`"" + 1` 的 `"1"` 与 `append(true)` 的 `"true"` 的差异 MUST 被该对照捕获）

#### Scenario: Null and object fragments keep their conversion

- **WHEN** 链里有 `append((Object) null)` 或 `append(someObject)` 形态的片段
- **THEN** `null` 片段 MUST 呈现为 `null`，对象片段 MUST 按 `String.valueOf` 等价的转换呈现；两侧执行（含 `null` 输入与一个非 `null` 对象）返回值 MUST 逐项相同（验收 A13）

#### Scenario: A part that is itself an addition keeps its own grouping

- **WHEN** 链的**一个**片段的表达式本身是一次数值加法（源码形状 `new StringBuilder().append(a + b).append("!")`，两个 int 参数），或**两个**片段各自是一个数值（`append(a).append(b).append("!")`）
- **THEN** 前者 MUST 呈现为 `"" + (arg0 + arg1)` 一类保持该加法分组的形态（输入 `(1, 2)` 得 `"3!"`），后者 MUST 呈现为 `"" + arg0 + arg1`（同一输入得 `"12!"`）；MUST NOT 把前者的片段拆成两个片段、也 MUST NOT 把后者的两个片段合并成一次数值加法，两种改写都会让执行对照在至少一个输入上以值不同变红（验收 A13）

#### Scenario: The order and count of evaluations are unchanged

- **WHEN** 链的某个片段是可能抛异常或可观察的求值（调用、字段读取），且它夹在需要转换的片段之间
- **THEN** 插入的字符串上下文 MUST NOT 重复、移动或删除该片段自己的求值；两侧的调用次数、顺序与异常 MUST 逐项相同（验收 A12）

#### Scenario: Chains already in a string context are unchanged

- **WHEN** 链的每个 `append` 参数都是 `java.lang.String`（全字符串），或首个片段已经是字符串、需要转换的片段在其后（源码形状 `append("x").append(a)`）
- **THEN** 文本 MUST 与修正前逐字相同，MUST NOT 增加空字符串片段或其它装饰

#### Scenario: An unstateable conversion is refused

- **WHEN** 某个片段的 `append` overload 的转换无法用呈现表达（既有识别规则不接受的 overload，或任何拼不出同一转换的形态）
- **THEN** 该区域 MUST 按既有 refusal 契约拒绝并保留 bytecode 与 origin（representation=Mixed、quality=Fallback、拒绝诊断点名相关 BCI 与物理方法）；本要求 MUST NOT 通过扩大接受范围、猜测类型或改写转换来换取反例通过

## MODIFIED Requirements

### Requirement: Recovery recursion is bounded and a stop is published

恢复调用 MUST 返回 `RecoveryReport`，MUST NOT 因递归耗尽进程栈而让进程以 signal 终止。恢复层 SHALL 只在能证明递归会完成时继续加深；每次递归下降前 MUST 检查一个显式的深度界，界内递归照常完成，超过界 MUST 在继续加深之前结束为已发布停止：`outcome = Stopped`、非 `Complete` 的 `execution`（Partial 或 Cancelled）、以及点名停止原因与位置的诊断，全部复用既有停止词汇与平面。停止 MUST 保留本次运行的真实 usage 与停止位置，MUST NOT 提交部分构建的文本/段表（`content = not_produced`，与既有停止语义一致），MUST NOT 以空产物冒充成功，也 MUST NOT 只记录日志后返回成功。结构性平面（representation/quality/content/execution）MUST NOT 被当作本要求的证据：本要求的证据是入口返回的报告与其中的停止平面。

本要求同样约束恢复层**自己构造的表示**：一条拼接链在输入里的 append 数 MUST NOT 变成这一层的递归深度——链的构造、发射、克隆与释放（以及任何复制该表示所在结构的路径）的递归深度 MUST NOT 随该链的 append 数增长，增加一个片段 MUST NOT 加深这些路径的嵌套。片段**内部**真正嵌套的子表达式仍按既有的值嵌套界受限，本要求不改变那条界（真正的递归下降前仍 MUST 检查它），也不得用「更大的界」或「放大线程栈」（更大的 `RUST_MIN_STACK`、大栈线程或等价手段）替代上述性质。可观察结果：链长达到复核记录的区间（debug 构建在 N = 1536/2048 以 signal 结束、N = 1024 完成）与更大规模时，公开入口与 CLI MUST 返回报告而不是 signal；链**能**呈现时 MUST 正常完成（MUST NOT 把「2048 个片段必须停止」写成契约），不能呈现时 MUST 结束为既有已发布停止；预算或取消在深链的构建/发射过程中生效时，MUST 发布既有停止形态并在释放该结构后返回，MUST NOT abort。debug 与 release 是同一份代码的两条边界证据，两个构建 MUST 都返回报告；MUST NOT 用 release 对更深输入完成代替 debug 的 signal 证据，也 MUST NOT 用任一构建的完成深度代替界的依据。

#### Scenario: A request that used to abort now answers

- **WHEN** 对固定复现（默认预算、公开入口、standalone class）发起恢复，且修正前同一输入使进程以 `stack overflow, aborting` 结束
- **THEN** 调用返回报告：`outcome` 为 Stopped、`execution` 非 Complete、诊断点名递归界与位置；CLI 以既有「执行未完整」状态（exit 4）退出且 stdout 为报告。MUST NOT 出现 signal 终止、空 stdout 或无报告的失败（验收 A13、A14）

#### Scenario: The bound is checked before the descent

- **WHEN** 下一次递归会超过显式界
- **THEN** 在该次递归开始之前结束并发布停止，位置与原因可定位；不能在递归内栈耗尽后再补救，也不能先写出部分产物再撤回

#### Scenario: Re-entering a state inside the bound

- **WHEN** 递归在界内重新进入本次运行已经访问过的状态（同一 SSA 值、同一块或同一表达式节点的重复进入），说明该递归不能自行完成
- **THEN** 结果 MUST 是已发布停止，或该层能把它局部化时按既有 refusal 语义处理：被拒区域保留 bytecode 与 origin、不得标记结构化，其余工作如实报告；两种结果都 MUST NOT 继续加深直到栈溢出，诊断 MUST 说明是重新进入而不是输入过深（验收 A13）

#### Scenario: In-bound recovery is unchanged

- **WHEN** 已提交语料与受控 fixture 的递归在界内完成
- **THEN** 结果与修正前逐字段一致（仅 elapsed_millis 可不同），不因新增界升级拒绝、降级 quality、改变 content 或改变停止/取消语义（验收 A13）

#### Scenario: The stop is the existing stop shape

- **WHEN** 一个运行因超界停止
- **THEN** 其 `execution`、诊断形态、`content = not_produced` 与 text/段表为空的性质与其他既有停止一致（仅 code 与位置不同）；不得新增 `StopReason` 变体、预算维度或报告平面

#### Scenario: A chain's length does not become this layer's depth

- **WHEN** 一条已验证的拼接链含数千个 `append` 片段
- **THEN** 构造、发射、克隆与释放的递归深度 MUST NOT 随 append 数增长（增加一个片段 MUST NOT 加深这些路径的嵌套）；片段内部真正嵌套的值仍按既有值嵌套界受限，本要求 MUST NOT 被表述为取消该界（验收 A13）

#### Scenario: A deep chain answers without a signal

- **WHEN** 链长落在复核记录的区间（N = 1536/2048；N = 1024 是修正前的完成对照）或更大，且分别在 debug 与 release 构建上恢复
- **THEN** 两个构建 MUST 都返回报告（能呈现时为产物，不能呈现时为既有已发布停止），MUST NOT 出现 signal、exit 134、空 stdout 或 `fatal runtime error: stack overflow, aborting`；MUST NOT 通过放大线程栈取得该结果（验收 A13、A14）

#### Scenario: A deep chain may complete normally

- **WHEN** 深链的每个片段都能按本要求呈现
- **THEN** 该运行 MUST 正常完成并给出链自己的文本与 content 分类，MUST NOT 被一个防御性界停止；「N 个片段必须停止」MUST NOT 被写成产品契约（验收 A13）

#### Scenario: A stop inside the deep path still answers

- **WHEN** 预算或取消在深链的构造或发射过程中生效
- **THEN** MUST 发布既有停止形态（`outcome = Stopped`、非 `Complete` 的 `execution`、点名原因与位置的诊断、`content = not_produced`），并在释放该结构后返回；MUST NOT abort、MUST NOT 提交部分文本（验收 A13、A14）
