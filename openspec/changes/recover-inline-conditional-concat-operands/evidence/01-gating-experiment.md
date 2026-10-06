# Task 1.1 门控实验：`jre_concat_split` 的发出处、链所有权判据与子证明接入

日期：2026-10-07（子代理 worktree，未 push）。基线：本 worktree `HEAD` = `d1f0aa39`，源码零改动；
基线二进制由 `git archive HEAD | tar -x -C /tmp/icco/base-src` + `cargo build -p jarde-cli
--locked --target-dir /tmp/icco/base-target` 生成。

## 结论（先给判断）

1. **发出处与判据已定位**：`crates/jarde-java/src/concat.rs` 的 `verify`，在块内 walk **之前**的
   split 检查：它从候选 head 的 allocation/dup/ctor 三元组出发，找第一个 `bci > head`、在**另一块**、
   且接收者经 append 返回值/局部/汇合回溯到本链 allocation 的同 owner `toString`（`consumes_the_instance`
   → `reaches_the_allocation`，`RECEIVER_TRACE_BOUND = 16`），命中即拒绝整链：
   `"the concatenation that starts at BCI {head} ends in the `toString` at BCI {tail}, which is in another
   block: a branch cuts the chain in two, and the value it builds is not the value of one expression"`
   （`crates/jarde-java/src/concat.rs:882`，本片逐字未动）。
2. **HEAD 逐字复核**：`jarde-cli class-source --input <ni.jar> --class NI --format json --evidence
   rule_details` 给出 `{"code":"jre_concat_split","head":11}`，文本与冻结证据
   `../recover-committed-local-multireads/evidence/renders/ni.txt` 的 sha256 **完全相同**
   （`4b78780a…b534`）。
3. **只接"终端接受"不翻转 NI**（实验 A）：把 split 拒绝整个跳过，NI 渲染与基线**逐字节相同**——
   walk 随即在 BCI 28 的 `getfield` 上以 `jre_concat_interleaved_effect` 拒链，回落路径仍因
   deferred-binding 拒绝而整方法引用。
4. **链端到端拥有是必要条件、仍不充分**（实验 B）：把整段 span 强制 owned（模拟"链跨块拥有"）后，
   全部链上值的拒绝消失，唯一剩余阻塞是 **BCI 8 的 `getstatic System.out`**：
   `"the saved producer at BCI 8 has 3 consumers, so one local binding cannot prove its execution count"`
   ——该值在 cut 之前产生、在 join 块 BCI 97 的 `println` 处消费，穿越 join 时被块 91 的平凡 Stack(0)
   Phi 记了两条操作数记录（1 次真读 + 2 条记录 = 3）。
5. **同一子证明同时许可第二条件**：cut 的中间块"恰好是两臂常量物化、无其它副作用"，正是
   "该值的求值点与消费者表达式位置之间只隔一次无副作用物化"的依据。接上这一条（实验 C/D）后
   NI 翻转，呈现为源码形态 `"" + … + (a == b)`；真跨块副作用链**不翻**（见 1.2 负例）。
6. **子证明可只读复用**：`recover-conditional-values` 的两臂证明
   `crate::build::prove_conditional_value(&Region, &CanonicalCfg, &SsaTable, &Operations, &mut Budget)
   -> ConditionalValueAttempt` 是 `pub(crate)`，且**不使用** `Region::If` 的 `prefix`。本片据此在
   `concat.rs` 内按 CFG 事实陈述 `Region::If`（branch / 落空臂 / 取中臂 / join）交给该证明，由它
   **重新**校验每条边、臂入口、Phi 输入与消费者——没有重实现两臂证明。证书只补该证明不陈述的部分：
   两臂块内**只有**所推常量与（必要时）到 join 的 transfer、常量为 `0`/`1`、Phi 的唯一消费者是本链的
   `append(Z)`。
   → **STOP 条件不成立**（"would need re-implementing the two-arm proof" 不适用），继续实施。

## 1. 门的位置与它能看到的事实

- `plan()`（`concat.rs:351`）逐块逐指令找 `Operation::Allocate` 且类名在 `CONCAT_CLASSES`
  （`java/lang/StringBuilder` / `java/lang/StringBuffer`）的候选，调 `verify`；
- `verify` 的 split 检查在 walk 之前（`concat.rs:867–888`），其事实来源是 `all`（全部 SSA 指令）、
  `block_of`（BCI→块）、`consumes_the_instance`（接收者值流回溯）；
