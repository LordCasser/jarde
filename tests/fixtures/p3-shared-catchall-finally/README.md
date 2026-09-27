# CF-16 最小类族及 1.2 所有权门槛

基线：Jarde `134e1e5d`；JADX checkout HEAD `2fb1b16386941660fda07e9017285aec40fcb37f`。本目录的 `SharedFinally.java` 只含目标方法、计数器和读计数方法；外部 `SharedFinallyRunner.java` 调用正常与具名 catch 路径。`v8/` 是 `javac --release 8 -g:none` 编译的原 class。`javap.txt` 固定其物理指令和异常表；`jadx/sources/defpackage/SharedFinally.java` 是固定 checkout 的完整输出；`jarde/SharedFinally.java` 是本基线 CLI 的完整 class-source 输出。JADX 的 package 由其输出决定，故使用同包的 `jadx/JadxRunner.java`。

原始 `handled` 的异常表按物理 ordinal 是：

| ordinal | 半开保护范围 | handler | 类型 |
| --- | --- | --- | --- |
| 0 | `[4,21)` | 31 | `IllegalArgumentException` |
| 1 | `[4,21)` | 45 | catch-all |
| 2 | `[31,35)` | 45 | catch-all |

两条 catch-all 行共用 BCI 45 handler。正常返回值在 BCI 18 压入、20 保存 slot 1，清理副本在 21–26，29 读 slot 1、30 返回。具名 catch 在 31 将异常保存为 slot 1 的**另一条定义**，32 压入返回字面量、34 保存 slot 2，清理副本在 35–40，43 读 slot 2、44 返回。handler 在 45 保存原异常 slot 3，副本在 46–51，54 读 slot 3、55 重抛。slot 1 的复用不能作为跨 catch 的同一 SSA 值；两个正常完成的返回值须分别追溯其保存定义。三份 `getstatic; iconst_1; iadd; putstatic` 的效果均是给同一字段加一。这里记录的是物理定义/使用关系；尚未给出引擎 SSA 等价证书。

在本 checkout 重放：

```sh
javac --release 8 -g:none -d tests/fixtures/p3-shared-catchall-finally/v8 tests/fixtures/p3-shared-catchall-finally/SharedFinally.java tests/fixtures/p3-shared-catchall-finally/SharedFinallyRunner.java
java -Xverify:all -cp tests/fixtures/p3-shared-catchall-finally/v8 SharedFinallyRunner
# normal:1 / caught:1

cd /Users/lordcasser/workspace/testzone/jadx
./gradlew :jadx-cli:run --offline --args='-d <worktree>/tests/fixtures/p3-shared-catchall-finally/jadx <worktree>/tests/fixtures/p3-shared-catchall-finally/v8/SharedFinally.class'
# 上述命令的 <worktree> 换成当前 checkout 绝对路径；固定 HEAD 如上。
cd -

javac --release 8 -g:none -d /tmp/jarde-cf16-jadx-classes tests/fixtures/p3-shared-catchall-finally/jadx/sources/defpackage/SharedFinally.java tests/fixtures/p3-shared-catchall-finally/jadx/JadxRunner.java
java -Xverify:all -cp /tmp/jarde-cf16-jadx-classes defpackage.JadxRunner
# normal:2 / caught:1

javac --release 8 -g:none -d /tmp/jarde-cf16-jarde-classes tests/fixtures/p3-shared-catchall-finally/jarde/SharedFinally.java tests/fixtures/p3-shared-catchall-finally/SharedFinallyRunner.java
# exit 1；SharedFinally.java:21 缺少返回语句，不能执行
```

Jarde 的 `handled` 当前整体拒绝：`local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice`。这是正确的安全状态；不能把不可运行的 Jarde 源码算作行为等价。固定 JADX 正常路径在 try body 已保留一次 `cleanupCount++`，又在 finally 里执行一次，故是 `normal:2`；它只能作为差异证据，原 class 才是运行 oracle。原 `FinallyOnce` 的已冻结 `normal:2` 差异不因此消失。

