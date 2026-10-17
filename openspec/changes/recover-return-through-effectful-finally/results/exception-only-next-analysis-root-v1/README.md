# `FinallyCompletion.tryThrowsFinallyRuns()`：下一片独立债务架构备忘

状态：只读静态梳理；没有改共享文件，也没有运行 Cargo/JDK/JADX/CLI/Git。实际 canonical CFG、SSA 身份/consumer、effect 与 owner 证明仍待单独收费 probe，以下不把反编译注释或 classfile 线性指令提升为这些结论。

## 已登记边界

父 OpenSpec `recover-return-through-effectful-finally` 的 proposal/design/instrumentation 将 `FinallyCompletion.tryThrowsFinallyRuns()` 记为相邻正向欠账：它没有正常 return 路径，应由独立变更先定义规格；不得算进当前 return/finally 任务。finally 自身 return/throw 仍是边界。登记材料还明确要求保留整类原方法，不能删其它失败方法凑出可编译类。

精确字节码证据在 `results/remaining-controls-v1/finallycompletion-trythrows-finallyruns-classfile-memo-v1.json`：javac8/23 两个 907-byte class 的同名方法均为 Code 19 bytes、max_stack=1、max_locals=1，opcode 序列与 exception table 完全相同（constant-pool 编号不同）。表为 `[0,11) -> 10 any`；因此表的自保护重叠只覆盖 handler 的绑定 store BCI 10，cleanup 从 BCI 11 开始在该 row 范围外。这只是 exception-table 静态观察，canonical throw-site handlers 和 Operations effect 仍待 probe。指令按 BCI：

- body `[0,10)`: `bipush 7` (0), `invokestatic mark:(I)I` (2), `pop` (5), `getstatic TRY_FAILURE` (6), `athrow` (9)。
- handler `[10,19)`: `astore_0` (10), `bipush 8` (11), `invokestatic mark:(I)I` (13), `pop` (16), `aload_0` (17), `athrow` (18)。

语义要求是先观察 `mark(7)`，再抛出并保留原 `TRY_FAILURE` 对象；异常清理调用 `mark(8)` 恰好一次后重抛同一对象。该 classfile 直接证明两份控制流及范围；还不能证明任何 SSA value 是同一对象，或每个 throw-site 的实际 handler 列表。

当前现存报告（`finallycompletion-current-05e-and-sealed-ex-memo-v3.json` 与完整类 manifest）将它记为 `jre_guard_finally_copy` 加 `jre_region_uncovered_blocks`，并说 handler 路径“重复 normal path 也运行的代码”。这句是现存候选解释，不是 classfile 已证明的事实：这两个 sealed Code 中 `mark(8)` 只出现一次（BCI 13），位于 handler cleanup；`mark(7)` 只出现一次（BCI 2），位于 protected body。代码报告的“copy”归类需要真实 CFG/SSA probe 校正，不能沿用为两份 cleanup copy。当前 normal-flow view 遗漏 handler block BCI 10 是报告记录。报告本身不是 CFG/SSA 证明。两种编译器的完整当前输出在 `instrumentation.md` 记作各 4114 bytes、3 个 missing-return errors；不是本次产品的 runtime 证据。

## 当前架构接缝

当前 `guard::FinallyCompletion::Void` 不适用于此形。它表示 protected body 有正常完成，cleanup 正常副本落穿到方法 void `return`，且 handler 自保护行只覆盖绑定 store。目标 body 以 `athrow` 结束、没有正常出口，方法 descriptor 还是 `()I`；exceptional cleanup 完成后也不是 `Void`，而是重抛原异常。复用该变体会把“无返回值”误当作“异常完成”，破坏完成类型与保护行含义。

`guard::Shape::Finally` 的现有高层表达（可写为 Java `try { body } finally { cleanup }`）仍是合适的最终 AST 接缝；Builder 已能以 `StmtKind::Try { finally_body: Some(..) }` 表示它。新工作只需增加目标 completion 和复制关系的私有证明/计划入口，不足以证明必须扩展 public `Shape` / public enum。当前 `FinallyCompletion::SavedReturn/Joined/Void` 也没有“只允许 throw 的 body + normal cleanup 后恢复 pending primary exception”这一完成值。优先评估 crate-private bounded descriptor 或窄的内部判定/Writer路径；实际 probe 后再定是否放进已有 private Plan，而非立即加泛化 cleanup tree 或公开枚举。

