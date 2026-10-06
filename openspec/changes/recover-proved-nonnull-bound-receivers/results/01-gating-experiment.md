# 任务 1.1 / 1.2 门控实验与停手报告（coder，2026-10-06）

**结论先行**：本片 spec 钉的落点（`lambda.rs` 的 `reference_shape ∧ parameter_adaptation ∧ Reach::Receiver ∧
captures.len() == 1` 拒绝分支）**确为锚 ⑤ 诊断（`jre_lambda_sam_types` / "adapting this bound receiver would
move its null failure…"）的产生点**，双腿 `parameter_adaptation == true` 已实测；**但门控实验证伪了
"仅在该点加两门逃逸即可恢复锚"这一后半前提**——锚的**捕获值 SSA 定义不是 `Operation::Allocate`，而是 javac 为
绑定接收者写的丢弃空检查尾 `dup; Objects.requireNonNull|Object.getClass; pop` 的 `dup`**（模型自身记录
`captures.0.bci = 10`，见 `00-lambda-site-record-*.txt`），该 `dup` 不属于任何已证形状，触发**复制族值级拒绝**
→ 方法体整体引注。按 handoff「spec 未枚举的门 → 停手 + 提出方案 + 请示，不自行放宽共享判据」纪律，
**本片未落任何生产改动**；生产树与 HEAD 逐字节相同（`git diff` 空）。本文给出完整实验矩阵、**实测充分的最小机制**
与待 root 裁决的设计问题。

## 1. 前提验证（成立的部分）

- fixture：`results/fixtures/`——正例 `OP.java` 逐字节复制自 Optional 巡查
  （`openspec/evidence/java-syntax-2026-10-05/optional-chain-patrol/fixture/OP.java`，sha256
  `d6aca17f…`），负例 `BRN.java`（三形：可空参数读、可空字段读、捕获后重写）本次新写。
- 双腿：`javac --release 8 -Xlint:-options -d v8`（javac 23.0.1）与真 javac 8
  （`/Library/Java/JavaVirtualMachines/corretto-1.8.0_432`，`-d v8-javac8`）。
  **`v8/OP.class` 与巡查 jar 内的 `OP.class` `cmp` 逐字节相同**（巡查 jar 即同一条 `--release 8`
  命令的产物）；双腿 `sideEffect` 字节码 BCI 0–27 **完全同构**，唯一差别是空检查拼写
  （javac 23：`invokestatic Objects.requireNonNull`@11；javac 8：`invokevirtual Object.getClass`@11）。
- HEAD 干净构建（`cargo build -p jarde-cli --locked`）基线渲染：
  `00-baseline-{v8,v8-javac8}-{OP,BRN}.txt`。OP 锚 **1 条** 逐字拒绝（两腿文本逐字相同）；BRN **3 条**
  逐字拒绝（两腿整类文本逐字相同）。
- 模型自身记录（HEAD，`--evidence all`）：`00-lambda-site-record-{v8,v8-javac8}.txt`——
  `lambdas.0.use_site = 15`、`form = null`、`refusal.code = "jre_lambda_sam_types"`、拒绝文本逐字、
  **`captures.0.bci = 10`**、`news.0 = {head 0, dup 3, constructor 4, presented true}`。
- 门控实验 `JARDE_EXPERIMENT_BOUND_RECEIVER=report`（把该分支四个合取写进诊断文本渲染）：
  双腿四条位点（OP 锚 + BRN 三负例）全部输出
  `reference_shape=true parameter_adaptation=true reach=Receiver captures=1 typed_reference=false`
  （`01-experiment-report-*`，两腿逐字相同）→ **落点与 `parameter_adaptation` 前提成立**。

## 2. 门控实验：放宽该分支**不足以**恢复锚（证伪后半前提）

实验补丁见 `01-experiment.patch`（4 个环境变量，全部为**临时插桩**，已 `git checkout` 还原；
`JARDE_EXPERIMENT_KEEP_STATEMENTS` 仅用于诊断时保留语句，看清剩余拒绝）。矩阵（每条均为 v8 腿 +
v8-javac8 腿，两腿结论相同）：

