## ADDED Requirements

### Requirement: One local variable's type is decided once, before its statements are built

一个局部变量（既有 `LocalVariable` 身份）的类型 MUST 在**语句被构建之前**、在既有的声明规划里决定一次，形成一个按该身份索引的类型结果，并由提升声明、就地声明、赋值、条件与返回共同消费；这些位置 MUST NOT 各自再作一次类型判定。同一个值在两条声明路径上 MUST 得到同一个类型：写在包含全部使用的区域起点上的**提升**声明与写在首个写入处的**就地**声明 MUST 一致，结论 MUST NOT 取决于遍历顺序（同一变量的两个分支先后不得改变其类型）。

决定的证据 MUST 是一份有限、写清的清单：类文件 descriptor 的 boolean 事实（`Z` 参数槽的读取、返回 descriptor 为 `Z` 的调用结果、descriptor 为 `Z` 的字段读取）、一次对本 run **已判为 boolean 的局部**的读取（传播，MUST NOT 跟随值链：一个 store 的 own value、由多个 push 合并出的值仍不证明）、以及帧为其它类型给出的既有类型事实。`0`/`1` 字面量 MUST NOT **发起** boolean 判断（`int x = 0;` 与 `boolean c = true;` 是同一份字节），但目标已经确定为 boolean 时 MUST 按其拼写（`true`/`false`）适配。**每个**写入（不只是第一次写入）MUST 按已决定的类型检查：写入的值不能拼成该类型时即为冲突。证据的传播 MUST 有界（不随输入规模无限增长），并 MUST 接线到本 run 既有的预算与取消检查；规划 MUST NOT 成为绕过预算的无限工作量，预算或取消在规划期停止时 MUST 发布既有停止。

类型未知（没有可决定的类型）或冲突（写入与已决定的类型矛盾）时，本层 MUST 终止该结构的生成，而不是发布一个猜测：受影响结构按既有 refusal 契约处理（保留 bytecode 与 origin、representation=Mixed、quality=Fallback、诊断点名相关 BCI 与物理方法），MUST NOT 发布类型矛盾的赋值（例如 `int local3; … local3 = <boolean 值>; …`）。声明的结果 MUST 区分「没有声明到期」与「声明失败（fallback 已写）」两种含义；后者之后 MUST NOT 继续写出那条赋值。本要求 MUST NOT 被读作类型保真或语义等价的声明：字面量单独构成的提升声明按其与就地路径一致的既有答案呈现，这一类型差异 MUST 被记录为边界。

#### Scenario: The two declaration paths decide the same type

- **WHEN** 一个变量由提升声明定型（它的首个写入不在包含全部使用的区域起点上），而该写入存放的值是一次对**本 body 已证明 boolean 的局部**的读取（源码形状 `boolean a = b; boolean c; if (n == 0) c = a; else c = b; if (c) …`）
- **THEN** 该声明 MUST 与就地声明路径对同一个值的决定一致：呈现为 `boolean localN`，MUST NOT 出现 `int local3; … local3 = <boolean 值>;`；把正文放进成员自己的签名后 javac MUST 接受，`boolean cannot be converted to int` 不得出现（验收 A13）

#### Scenario: The conclusion does not depend on the traversal order

- **WHEN** 同一个方法里的两个形状以交换后的分支顺序出现（`if (n == 0) c = b; else c = a;` 相对 `if (n == 0) c = a; else c = b;`），或同一变量在两条路径上分别被提升与就地声明
- **THEN** 该变量的类型、它的声明形态与它在每个使用处的拼写 MUST 与交换前一致（只有分支文本本身按源码位置不同）；依赖遍历顺序作出不同结论 MUST 使验收失败并指名该变量与相关 BCI（验收 A13）

#### Scenario: A copy chain through a boolean local keeps it boolean