为隔离清理语句类型，`SharedFinallyCall.java` 是同形对照：唯一额外的私有 `cleanup()` 将计数器加一，三个副本各只含一条相同的 `invokestatic cleanup:()V`。`SharedFinallyCall.javap.txt` 固定其 ordinal 0 `[4,21)→26 IAE`、1 `[4,21)→35 any`、2 `[26,30)→35 any`；两份返回分别保存于 BCI 20/29，三份调用在 21/30/36，handler 在 35 保存异常并于 40 重抛。原 class 经 `javac --release 8 -g:none` 和 `java -Xverify:all` 输出仍为 `normal:1`、`caught:1`。`jarde-call/SharedFinallyCall.java` 显示 Jarde 仍以同一 slot 1 跨 quoted fallback 诊断整体拒绝，完整源码重编在第 29 行缺少返回。故 `iadd` 是增量效果证明要求，但**不是**共享 row/双返回结构门槛失败的唯一原因；调用形态也未取得所有权闭包。

## 1.2 结论：门槛未通过，保留拒绝

逐点核对现有 Guard/Region/Builder，尚无法证明该类族的有界子正文能完整且一次性认领：

1. `guard.rs::prove_finally_copy` 只拆出 handler 前的**最后一个**正常返回，`FinallyCopyProof` 只有一条 `row_ordinal`、一份 `saved_return` 和一份 `normal_cleanup`。对 ordinal 1，这个最后返回是 catch 路径 BCI 44，其副本从 35 开始，而该行止于 21；现有半开边界校验正确拒绝。调用对照样本同样是 catch 副本始于 30、row 止于 21。原 `cleanupCount++` 还包含 `iadd`，现有 `cleanup_sequence` 只接受 Push/Invoke/Field/pop，因而没有三副本效果证书；调用对照消除了这个额外限制却仍不闭合。放宽任一校验都会把未经证明的副本折叠。
2. `region.rs::Walker::region_at` 在同一入口先走 `starts_catch`；`guard::catches` 会请求 guard 裁决，guard 对候选 catch-all 拒绝后，不生成具名 catch 子 Region。现有 `Frame::own_finally` 仅存一条 `(ordinal, span)`，`finally_edges_accounted` 对另一条异常边返回 false；`finally_body` 只检查一份受保护正文及一次 save，且该 frame 抑制 nested catch 入口。没有经证书封闭的 `[4,21)` try、`[31,35)` catch 和两条 catch-all 边的联合 owner。
3. `build.rs` 的 `Shape::Finally` 分支只带一个 `(save, return)`，构建 `StmtKind::Try` 时固定 `catches: Vec::new()`；它的临时 `finally_return` 也只针对一处保存。当前失败的 slot 1 局部作用域与两个 saved return，不能靠放宽声明规则或把 catch 副本写进 try body 解决。必须先证明两个子 Region 和两个局部作用域、异常行优先级与失败 checkpoint 闭合，才能提交 AST 与 visited。

以上是当前接缝的具体阻碍，并非证明该形态永远无法实现。按照任务 1.2 的门槛，本分支不新增通用异常 IR、不放宽旧证书、不输出部分 try；2.x/3.x 维持未完成。

下一次可拆的**最小内部合同**是：私有 Guard 证书把 ordinal `[0,1,2]`、`[4,21)` 与 `[31,35)`、唯一 handler 45、两组 `(save, cleanup, load, return)` 及 `(45, cleanup, 54,55)` 同异常重抛作为一个不可分割的结果；清理的 `iadd` 只能在三个副本的 SSA 操作/值/字段目标与效果顺序全部相等后纳入该证书，不能单独放宽通用 `cleanup_sequence`。Region 必须先分别恢复恰好覆盖 `[4,21)` 和 `[31,35)` 的子正文，且逐条匹配 ordinal 0 的优先级和 ordinal 1/2 的异常边；每个子正文的物理块集与证书 `owned` 不交叠、并集完整。Builder 应在同一 checkpoint 下给两个 saved return 各安放其原值，只将一份清理放进既有 `StmtKind::Try` 的 `finally_body`，将 catch 参数与正文放进 `catches`。所有权、声明、来源或预算/取消任一步失败都回滚整个候选，原物理 BCI 和三条异常行保持可追溯拒绝。上述合同成立后才可运行 Jarde 完整源码与原 class 的行为对照。