已有普通 try/catch 读器 `guard::exception_only_catch` 和 `CatchTypes::ProvenThrowable` 从数据形状上很接近：body 只有一个块并以 throw 结束，catch-all entry 先 store，handler 尾部 load 同 slot 并 rethrow，throw-sites/边须闭合。但其 cleanup 语法 `exception_only_cleanup` 目前严格要求四条指令组成静态整数读、常数、加、写；目标 cleanup 是 `bipush; invokestatic mark; pop`，有调用副作用，故现规则拒绝。`Catches` 最终输出普通 `catch(Throwable)`，而非 `finally` 结构；即使把调用加入这个 grammar，也只是一个可能语义等价的重写路径，不证明/表达此债务登记的 finally。且 guarded shared-finally candidate 在普通 catches 前先检查；目标的已观察 fallback 已来自 `jre_guard_finally_copy`，不能只扩普通 catch helper 假定它自然接管。

因此不建议把 `FinallyCompletion::Void` 改名/重用，也不建议只扩 `ExceptionOnlyCatchAll` 的 cleanup matcher 作为正向解决方案。优先沿 private proved-finally 入口建立一项有限的新完成关系；公开 AST 保持 Try/finally。若实际 probe 表明现有 finally-copy proof 已覆盖大部分条件，应最大化复用其 copy/range/handler identity helper，仅给“body cannot complete normally; exceptional copy completes by rethrow”一个窄的私有 plan/completion 描述。不要涵盖 finally 自身显式 return/throw、nested/branch/loop；普通 cleanup 调用的异常覆盖按 Java finally 语义自然保留，不另建 arbitration 机制。

## 对照工具观察

固定 `openspec/evidence/java-syntax-2026-09-22/finally-completion/jadx.java.txt`（JADX 1.5.6 历史产物）把 `tryThrowsFinallyRuns` 写成 try/finally，但把原 `mark(7)` 调用错写成无副作用的 `int i = 7`；相应历史运行输出 `try-throw-preserved` trace 为 `8`，原程序是 `78`。因此这个对照不能当作目标行为正确的 oracle。相同对照中 `finallyThrows()` 的 finally 副作用变成 `566`，说明外部/“throwing cleanup”路径有已知重复执行错误。独立 finally-child 应以 sealed 原始 class + Java source runner 为 oracle，分别检查 try side effect、cleanup once、原异常类型/message/对象身份；JADX 仅作为有明确失败已知项的比较来源，不是证明。

历史 `analysis.md` 记录了源级原程序 oracle：`try-throw-preserved:throw:java.lang.IllegalArgumentException:try=true:finally=false:78`。该文档是 2026-09-22 的 source audit，不是当前冻结产品的重放；新 OpenSpec 应重新封存输入和独立 class/runner，并标注旧观察的来源日期。

## 下一独立 OpenSpec 的最小任务建议

1. 先对两个封存 compiler class 逐一做真实 `Facts`/SSA probe：实际 `examine`/`resources` 首次拒绝，row ordinal/order、catch type、canonical block/edge list、throw-site handlers、所有 instruction effects、handler binding→local reload→`athrow` SSA 链、method-owned physical BCIs、source map/ownership。不能只采 classfile JSON，也不能拿已有 `FinallyCompletion::Void` 的证据外推。
2. 把正向范围写成：只支持单条 straight-line protected body，其已证完成是原始 `athrow`；单个有界 catch-all exceptional cleanup，cleanup 可包含普通 may-throw 调用；cleanup 后按 SSA identity 重抛 pending primary（当 cleanup 正常完成时）。在此无正常完成的 body 中，证明唯一 cleanup 只从 exceptional transfer 到达且恰好一次；不要假设存在 normal cleanup copy。若实际 CFG 暴露第二条 cleanup 入口，再分别证明它们的范围与 effects关系。必须证明原 catch-all 的保护范围不吞掉 cleanup 调用的异常；AST 若写为 Java finally，cleanup 自身运行时异常/错误自然覆盖 pending primary，这属于 finally 完成规则，应保留，不是拒绝条件。cleanup 中禁止显式非局部 `return`/`throw` 是当前范围边界；这与可能抛出的普通调用不同。
3. 反例拒绝至少覆盖：normal return、异常 handler 不重抛原值、wrong slot/value、cleanup 指令/receiver/argument 不同、异常表缩放/增加竞争 row、cleanup 落入不应保护它的原 handler、handler/body 非局部边、finally 自身显式非局部 return/throw、额外前驱和非连续/遗漏 physical BCI。普通 cleanup call 可能抛出的 RuntimeException/Error 应纳入动态矩阵，验证它覆盖原异常且执行顺序正确，不作为拒绝边界。
4. 动态 oracle：原对象实例同一性（fixture 可通过 runner 从捕获点作 `==` 比较），`mark(7)`、`mark(8)` 精确次序/次数、原异常类别与 message；再做 fresh twincompiler 完整 fixture 编译 + rebuilt 执行。目标原 class 仅此方法的静态 proof 不能消除完整 `FinallyCompletion` 另外两个独立红项。

