# 任务 2.1/2.2 证据：实现、落点实验、锚/负例、corpus 差分（coder，2026-10-06，v2 重设计）

## 落点与实现（file: `crates/jarde-java/src/init.rs`、`crates/jarde-java/src/build.rs`、`crates/jarde-java/src/lambda.rs`）

**所有权走 `init::Sites` 通道**（`owned` 集合，走查经 `self.sites.owns(at)` 消费；无平行所有权机制）：

1. `init.rs`：`discarded_null_check_tail` 的三指令纪律提取为共享的
   `discarded_null_check_window`（`dup; <is_discarded_null_check 拼写>; pop` + 两条单用链；闭包
   `reads_guarded` 的答案回传），构造站点与绑定接收者两处调用**同一**函数——判定逻辑不复制。
2. `init.rs::receiver_tails`：BCI 序跨块的 `[dup, check, pop, indy]` 窗口（同块窗口实测不成立，E7），
   `dup` 的读经 `proved_receiver_class` 证明（`build.rs::allocation_behind_value` 的 move 链：
   `Load`/`Store`/`Duplicate` + `<init>` 接收者 → `Operation::Allocate`；`<init>` 臂是本次实证新增，
   锚的链是 `Load@9 → Store@7 → <init>@4 → Duplicate@3 → Allocate@0`），且该局部槽在读取 BCI 之后
   **无 store**。通过者把尾三 BCI 记入 `Sites.owned` 并把 `ReceiverTail` 存进 `Sites`。
3. `build.rs::lambda_expr`：站点经 `self.sites.receiver_tail_at(bci)` 取得尾，**经尾读回**捕获值
   （`captured_value`：`copy_value → receiver`）供 `unreplayable`/`render_value`；站点自身读到的
   值（`operands`）仍是记录证据（`captures.0.bci` 保持 10）。`receiver_nonnull = tail.is_some()` 传给 plan。
4. `lambda.rs::plan`：新增参数 `receiver_nonnull`，拒绝分支加合取 `&& !receiver_nonnull`；判据块其余
   部分、拒绝文本、拒绝码、`VALUE_LEVEL_REFUSALS` 六族表、`unreplayable`/`render_value` 逐字未动。

## 落点单独翻转锚（最小门控实验）

`results/02-locus-experiment.patch` 的临时门（`JARDE_EXPERIMENT_NO_TAIL_CLAIM`，已还原）只关掉
`init::sites` 里的尾声明，escape 与读回**保持原样**：

| 二进制状态 | `OP.sideEffect` 的 ⑤ 诊断 | 渲染 |
| --- | --- | --- |
| 尾声明开启（生产） | 0（两腿） | `02-locus-experiment-claim-on{,-javac8}.txt` |
| 尾声明关闭（其余不变） | 1（两腿） | `02-locus-experiment-claim-off{,-javac8}.txt` |

**关闭时与 HEAD 基线逐字节相同**（`diff` 实测），开启时锚恢复 → 该落点单独翻转锚。

## 窄口径的第四道必要门（corpus 差分暴露，实测）

corpus 差分（`02-corpus-sweep.sh`）在**未加**该门时暴露 `C1.chain`
（`realistic-class-combination-patrol`）：其绑定接收者 `out` 是 `List<String> out = new ArrayList<>()`
——分配类 `ArrayList` 而**站点描述符**写 `List`（声明类型），plan 的三路捕获检查必然拒绝该站点；尾声明
却把该站点自己的空检查从"引注"里拿掉，方法从**整体引注**变成**部分呈现**，剥离后 `chain` 仍可编译且
丢掉 `forEach(out::add)` 的副作用（实测 diff 见下）。故尾声明加一道**必要门**（是 plan 三路检查中
frame/site 那一半的必要条件，不是平行判定）：

> 站点自身描述符的第一个参数（捕获值的位置）必须**逐字命名分配类**（`captured_reference` 经
> `jarde_reader::classfile::descriptor_facts` 读，与 `build::array_descriptor` 同一读取）。

加门后 `C1` 渲染与 HEAD **逐字节相同**（`diff` 实测），单元测试把该形钉住
（`the_bound_receiver_tail_is_claimed_exactly_where_the_receiver_is_proved` 的第三段）。

## 锚 / 负例 / 零回退

- 锚：`OP.sideEffect` 两腿 0 引注，文本 = `results/02-anchor.txt`（钉在
  `tests/recover_proved_nonnull_bound_receivers.rs::OP_SIDE_EFFECT`）；剥离→编译→`-Xverify:all`
  运行两腿输出 `hi/none/none/3/0/42/false/S/`（ignored 回放测试，实测 1 passed）。
- 负例：`BRN` 三形（可参数字段读、可空字段读、捕获后重写）两腿整类文本与 HEAD **逐字节相同**
  （`diff` 实测），测试钉住三段成员文本 + 类级 3 条拒绝。
- 零回退：`tests/class_source.rs`（typed-functional `this::length`/`this::label`）、
  `tests/p3_lambda_adaptation.rs`（bound-null 拒绝锚）全绿；`-p jarde-java` 库测试 288/288。

## corpus 差分（`02-corpus-sweep.sh`，自检先行）

- 自检：已知正例 `OP` 两腿 1 → 0 拒绝；已知负例 `BRN` 3 → 3 且逐字节不动；每条渲染先断言
  jarde 自述头（`// jarde: presentation of`）——无自述头即退出，不产生假零。
- 语料：`openspec/evidence` + `tests/fixtures` 的**全部** `.class`（2726 个松散文件）+ 全部 jar 的
  `.class` 条目（738 个）= 3464；**13 个**渲染不出（12 个是被有意损坏/字节补丁的负例 fixture——
  `javap` 都读不出完整类名；2 个 `package-info`），脚本逐个列出、不静默丢弃。
- 结果：**moved = 5**，全部是解锁形且全部 `refusals 1 → 0`：
  `M1.class`（method-reference-patrol，`local2::inst`）、`OP.class`（本片两腿）、`op.jar!OP.class`、
  `fam.jar!M1.class`。**C1 不在其中**（与 HEAD 逐字节相同）。
- 全语料 ⑤ 拒绝句总数 12 → 7（−5 = 上述五处各一条）。

## 残余边界（如实登记，供 root 裁决）

尾声明要求"分配类 == 站点描述符的捕获类型"，但 plan 的三路检查还有**实现句柄 owner** 那一半：
javac 把继承成员解析到**声明类**（实测探针 `out::equals`：frame/site `ArrayList`、impl `AbstractList`）。
该形（语料内**零**实例）下尾仍被声明、站点仍被 plan 拒绝 → 方法部分呈现（与 HEAD 同为部分呈现，仅少两条
引注；剥离后不编译）。关掉它需要在声明时知道实现句柄（bootstrap 表），超出本片已裁定的机制；登记为后续项。

## 复现

```sh
# 锚与负例（双腿 fixture 见 results/fixtures/README.md）
target/debug/jarde-cli class-source --policy single-class \
  --input results/fixtures/v8/OP.class --class OP --format text
# 语料差分（需 HEAD 基线二进制；脚本头注释给出构建方式）
sh openspec/changes/recover-proved-nonnull-bound-receivers/results/02-corpus-sweep.sh
```