| 实验 | 放宽集合 | 锚 `sideEffect` 的观察结果 | 文件 |
| --- | --- | --- | --- |
| E2 | 仅放宽拒绝分支 | ⑤ 诊断消失，但换成**捕获不可复现**拒绝："the value captured for the site's argument 0 (BCI 10) cannot be written where the shape reads it: it comes from an Duplicate at BCI 10…"；**方法体仍整体引注（无语句）** | `01-experiment-relax-*` |
| E3 | + `unreplayable` 跟随 `Duplicate` | 换成**捕获无表达式**拒绝："…produces no expression this subset writes"（`render_value` 的 Duplicate 臂："the copy at BCI 10 has no proved local assignment"）；体仍引注 | `01-experiment-relax2-*` |
| E4 | + 捕获经 `dup` 读回（`lambda_expr` 的 operands） | 捕获两拒绝消失；体仍引注，引注理由是**复制族值级拒绝**（`VALUE_LEVEL_REFUSALS` 第 1 族），BCI 覆盖全方法 | `01-experiment-relax3-*` |
| E5 | + 该尾三指令（`dup`/check/`pop`）记为站点所有（**同块窗口版**，失败） | 同上（`dup` 仍无主） | `01-experiment-relax4-*` |
| E6 | 同 E5 + 临时关闭值级逃逸守卫（仅诊断用） | **语句首次可见**：声明 + `arg0.ifPresent((Consumer) ((Object p0) -> local1.append((String) p0)));` + `return local1.toString();`，残留一条引注 `@bytecode 11 14 9 / the copy at BCI 10 has no proved local assignment` | `01-experiment-relax5-*` |
| E7 | 同块窗口（`instructions[index..index+4]`） | 同 E6（**同块窗口在真实字节码上不成立**：canonical CFG 在可能抛出的 check 处断块） | `01-experiment-relax6-*` |
| **E8** | 窗口改按 **BCI 序**跨块取 `[dup, check, pop, site]` | **锚完全恢复**（见下），守卫开启（E8b）与关闭（E8a/E7-变体）结果相同 | `01-experiment-relax7-*`、`01-experiment-e8a/e8b-*` |

**E8b 实测渲染（双腿 `sideEffect` 逐字相同，0 条 ⑤ 诊断，0 条 gap）**：

```java
    static java.lang.String sideEffect(java.util.Optional arg0) {
        // @method sideEffect(Ljava/util/Optional;)Ljava/lang/String;
        // @declaration a static method of `OP`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1 = new java.lang.StringBuilder();
        arg0.ifPresent((java.util.function.Consumer) ((java.lang.Object p0) -> local1.append((java.lang.String) p0)));
        return local1.toString();
    }
```

**行为牙**：`01-experiment-replay.sh`（剥离 `//` 行 → 装机 `javac --release 8` 与真 javac 8 各编译 →
`java -Xverify:all` 运行 → 与 fixture 自身 class 对照；脚本先自检 jarde 自述头与类文本）。实测 6 行输出：
两腿原 class 与两条编译腿全部 `hi/none/none/3/0/42/false/S/`，**逐字一致**。

**E8c（反证：门是承重的）**：E8 原型把拒绝分支**无条件**放宽后，三条负例也会被呈现（例如
`nullableParameter` 渲染成 `arg0.ifPresent((Consumer) ((Object p0) -> arg1.append((String) p0)));`，
即"值级捕获改为槽级捕获"的 NPE 时机移动）——`01-experiment-e8c-v8-BRN.txt`。故 spec 的两道门不是装饰，
而是负例守恒的唯一依据。

## 3. 实测充分的最小机制（供 root 重新设计落点）

由 E8a 与 E8b 的差集可知**哪些是必需、哪些不必要**：

1. **`lambda.rs` 同 verdict 点两门逃逸**（spec 原钉，必需）：捕获接收者的（尾前）值 SSA 单定义链只经
   move（`Load`/`Store`/`Duplicate`，即 `array_of_value`/`written_type` 同族走法）终于 `Operation::Allocate`；
   且捕获 BCI 之后该局部槽无 store。任一不过 → 现有拒绝逐字保留。