## 只读 probe 草稿

输入固定为 memo 中 javac8 与 javac23 两个 `FinallyCompletion.class`，精确方法 identity `tryThrowsFinallyRuns()I`。probe 应在 default 与所有 recovery profiles 分别：

- 打印 `guard::examine` 的 Verdict 与实际首拒；不要只打印最终报告 warning。
- 逐行输出 exception rows (ordinal/start/end/handler/type)、canonical blocks、normal/exception edges 与 `throw_sites()` handlers；核 `[0,11)->10` 的 handler binding 是否落在 row 范围末端。
- 对 BCI `0,2,5,6,9,10,11,13,16,17,18` 列出 `Operations` effects/may_throw 与 SSA defs/reads/writes；检查 BCI 6 到 9 的 primary identity，以及 10 store → 17 load → 18 throw 的完整 same-value 链；逐个 throw-site 检查 handler set；静态 row `[0,11)->10` 只自保护 store10，call 13 位于范围外，但其实际 canonical handler set/effect 仍由 probe 确认。
- 打印 ordinary catches/exception-only candidate 若可从现有内部函数独立调用的 verdict，明确 `exception_only_cleanup` 首拒处；只做事实定位，不临时放宽 grammar。
- 用 paid budget 测试首拒路径也计费；构造返回计划时再补精确 physical BCIs/unique ownership/source-map closure。

上列 BCI 是 classfile 固定布局，probe 还需对 javac8/23 都按 pinned SHA/Code digest 验证；生产候选仍须由相对关系与 Operation/SSA 通用证明导出边界，禁止以类名、方法名或 BCI literal 作为识别规则。

## 精确来源 pins

- 父 ledger: `openspec/changes/recover-return-through-effectful-finally/{proposal.md,design.md,instrumentation.md}`。
- 目标 Code 证据: `openspec/changes/recover-return-through-effectful-finally/results/remaining-controls-v1/finallycompletion-trythrows-finallyruns-classfile-memo-v1.json`；class SHA256 javac8 `49e649478c3d45ba6486029778a8415239caafdd93234a1d5956f5ccf3031393`，javac23 `2b8902d998065d2d1747abaeba386e006cdf5ff7f1311abb28506e8d8dc0cc94`；目标 Code SHA256 分别 `203dc82caf76e6ad5c022413d8bc424593b51e27bf4aaa75e41fe7c17a97a7d1` 与 `8e57e4c33c413043d2d4d74bda593de2edef9eef7cad27855d31b88389d18430`。
- JADX 历史结果: `openspec/evidence/java-syntax-2026-09-22/finally-completion/{FinallyCompletion.java,javap.txt,jadx.java.txt,jadx-run.txt,original-run.txt,analysis.md}`；新一轮应重新跑/封存，不复用旧文本当成当前验证。
- 既有异常专用 catch: `crates/jarde-java/src/guard.rs` 的 `CatchTypes::ProvenThrowable`、`exception_only_catch`、`exception_only_cleanup`；已有 finally completion/shape: 同文件 `FinallyCompletion`/`Shape::Finally`；构造/打印接缝：`crates/jarde-java/src/region.rs` 与 `crates/jarde-java/src/build.rs` 对 `Region::Try`、`Region::Guard` 和 `StmtKind::Try` 的处理。
