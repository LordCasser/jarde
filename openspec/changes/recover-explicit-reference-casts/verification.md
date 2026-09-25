# 验证记录

本记录对应 `ReferenceCastProbe`，class 由自写源码用 `javac 23.0.1 --release 8 -g:none`
生成。提交 class 的 SHA-256 为
`cd98f688dcdc023cd76f15c7270d47128aee277fbfe6ff2b2eb6485903c3f499`，暂不修改共享
census/fingerprint。

## 三方修前证据

原始源码覆盖以下真实字节码形状：

```text
direct:                 aload_0; checkcast String; areturn
receiver:               aload_0; checkcast String; invokevirtual String.length; ireturn
intArrayRead:           aload_0; checkcast [I; iload_1; iaload; ireturn
stringArray:            aload_0; checkcast [Ljava/lang/String;; areturn
multiArray:             aload_0; checkcast [[Ljava/lang/String;; areturn
local:                  aload_0; checkcast String; astore_1; aload_1; areturn
staticParameter:        aload_0; checkcast String; invokestatic consume; areturn
callOnce:               invokestatic source; checkcast String; areturn
nested:                 aload_0; checkcast Runnable; checkcast Number; areturn
nestedCall:             invokestatic source; checkcast Runnable; checkcast Number; instanceof Comparable; ireturn
discardedCastBeforeCall: aload_0; checkcast String; astore_1; invokestatic source; areturn
```

`jadx 1.5.6` 对照保留了上述显式转换和括号；其中 `local` 的无用局部被 JADX 压缩，
这不作为 jarde 的执行预言机。修复前的 CLI 命令为：

```text
target/debug/jarde-cli class-source \
  --input tests/fixtures/p3-reference-cast/v8/ReferenceCastProbe.class \
  --class ReferenceCastProbe --policy single-class --release 8 --format text \
  > /tmp/reference-cast-jarde-before.java 2> /tmp/reference-cast-jarde-before.report
```

修复前 `direct`、`receiver`、`intArrayRead`、`stringArray`、`multiArray`、`local`、
`staticParameter`、`nested` 均只有 `@bytecode` 说明，没有显式转换正文；`callOnce` 和
`nestedCall` 只保留了 `source();`，并分别说明 BCI 3/6 的检查未被呈现，后者还拒绝了
`instanceof` 下游消费。这个结果是预期的
修前失败，不能当作最终验收。

## 修复后验证

Rust 测试 `tests/p3_reference_cast.rs` 的 5 个非 ignored 用例全部通过。正面断言覆盖对象、
接收者、原始数组、引用数组、多维数组、局部、静态参数和一次性调用；`nested` 的源码是
`(Number) (Runnable) arg0`，Java 的外层书写顺序保留了内层 Runnable 检查先执行的语义。

`nestedCall` 的 `instanceof Comparable` 是合法但当前不支持的最终消费。拒绝文本保留
`@bytecode 6 3 0`（外层 cast、内层 cast 和 `source()` 调用）以及 BCI 9 的下游指令，未丢失
任何中间运行时检查。内存 patch 将 `discardedCastBeforeCall` 的 `astore_1` 改为 `pop`；普通
cast 不被静态调用限定符形状误认，检查和 pop 留在 quote 中，独立的 `return source();` 只
出现一次。原始 `astore_1` 版本则恢复为局部声明后调用，属于可编译的正常正文。

ignored JDK 对照先编译运行原始 class 与临时 runner，再将 `Engine::class_source` 的真实文本
写入另一临时目录，用同一 runner 在 `javac --release 8 -g:none` 下编译并逐字比较 stdout。
runner 覆盖正常值、null、错误对象类型、错误数组类型、引用数组 null、调用计数、
`ClassCastException`、`NullPointerException` 及嵌套转换；测试实际通过。为了让拒绝正文不阻塞
整类编译，helper 只剔除 `nestedCall`，没有手写替换任何被测方法正文，也没有提交 runner
class。`discardedCastBeforeCall` 已纳入真实编译和运行比较。

## 主代理独立验收（2026-09-23）

- `cargo test --locked -p jarde-java`：93 个 unit、32 个 recovery 和43个 patterns，共168项通过。3个 bridge 用例保留原 verdict/refusal，并改为检查显式普通 cast；2个调用消费者现在验证实际 cast 正文。没有把拒绝逻辑删掉来通过测试。
- `p3_eval_context` 10/10、`p3_reference_cast` 5/5 通过；后者启用 `RecoveryEvidenceRequest::all()`，检查 direct 的0/1/4、nested的0/1/4/7、nestedCall的0/3/6/9和pop变体的1/4/5均有实际来源段。JDK ignored 对照1/1通过。
- 40项相邻回归通过，覆盖旧局部值、数组、参数转换、特殊调用正反例和失败前缀保留，另1项JDK用例按默认忽略。先前 meeting 6/6、new-value 4/4也已通过。
- R9旧字段拒绝用例改成 `checkcast; pop; aconst_null; areturn` 的合法内存变体，继续检验丢弃消费者的字段效果保留。三个变体的临时 class 由 `java -Xverify:all` 接受，主代理再次确认 null/NPE/null；证据见 `../../evidence/java-syntax-2026-09-22/casts/r9-refusal/`。
- 独立 CastAudit 的完整 CLI 整类输出直接重编译，23行运行结果与原 class 完全相同，包括 null、错误类型、数组、嵌套检查、静态/实例字段、局部和检查失败阻止后续调用。jadx 在无用局部场景删除了原程序的可失败检查，3行结果不同；不以 jadx 为正确性预言机。证据见 `../../evidence/java-syntax-2026-09-22/casts/independent/`。

初版独立 CastAudit 带静态字段初始化，暴露 `<clinit>` 中 `return;` 的独立非法源码问题。保留失败输入及日志于 `with-clinit/`，随后修改**原始测试输入**为由 driver 设置字段，再重新编译原 class、恢复和对照；没有删改恢复正文。该初始化问题单独分析，不混入本 change。

受影响文件 rustfmt 通过。最终全仓 fmt 检查尚被下一轮正在准备的 invocation/numeric 测试格式挡住，不能在此宣称全仓通过。`cargo clippy --locked -p jarde-java --all-targets -- -D warnings` 仍仅被既存 `region.rs:1736` 的 type_complexity 拦截，未增加 allow。命令原始日志已保存在 casts/root-*.log。

本 fixture 加入后曾集中验过 census `(86,431,74,197,8)` 及 fingerprint；随后 numeric/invocation 新 fixture 又加入，当前全仓计数需在它们冻结后集中重验。任务3.2因此保留未完成，不用过期 census 或受阻 strict 门禁声称全部通过。cast 的生产行为与执行语义已获独立验收。

补充：后续两组fixture已集中冻结，当前reader census `(88,480,74,227,8)` 重跑通过，fingerprint 205项（较197新增8、修改0、删除0）及5项验证通过。该计数维护已完成；strict clippy既存问题仍保留，不改变cast独立行为验收结论。