此门槛提交未改 Rust 恢复代码。基线复放：`jarde-java` 的 `finally_copy_tests` 10/10、`p3_finally_straight` 5/5、`p3_typed_catch` 6/6；`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`openspec validate recover-shared-catchall-finally --strict`、`git diff --check` 均通过。Rust 使用专用 `/tmp/jarde-cf16-target` 编译目录，并在提交前执行 `cargo clean`。

## 调用型三副本切片的实现验收

本次只恢复 `SharedFinallyCall.handled` 的无参静态 void 调用家族。Guard 要求三行 ordinal 连续、两段受保护区间及 handler 严格对应，三份 `invokestatic` 目标和 `()V` 描述符相同，两份返回的字面量生产者经 SSA 直接连接到各自 save 且仅由它消费，handler 保存的原异常经 load 到 `athrow`。正常完成只允许两处已证明的返回；额外出口、额外入口、不同调用目标、异常行改变均拒绝。Region 将 `[4,21)` 和 `[26,30)` 作为互不重叠的有界正文原子认领，Builder 在一个 checkpoint 内输出一处具名 catch 和一处 finally。清理方法名没有语义门槛。原有 `cleanupCount++` 与 `FinallyOnce.escaping` 仍是独立切片。

冻结的完整源码在 `SharedFinallyCall.java`、`jadx-call/sources/defpackage/SharedFinallyCall.java`、`jarde-call-after/SharedFinallyCall.java`。后两者分别配 `jadx-call/JadxCallRunner.java` 与原 `SharedFinallyCallRunner.java`。固定 JADX checkout HEAD 仍为 `2fb1b16386941660fda07e9017285aec40fcb37f`。三方分别执行 `javac --release 8 -g:none -Xlint:-options` 编译**完整类与 Runner**，然后 `java -Xverify:all`：

| 来源 | normal | caught | 编译/验证 |
| --- | --- | --- | --- |
| 原 Java 8 class（SHA-256 `644c0da948fcb64673898bff8f5509d9cefac7ecd7f58399e0f18b0491f2f325`） | `normal:1` | `caught:1` | 通过 |
| 固定 JADX 完整源码 | `normal:2` | `caught:1` | 通过，但正常路径重复清理 |
| 本次 Jarde 完整源码 | `normal:1` | `caught:1` | 通过 |

可重放命令（在仓库根目录）：

```sh
javac --release 8 -g:none -Xlint:-options -d /tmp/jarde-cf16-original-final tests/fixtures/p3-shared-catchall-finally/SharedFinallyCall.java tests/fixtures/p3-shared-catchall-finally/SharedFinallyCallRunner.java
java -Xverify:all -cp /tmp/jarde-cf16-original-final SharedFinallyCallRunner
javac --release 8 -g:none -Xlint:-options -d /tmp/jarde-cf16-jadx-final tests/fixtures/p3-shared-catchall-finally/jadx-call/sources/defpackage/SharedFinallyCall.java tests/fixtures/p3-shared-catchall-finally/jadx-call/JadxCallRunner.java
java -Xverify:all -cp /tmp/jarde-cf16-jadx-final defpackage.JadxCallRunner
javac --release 8 -g:none -Xlint:-options -d /tmp/jarde-cf16-jarde-final tests/fixtures/p3-shared-catchall-finally/jarde-call-after/SharedFinallyCall.java tests/fixtures/p3-shared-catchall-finally/SharedFinallyCallRunner.java
java -Xverify:all -cp /tmp/jarde-cf16-jarde-final SharedFinallyCallRunner
```

`p3_shared_catchall_finally` 定向测试还检查完整输出含两个字面量 return、唯一具名 catch/cleanup、无 bytecode fallback，且 source map 覆盖 `handled` 全部 24 个物理 BCI：`0,1,4,5,8,11,12,14,17,18,20,21,24,25,26,27,29,30,33,34,35,36,39,40`。同一测试把异常行交换、把 try 的 catch-all 保护终点扩至 24，并把一份清理调用改指向另一个真实 `static void other()`；均拒绝 finally 恢复。另一个 Java 8 样本 `SharedFinallyExtraReturn` 在 try 内加入提前返回，也被拒绝。额外返回样本经 `java -Xverify:all` 输出 `early:1 / normal:1 / caught:1`。`other()` 目标样本先由源码编译，再把 catch 副本的 `invokestatic #25 cleanup:()V` 改为 `#13 other:()V`；修改后的 class 经 `java -Xverify:all` 输出 `normal:1 / caught:2`。交换行与扩围行的字节码也分别通过 `-Xverify:all`，拒绝属于恢复证书而非验证器失败。Guard 单测另覆盖缺行/handler 变化、预算耗尽与取消传播。