- **WHEN** 提升变量的值经另一个**本 run 已判为 boolean 的局部**间接得到（源码形状 `boolean x; boolean y; if (b) { x = b; y = x; } …`），链长在有限集合内
- **THEN** 传播 MUST 让链上的每个变量与其第一个变量同型（`boolean`），MUST NOT 因为「证据在看某个写入时还没写出来」而退化为帧的 `int`；链上的每个写入同样受一致性检查（验收 A13）

#### Scenario: Every write is checked against the decision

- **WHEN** 同一变量的不同分支各有一个写入（`if (n == 0) c = a; else c = b;`，或某个分支写入一个不能拼成该类型值的形态）
- **THEN** 每一个写入 MUST 按已决定的类型检查，MUST NOT 只检查第一次写入；与决定冲突的写入 MUST 终止该结构（见「A conflicting or unknown type terminates the structure」），MUST NOT 被静默写入（验收 A13）

#### Scenario: A conflicting or unknown type terminates the structure

- **WHEN** 某个写入的值与已决定的类型矛盾，或没有任何写入能决定出可拼写的类型
- **THEN** 受影响结构 MUST 按既有 refusal 契约终止：保留 bytecode 与 origin、representation=Mixed、quality=Fallback、诊断点名相关 BCI 与物理方法；MUST NOT 发布猜测的类型、MUST NOT 为了「保持一致」而临时换一个更强的类型，也 MUST NOT 只记录日志后继续发射（验收 A13）

#### Scenario: A failed declaration is not followed by its assignment

- **WHEN** 一次局部声明因未知或冲突而不能成立（今天的 `declare()` 在这种情形下返回与「没有声明到期」相同的 `Ok(None)`，调用方随后仍会写出赋值）
- **THEN** 两种含义 MUST 被区分，且「声明失败」之后 MUST NOT 继续写出那条赋值；产物 MUST NOT 出现一个没有声明却被赋值的名字，也 MUST NOT 出现与拒绝原因矛盾的语句（验收 A13）

#### Scenario: The propagation stays inside the run's budget and cancellation

- **WHEN** 证据传播（工作队列）在本次运行的预算或取消检查下走到停止
- **THEN** MUST 发布既有停止形态（`outcome = Stopped`、非 `Complete` 的 `execution`、诊断点名原因与位置、`content = not_produced`），MUST NOT 继续构建、继续传播或提交部分产物；规划本身 MUST NOT 新增预算维度、MUST NOT 跳过既有计费（验收 A13、A14）

#### Scenario: The literal does not initiate the decision

- **WHEN** 提升声明的首个写入值是 `0`/`1` 字面量，且没有别的 boolean 证据（源码形状 `boolean x; if (b) { x = true; } else { x = false; } if (x) …`）
- **THEN** 两条声明路径 MUST 对同一个值给出同一个答案（保持 `int`，其使用按整数形态拼写为一个自洽、可编译、两侧执行一致的文本）；MUST NOT 重新接纳字面量作为声明的发起证据，也 MUST NOT 声称该形状的类型已与源码一致——这一差异 MUST 被记录为边界（验收 A13）

#### Scenario: The decided type is consumed at every site

- **WHEN** 一个已决定的 `boolean` 局部同时出现在自己的声明、后续赋值、条件与 `Z` 方法的返回处
- **THEN** 各处的拼写 MUST 与该决定一致（`boolean localN;`、`localN = true;`、`if (localN)`、`return localN;`），MUST NOT 在任一位置退回帧的 int 拼写或与 `0` 比较；这些位置 MUST NOT 各自重新判定该变量的类型（验收 A13）

#### Scenario: The existing controls keep their types

- **WHEN** 修正后运行由 boolean 参数或返回 `Z` 的调用提升的局部，以及一个只被 `0`/`1` 填充的 `int` 局部
- **THEN** 前者 MUST 保持 `boolean`（既有 descriptor 证据路径），后者 MUST 保持 `int`，两者继续通过编译执行对照；本修正 MUST NOT 用普遍改写成 boolean 或普遍拒绝换取反例通过（验收 A13）
