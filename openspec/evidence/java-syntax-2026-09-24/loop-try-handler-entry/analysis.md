# 循环体内 try/catch 处理器入口审计

本样本对应 `openspec/changes/present-proved-java-structure/evidence/baseline-three-way.md` 第 78 批的 `Try78.loopTry`：循环体内有受保护代码、命名 `RuntimeException` 处理器，处理器后仍执行递减并回到循环头。不同点是 `try` 中显式调用 `maybeFail`，该静态调用在参数匹配的迭代抛出无参 `IllegalStateException`（属于 `RuntimeException`），使处理器路径能由 Runner 稳定触发。

## 构建与执行

环境：OpenJDK / `javac 23.0.1`，JADX `1.5.6`。固定输入由以下命令以 Java 8 classfile、无调试信息编译：

```sh
javac --release 8 -g:none -d . LoopTryHandlerEntry.java Runner.java
```

`javac` 仅打印当前 JDK 对 `--release 8` 的过时选项提示。冻结的 `LoopTryHandlerEntry.class` 为 395 字节、major version 52，SHA-256：`79254ba10b17846830ed420a8c13451a92630209edef9cb5d977fb6765e77f38`。Runner 在 `-Xverify:all` 下执行两条路径：

```text
normal=6
caught=0
```

`normal` 以 `failAt=-1` 走完三次静态调用，结果为 3+2+1。`caught` 在 `n=2` 抛出并捕获异常，将累加器设为 -1，之后循环仍递减并处理 `n=1`，最终为 0。因此 catch 入口及其后续循环边均实际执行。

## JADX 往返

使用 `jadx --no-res -d <dir> LoopTryHandlerEntry.class` 对冻结 class 反编译。原始输出保存在 `LoopTryHandlerEntry.jadx.raw.java`；为了默认包重编，仅删除 JADX 自动生成的 `package defpackage;` 行，结果保存为 `LoopTryHandlerEntry.jadx.java.txt`。按 Java 源文件命名规则，将同内容暂存为 `/tmp/loop-try-handler-entry-jadx-src/LoopTryHandlerEntry.java`，再运行：

```sh
javac --release 8 -g:none -d /tmp/loop-try-handler-entry-rebuild /tmp/loop-try-handler-entry-jadx-src/LoopTryHandlerEntry.java Runner.java
java -Xverify:all -cp /tmp/loop-try-handler-entry-rebuild Runner
```

原 class 与 JADX 重编 class 的逐行运行输出相同：

```diff
 normal=6
 caught=0
```

JADX 重编 class SHA-256 为 `78561427b8cf0c61d63ac43e50fe8eafc01ce449750e964c749c6aae0be610b5`，与原 class 字节不同。结构上，JADX 保留了 `while (n > 0)`、包围静态调用和累加的 `try`、`catch (RuntimeException)`、catch 后递减，以及回到循环头的控制流；它把 `result = result + ...` 规范化为 `+=`，把 `n = n - 1` 写成 `n--`。异常表保护范围是 BCI `[6,14)`，处理器入口 BCI 17，类型为 `RuntimeException`；保护范围仅覆盖循环体的调用与累加，不覆盖循环头或递减。`maybeFail` 的异常路径由 `new IllegalStateException()` 与 `athrow` 构成，不含 StringBuilder 或其他嵌套构造；其正常返回仍为参数 `n`。

## 文件哈希

- `LoopTryHandlerEntry.java`：源码输入；SHA-256 `4a0ad2d2df34dda30c7e7bace665f058d3a706d5c8692eaa603b489325cb8029`。
- `Runner.java`：源码执行器；SHA-256 `33bebd614b2da4f96e72774fb5bfd217f95c748be22928d237553862a1b33b75`；编译所得 `Runner.class` SHA-256 `2f3470ea8d7d35d802c25f10c088c8e17521f59c74070dbac2ec208acbd10b31`。
- `LoopTryHandlerEntry.class`：冻结 Java 8 输入；SHA-256 `79254ba10b17846830ed420a8c13451a92630209edef9cb5d977fb6765e77f38`。
- `LoopTryHandlerEntry.jadx.raw.java`：JADX 原始输出；SHA-256 `58e184ff68367a120a84f4145a275c8595014f1d7e97fe89d1871715d9f366f8`。
- `LoopTryHandlerEntry.jadx.java.txt`：去掉默认包声明的重编文本；SHA-256 `6b107bbcb56876cd349b5002060787d82492762c7368be7e0420a0ce55978a99`。