- walk 是准入权威（`concat.rs:946–1108`）：块内连续游走，`append`/`toString` 之外的指令只有
  `Push | Load | Arithmetic | Negate` 与"值被读到的 invoke"可作操作数生产者；`Operation::Field`
  落入 `StatementFree` 拒绝（`jre_concat_interleaved_effect`）。**这就是 NMA/NI2 呈现为原始
  builder 链而不是 `+` 的原因**：它们的链内含 `getfield`，链不被 owned，回落路径把整链渲染成
  `(String) new StringBuilder()…toString()` 表达式（本片硬不变量 1 要求其逐字节不变）。

## 2. 门控实验矩阵（全部实测，非推断）

| 实验 | 做法 | NI | 真跨块副作用负例（ICN） | 判定 |
|---|---|---|---|---|
| 基线 | HEAD | 拒（`jre_concat_split` BCI 11→94），整方法引用 | 拒（逐字） | 基线 |
| A | 只跳过 split 拒绝 | **不翻**（逐字节同基线；walk 改判 BCI 28 `jre_concat_interleaved_effect`） | 不翻 | 终端接受不充分 |
| B | A + 整段 span 强制 owned（模拟链拥有） | **不翻**；唯一剩余 `the saved producer at BCI 8 has 3 consumers` | 不翻 | 链拥有不充分 |
| C | B + cut 许可的 deferred-binding 计数/声明区条件 | **翻**（`+` 形态） | 不翻 | 充分 |
| D | 本片实现（证书 + cut 许可条件 + 布尔呈现证据） | **翻**（`+` 形态，见 `renders/`） | 不翻（逐字节） | 采纳 |

实验 A/B 的 verbatim 尾巴（`--format text`）：

```
A: // the saved producer at BCI 8 has 3 consumers, so one local binding cannot prove its execution count
   // @bytecode 97 8 28 63 23 39 55
   // the value at BCI 97 was produced by a saved declaration this run could not commit
B: （同上；其余 30 余条链上值的拒绝消失，只剩这一条）
C/D: NI local1 = new NI();
     java.lang.System.out.println("" + local1.make(3).v + "/" + local1.make(2).outerTag() + "/" +
        externalMake(local1, 1).outerRef().tag + "/" + (externalMake(local1, 1).outerRef() == local1));
```

## 3. 第二条件的判据（本片实现，`build.rs::Builder::crosses_a_proved_cut`）

一个值穿越已证 cut，当且仅当三条都成立（全部读自本 run 的事实）：

1. 生产者站在 cut 的 head 块内，且该块内 producer 与 branch 之间的**每条**指令都被该链 owned
   （链自己的文本，不是独立效果）；
2. 该值至少有一次真指令读（join 留下的 Phi 记录不是读：无论哪条路，值只求值一次）；
3. 每一条真读都落在 cut 的 join 块内——join 块内 producer→reader 的区间仍由既有的
   `has_independent_boundary` 判定，与任何同块值一视同仁。

据此：`binding_consumers` 对穿越值按真读计数（计数门与 `terminal_consumer` 共用），
`shares_a_declaration_region` 对穿越值接受跨块声明区（声明写在 cut 之前、读者在 join 之后，
两者之间只有一次常量物化，没有块也没有 handler 能结束声明的作用域）。

## 4. 复现步骤

1. 基线二进制：`git archive HEAD | tar -x -C /tmp/icco/base-src && (cd /tmp/icco/base-src && cargo build
   -p jarde-cli --locked --target-dir /tmp/icco/base-target)`；
2. 探针/负例双腿编译（`javac --release 8 -g -nowarn -d v8` 与 Corretto 1.8.0_432 的 javac
   `-g -nowarn -d v8-javac8`），各自 `zip` 成 jar；
3. `jarde-cli class-source --input <jar> --class <C> --format text|json --evidence rule_details`，
   两个二进制各跑一遍并 `diff`；
4. 计数前先断言渲染自报头（`// jarde: presentation of \`<class>\``）：无自报头即失败。

## 5. 未验证 / 边界

- 本记录只覆盖任务 1.1/1.2 的门控与负例；全门禁、corpus 指纹与 oracle 腿见 `03-corpus-delta.md`
  与 `04-gates.md`。
- 实验 B 的"强制 owned"是**临时插桩**（env 门控，已回滚，工作区无残留）；它的作用是定位第二阻塞，
  不是候选实现。