2. **`build.rs::lambda_expr`：站点自有丢弃空检查尾的识别与"经尾读回"**（spec 未枚举，必需）：
   站点捕获操作数若由其前方**紧邻的** `[dup, check, pop]` 尾的 `dup` 定义，则该尾即 javac 为**本站点**写的
   创建期空检查；读回 `dup` 所复制的前尾值（正是源级接收者），使 `unreplayable`/`render_value`/source-map
   全部看到该 load（于是**不需要**改 `unreplayable`——E8a 已实测无该改动亦干净）。
   尾的证明纪律可镜像 `init.rs::discarded_null_check_tail`（仅位置改为站点前）：`dup` 的单一栈读 = 站点
   描述符命名的接收者值；check 命中 `facts::is_discarded_null_check` 两种拼写且**单次**读 `dup` 的写；
   `pop` 为 `0x57` 且**单次**读 check 的结果。**实测要点：窗口不能按"同一 canonical block"取**
   （check 会抛 → 断块，E7 失败、E8 成立），须按 BCI 序跨块取相邻四条。
3. **同尾三 BCI 记入"站点所有"集合**（spec 未枚举，必需）：发射走查在 `Operation::Duplicate` 分支对
   `chained_pair` 之外一律引注 → 值级拒绝 → 体整体引注（E4/E5）。落点须在走查**到达尾的 BCI 之前**决定
   （尾在站点之前）——即需要一个 `prepare_*` 型前置遍或由站点计划发布 owned 集合（见第 4 节）。
4. **不需要**的改动（实测排除）：`unreplayable` 的 `Duplicate` 臂、`render_value` 的 `Duplicate` 臂、
   `VALUE_LEVEL_REFUSALS` 六族表、拒绝文本与拒绝码——保持逐字不动即可（E8a）。

## 4. 待 root 裁决的设计问题（本片停手原因）

1. **尾证明的所有权落点**：走查在站点前就已走过尾的三条指令，故 owned 必须在站点渲染前可知。
   选项：(a) 新增 `prepare_lambda_tails` 型前置遍（与既有 `prepare_conditional_regions`/
   `prepare_deferred_bindings` 同族）——但需注意 `lambda::plan` 会被二次调用带来的预算计费重复；
   (b) 站点计划自身发布 owned 集合（走查延后判定/记账）；(c) 走查内对"站点前的丢弃空检查尾"就地证明并
   登记（证明用 SSA+operations，与站点计划无关，可行性已由 E8 原型证实）。
2. **范围**：本机制天然同时解开**所有**"绑定接收者 = 捕获局部/参数/字段读"的方法引用（含
   **无需参数适配**、今天走 `LambdaForm::MethodReference` 却在渲染期被同一尾挡住的形——
   实测 `arg0::trimToSize` 在 HEAD 即被同一引注挡住）。spec 现文写"既有锚渲染逐字节不变"，
   故须先裁决：**窄口径**（仅本片逃逸通过的形配尾所有权）还是**宽口径**（全绑定接收者族一并恢复，
   那是一个更大的片，且要重估 `recover-typed-functional-method-references` 的锚）。
   窄口径与"Everything else stays byte-identical"相容；E8c 已给出无门时的反例。
3. **守卫交互**：解开复制族引注后，`ff3cf21b`（值级拒绝 → 体整体引注）与 7503 行的
   `local_assignments`/ragged 逃逸不再对该形状触发；须确认这不会给**其它**形状留下"可编译且行为不同"的
   漏口（建议随 redesign 一并做 corpus 渲染差分，脚本可复用
   `openspec/changes/recover-temporal-argument-widening/results/corpus-sweep.sh` 的骨架）。

## 5. 本片未做的事（边界）

- **零生产改动**：`crates/jarde-java/src/{lambda.rs,build.rs}` 与 HEAD 逐字节相同（实验补丁仅存于
  `01-experiment.patch`，已还原）；未新增/修改任何测试 fixture 于 `tests/`（避免"未被 CI 引用的冻结
  fixture"违反 handoff 纪律）；未跑全门禁与 corpus 指纹（无代码变更，HEAD 门禁即现状）。
- tasks.md：1.1/1.2 已勾（依据本文与 `results/`），2.1/2.2/3.1 保持未勾并注明阻塞原因。
- 复现：`results/fixtures/` 下 `.java` 两份 + 双腿 `.class`（sha256 见 `fixtures/README.md`），
  渲染命令 `jarde-cli class-source --policy single-class --input <class> --class <Name> --format text`。