## 实施前 Jarde 独立复核与架构归因

root 原样重编冻结 class 得到相同 SHA，并独立重编/运行原源码与冻结的 JADX 文本，两行逐字相同。实施前 Jarde CLI SHA-256 `076c56649e04a4c84b8c10b03b13157cda8aa19e8c384f89997002f3e6e8ae88`；`class-source --policy single-class --release 8 --evidence all` 对 `<init>` 和 `maybeFail(II)I` 无 warning、无引用，但 `loopTry(II)I` explanation only，唯一主诊断为 `jre_region_irreducible`，块 `[2, 6, 20]`。这样已排除异常对象构造或字符串拼接导致整类失败的干扰。

`normal_flow.rs` 建图时排除 `CanonicalEdgeKind::Exception`，但保留处理器入口 BCI 17 到循环体 BCI 20 的普通边。BCI 17 从方法入口经普通边不可达；它由异常表 `[6,14) -> 17` 进入，再在 20 与正常路径汇合。现有 `irreducible_blocks()` 对全图做 SCC，并把 SCC 外任意普通前驱算作第二入口；因此把处理器的 17→20 与方法入口经 BCI 2 的路径一起视为循环的双入口，先于 `try_region` 走访拒绝整方法。这里的因果结论由字节码边、Jarde 诊断及当前代码共同支持；不能说 Jarde 没排除异常边，也不能以忽略所有处理器为修复。

可复用现有 `CanonicalHandlerRow` 的 handler、protected 块身份与 `NormalFlowView` 的 loop/可达关系：只有证明该处理器属于这个循环内的受保护范围、且它的普通后继是已证明的 try/catch 汇合时，才不将这条后继当作外部进入循环的普通 CFG 入口。然后仍由 `try_region` 认领处理器和汇合后的 BCI 20，核验 catch 路径也执行递减并回到 BCI 2。处理器来自循环外、跨越不相容保护区，或有其它普通入口时须继续拒绝。此处是正常流结构检查的边界修正，不需要新的全局 pass，也不能简单删除处理器节点或异常边。JADX 的 `RegionMakerVisitor` 在主区域之后调用 `ExcHandlersRegionMaker`、再调用 `ProcessTryCatchRegions`；可借鉴“先识别循环普通入口、再按异常表归属接回处理器”的顺序，但其区域重排不是 Jarde 的正确性证明。

## 入口子集实施与剩余边界

2026-09-24 的入口子集实现仍保留 `NormalFlowView` 中的 17→20 普通边。`region` 在 SCC 和自然循环入口判定前，逐 handler 核对同目标的全部异常表行：handler 没有普通前驱、恰有一个普通汇合后继；每行都是命名 catch；原始 `[start,end)` 覆盖的全部块都在同一循环体内且由循环头支配，不能覆盖循环头；每个由该行保护的块的普通路径在到达终止或环之前都到达汇合。任一行不符时不豁免。`CanonicalHandlerRow.protected()` 对有抛点的行只列真实抛点，故全范围检查必须另读原异常表，不能只据此集合放行。豁免仅用于入口判定，`try_region` 仍须认领 catch、汇合与回边。

原冻结 class（SHA-256 `79254ba10b17846830ed420a8c13451a92630209edef9cb5d977fb6765e77f38`）未修改。它越过入口判定后，暴露出独立的 `build.rs` 局部声明门禁：`local 2 crosses a protected region, but SSA does not prove that every path to its reads reaches a presented write`。`result` 在 try 内以 `result + maybeFail(...)` 更新，既有 `all_reads_reach_presented_writes` 只接受直接整数常量的存储；本子集未改该门禁。因此原 `loopTry(II)I` 仍是 explanation only，不能把它记为完整 3.1 验收。该局部作用域问题已在 `preserve-local-scope-across-exception-regions` 独立规划。

隔离入口机制的自写相邻样本 `tests/fixtures/p3-loop-try-handler-entry/LoopTryHandlerEntryArgs.java` 改用第三个参数承载累加器，循环、真实抛出、catch 内赋值、汇合后递减和回边保持同一形状；`loopTry(III)I` 的保护范围 `[4,12)`、handler BCI 15、汇合 BCI 18。编译命令为 `javac --release 8 -g:none -d tests/fixtures/p3-loop-try-handler-entry/v8 LoopTryHandlerEntryArgs.java Runner.java`（在该 fixture 目录运行）；源码 SHA-256 `ca53efdb5f7a29cb35ead82c6b7fedbfb6b9c75d53f75671f968726a78200f8f`，冻结 class SHA-256 `e83048988366411b83e814ed69a3e3c22d8461c264bf7debfdfde14dd17ad6a5`。JADX 1.5.6 原始输出 SHA-256 `8b2bf8ed9ed3e02f1e30dbc6df52e5057001180f9f20d280c291925efe4ca6e7`；只删去默认包声明的 Java 8 重编文本 SHA-256 `1cc23ff89b197ef5ed75a1dcec9e987bc2ada3c903d00e3854292352af31ec39`。原 class、JADX 重编与 Jarde 整类 Java 8 重编均在 `java -Xverify:all` 下逐行输出 `normal=6`、`caught=0`。Jarde 文本含完整 `while`、`try`、`catch (java.lang.RuntimeException ...)`、`arg0 = arg0 - 1`、`return arg2`，无 `@bytecode`；source map 钉住 handler BCI 15 的 derived origin，以及 BCI 17、18 的 direct origin。

拒绝边界使用两条仅改异常表范围的变体：单行 handler 与共用 handler 的第二行分别由 `[4,12)` 改成 `[0,12)`，新增覆盖循环测试。两条 class 都通过 `java -Xverify:all` 加载；BCI 0–4 没有同步抛错指令，所以这里只证明对不相容**表范围**的保守拒绝，不声称运行时有外部异常入口。两条均保持 `the graph is not reducible` 的整段引用；共用 handler 的另一行虽仍合法，也不能为不相容行借到豁免。`cargo test --test p3_loop_try_handler_entry --test p3_loop_transfers --test p3_switch_loop_exits --locked` 的 5、5、3 项通过，预算耗尽和预取消均不发布半个循环。当前共享树另观察到 `p3_nested_try` 的 1 项与 `p3_typed_catch` 的 2/5 项失败，输出分别为 catch 参数作用域、`finallyIncrements` 的 quoted fallback 和 `twoCatches` 的 catch 参数作用域；未独立归因，亦未混入本子集修复。

root 用实施后 CLI SHA-256 `50fbc4ea18371df51e68056acdf55fa084daf1ee535fca0d5eef70d70991f1c5` 独立重放：参数正例源码重建 class 与冻结 SHA `e83048988366411b83e814ed69a3e3c22d8461c264bf7debfdfde14dd17ad6a5` 完全一致；Jarde 整类无 `@bytecode`、通过 `javac --release 8`，原/JADX/Jarde 的 `normal=6`、`caught=0` 逐行相同。JSON source map 中 BCI 15 是 try 区域的 derived origin，BCI 17/18 有 direct origin。root 还独立制作两个扩围变体，均可在 `-Xverify:all` 加载并运行正常路径、均由 Jarde 整段引用且报告不可约。Agent 的独立 Cargo target 已通过 `cargo clean --target-dir` 清理约 3.3 GiB。
