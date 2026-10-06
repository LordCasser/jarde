# Task 1.1 门控实验：计数门的发出处、输入分类与结论（STOP）

日期：2026-10-06（子代理 worktree，未 push）。基线：本 worktree `HEAD` = `2e17ac06`，源码零改动。

## 结论（先给判断）

1. **计数门的输入类本来就只有一类**：`prepare_deferred_bindings` 的自身前置条件要求"生产者指令必须把该值写进 `Slot::Stack(_)`"，因此"生产者已提交为局部声明"（其 store 写 `Slot::Local(_)`、已作为声明语句呈现）**根本到不了计数门**——不是分类缺失，而是该输入类不在门的输入集合里。对它"放宽"是空操作。
2. **NI 的两处计数拒绝都不是"已提交局部的多读"**：BCI 8 是 `getstatic System.out` 的值，BCI 71 是 `append` 的返回值——两者都是**栈携带值**，各自只有 **1 次真实指令读**；"3 consumers" = 1 次指令读 + **2 条被替换平凡 phi 的操作数记录**（汇合块 91 的 Stack(0)/Stack(1) phi 两个前驱都命名同一个值）。
3. **本片想要的可恢复性今天已经达成**：已提交局部在单表达式内被读 4 次按源码形态呈现（探针 `NMA`、以及 NI 去掉 `+` 链内分支的 `NI2`：`local1` 各出现 4 次，编译与运行一致）。
4. **NI 被拒的真实机制是 `concat@1` 的 `jre_concat_split`**：`""+…+(externalMake(n,1).outerRef() == n)` 里的 `==` 被编译成分支，`toString` 落在另一块，整条 `+` 链按既有设计被拒 → 链未被 `owns` → 回落 deferred-binding → 计数门只是这条回落链上的**级联症状**之一（还有 `no bounded final expression consumer`、`do not share a proven declaration region`）。
5. 唯一能把 NI 计数诊断归零的通道（把 phi 操作数记录排除出计数）**不翻转 NI**（改判 region 族），并且**改变 BI 的拒绝文本**——违反本片硬不变量 1/2 与验收的 "BI 拒绝逐字"。没有任何门处通道能"NI 翻、BI 不翻"。

因此按任务纪律（"If the gate cannot distinguish the classes with the facts it holds and a clean channel does not exist, STOP and report — do not approximate"）**停止实施**：2.1/2.2/3.1 未动，源码零改动；1.2 的锚/负例基线已冻结在 `renders/`。

## 1. 门的位置与它能看到的事实

- 发出处：`crates/jarde-java/src/build.rs`，`Builder::prepare_deferred_bindings`（函数锚点，非行号）：
  ```rust
  if value_facts.uses().len() != 1 {
      rejections.push(BindingRejection {
          value, anchor, producer,
          reason: format!(
              "the saved producer at BCI {anchor} has {} consumers, so one local binding cannot prove its execution count",
              value_facts.uses().len()
          ),
      });
      continue;
  }
  ```
- 同一函数内、计数门**之前**的前置条件（这就是分类边界）：
  ```rust
  let Some(instruction) = self.instructions.get(&producer).copied() else { continue; };
  if !instruction.writes().iter().any(|(slot, written)| matches!(slot, Slot::Stack(_)) && *written == value) {
      continue;
  };
  ```
  → 只有"生产者写 **Stack** 槽"的值进入计数门。写 `Local(n)` 的值（即"已提交局部声明"的 store）在这里被 `continue` 排除。
- 同族的另两道门（同一回落链，文本各自逐字）：`terminal_consumer` 同样以 `uses.len() != 1` 为界（`the saved producer at BCI N has no bounded final expression consumer`）；其后 `same_block` 判据给出 `the saved producer at BCI N and its final consumer at BCI M do not share a proven declaration region`。
- 分类事实不需要"穿入新通道"：**该输入类不达门**。局部值的多读呈现走的是 `render_value` 的局部名路径（`Declarations`/`reuse` 决定的名字），与 deferred-binding 计数门无关。

## 2. NI 的真实 SSA 事实（`dumps/ni-main-ssa.txt`）

`NI.main` 相关值（`javac --release 8` 腿；块 0 = 直线体，块 91 = `if_acmpne`(BCI 83) 的汇合块）：

| 值 | 定义 | 写槽 | uses（BCI, 块） | 计数门 |
|---|---|---|---|---|
| V9 | ctor @4（site head 0 dup 3）| `Stack(0)` | 7（`astore_1`，块 0） | 1 → 通过 |
| V10 | `astore_1` @7（**`n` 的值**）| `Local(1)` | 23/39/55/74/82（5 次 `aload_1`）+ `(None,91)`×2 | **不达门**（Local 写） |
| V11 | `getstatic System.out` @8 | `Stack(0)` | `(None,91)`×2 + 97（`println`） | **3 → 拒绝（BCI 8）** |
| V39 | `append` @71 | `Stack(1)` | `(None,91)`×2 + 91（`append(Z)`） | **3 → 拒绝（BCI 71）** |

块 91 的 `Phi{slot: Stack(0)}`/`Phi{slot: Stack(1)}` 两个操作数都命名同一个值（V11 / V39）→ 平凡 phi → 被替换（`replaced_by` 指向该值），两条操作数记录按 SSA 契约移到该值。**"3 consumers" = 1 次真实指令读 + 2 条 phi 操作数记录**。

同一现象在 `BI.earlyRet`（BCI 22）与 `CMP.main`（BCI 16/28）上逐字复现：都是"单次指令读 + 汇合块平凡 phi 的两条记录"。归档语料 42 个文件、114 条该诊断，**全部是 "has 3 consumers"**（无其它计数），与"1+2"结构一致。

## 3. 门控实验矩阵

| 通道 | 做法 | NI | BI | NJ | 判定 |
|---|---|---|---|---|---|
| A：只对"已提交局部声明"放宽 | 门处判别生产者写槽 | 不达门，**不翻** | 不达门，不翻 | 不翻 | 空操作 |
| B：把 phi 操作数记录排除出计数（`uses().iter().filter(\|u\| u.bci().is_some()).count()`，同改 `terminal_consumer`）| 唯一能把 NI 计数诊断归零的通道 | **仍拒**（改判 `do not share a proven declaration region`） | **拒绝文本改变**（→ `has no bounded final expression consumer`） | 逐字不变 | 违反零回退 |
| C：不加改动（基线） | — | 拒（计数 2 处） | 拒（计数 1 处） | 恢复 | 基线 |

通道 A 的空操作性质由 `dumps/ni-gate-facts.txt`、`dumps/bi-gate-facts.txt`、`dumps/nj-gate-facts.txt` 直接证明：到达门的每一条都写 `Stack(...)`（无一条写 `Local(...)`）。

通道 B 的实验补丁（逐字，已回滚，工作区无残留）：

```rust
// crates/jarde-java/src/build.rs，prepare_deferred_bindings
            if value_facts
                .uses()
                .iter()
                .filter(|use_| use_.bci().is_some())
                .count()
                != 1
            {
// crates/jarde-java/src/build.rs，terminal_consumer
        let uses = self.ssa.value(value).uses();
        let instruction_uses: Vec<u32> = uses.iter().filter_map(|use_| use_.bci()).collect();
        if instruction_uses.len() != 1 {
            return Ok(None);
        }
        let Some(reader) = Some(instruction_uses[0]) else { return Ok(None); };
```

通道 B 下的 verbatim 尾巴（`jarde-cli class-source`，2026-10-06）：

NI.main（计数诊断消失，方法仍拒）：
```
        // @bytecode 8
        // the saved producer at BCI 8 and its final consumer at BCI 97 do not share a proven declaration region
        // @bytecode 11 14 15
        // the saved producer at BCI 15 and its final consumer at BCI 97 do not share a proven declaration region
```
BI.earlyRet（**原为 `the saved producer at BCI 22 has 3 consumers, so one local binding cannot prove its execution count`**）：
```
        // @bytecode 22
        // the saved producer at BCI 22 has no bounded final expression consumer
```
NJ.main：与基线逐字相同（未贴）。

## 4. 真正阻断 NI 的机制

`jarde-cli class-source --input …/ni.jar --class NI --format json` 的 main 报告诊断（verbatim）：

```
jre_concat_split: the concatenation at BCI 11 was not presented: the concatenation that starts at BCI 11 ends in the `toString` at BCI 94, which is in another block: a branch cuts the chain in two, and the value it builds is not the value of one expression
jre_concat_chains: 1 concatenation candidate(s) read under concat@1: 0 presented, 1 refused
```

链被拒后，链上每个值都进入 deferred-binding 回落，逐值失败：计数门（V11/V39，平凡 phi 计数膨胀）→ `terminal_consumer` 在 void `println`(97) 处返回 `None`（`no bounded final expression consumer`）→ 即使过了前两道，生产者在块 0、消费者在块 91，`same_block` 判据拒绝。

**对照探针（`probes/`，双腿编译，见 §5）**：

- `NMA`（已提交局部读 4 次，`+` 链内无分支）→ **完整恢复**，`local1.add(1)`、`local1.add(2)`、`local1.self().tag`、`local1.tag`（`renders/NMA.txt`）。
- `NI2`（NI 原样，只把 `+` 链内的 `(externalMake(n,1).outerRef() == n)` 换成 `n.tag`）→ **完整恢复**，`local1` 出现 4 次：
  ```
  java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("").append(local1.make(3).v).append("/").append(local1.make(2).outerTag()).append("/").append(externalMake(local1, 1).outerRef().tag).append("/").append(local1.tag).toString());
  ```
  → 本片声明的可恢复性目标对"已提交局部"类**已经成立**；NI 与 NI2 的唯一差别是 `+` 链内那个分支。
- `NMB`（= `NMA` + `+` 链内一个 `(n.self() == n)`）→ 与 NI 逐字同形：BCI 8 计数拒绝 + 11 条 `no bounded final expression consumer`（`renders/NMB.txt`）。
- `CMP`（`"eq=" + (p == q) + " tag=" + p.hashCode()`，无多读局部）→ 同样两处计数拒绝（BCI 16/28）+ `no bounded final expression consumer`：**这是普遍形状，与"局部多读"无关**。
- `SC`（`for (int x : data())`，数组值栈携带多消费者）→ 完整恢复（缓存进 `local0`）。

## 5. 基线（可复核，双腿一致）

`renders/*.txt` 由 `jarde-cli class-source` 产出；脚本在计数前先断言渲染自报头（`// jarde: presentation of \`<class>\``），无自报头即失败。探针双腿（`javac 23.0.1 --release 8` 与 Corretto `1.8.0_432` javac 8）渲染 **sha256 完全相同**。

| anchor | 来源 | 计数诊断 | 结果 | sha256（render） |
|---|---|---|---|---|
| NI | `evidence/java-syntax-2026-10-05/multiconsumer-local-soundness-patrol/fixture/ni.jar` | 2 | 整方法拒（守卫） | `4b78780a421c43d17e20e16f227f4815a9d8799340b67fe7176ff2cf2babd534` |
| NJ | 同上 `nj.jar` | 0 | **恢复**（2 消费者对照） | `fe734acb11a52e0396ad8995f920a7f1d8fd6de1c398be3ece95452a761eda8a` |
| BI | `evidence/java-syntax-2026-10-05/boolean-loop-earlyret-patrol/fixture/bi.jar` | 1（BCI 22） | 拒 | `b712a20b337af01478fb584b8ae6185e88fb67fdf79c4cee309d95ba0e561596` |
| BS | `evidence/java-syntax-2026-10-05/binary-search-twopointer-patrol/fixture/bs.jar` | 0 | 拒（别族） | `41d0d227697bb6e991763467d6d6ae774211000103ad1c07c54fef3b57e403dd` |
| NMA | 探针 | 0 | **恢复**（`local1` ×4） | `7a0eb5ca4dbe42435178965e23e6d53b2b43e6168fccda60ce5cb4a184ee7705` |
| NMB | 探针 | 2 | 拒（NI 同形） | `7d073f4ccf891b81787847aabfc0ab65b100d12249847e709def85b546b09127` |
| NI2 | 探针 | 0 | **恢复**（`local1` ×4） | `b43ac537d6a6536ac68efcd333a25b574914c03490b93c7582763365a0abcfe4` |
| CMP | 探针 | 2 | 拒 | `a975a974eb29520b507fbedacc6e2501d2cc4a506b2d7326ec207869b4fef153` |
| SC | 探针 | 0 | **恢复** | `31e8d82985d94997375e711a319247d6195d03ce5f4a6e8bb61ada84d4dc2d07` |

## 6. 若要继续推进 NI，需要动什么（均超出本片声明范围）

- **P1 deferred-binding 回落**：①把 void invoke 认作语句位置的终止消费者；②跨块绑定证明（声明区覆盖消费者，替换 `same_block` 的保守判据）；③计数门对 phi 操作数记录的分类。三条都在 deferred-value 放置/顺序证明内部，触及 `preserve-deferred-value-order` 的判据（违反本片硬不变量 3"顺序判据零触碰"），且影响面远超"只改计数门输入分类"。
- **P2 `concat@1` 接受跨块 `toString`**：当 `+` 链内分支的值本身可呈现为表达式（`(a == b)`）时允许链跨块。proposal 明确排除"新建表达式通道"。
- 无论 P1/P2，BI（`boolean-loop-earlyret-patrol`，BCI 22）与 NI 在计数门上**同类**，需要单独的判据才能区分（例如"该值所在汇合块是否位于环内"），该判据目前不存在于门处可见的事实中。

## 7. 复现步骤

1. 仪器化（临时，已回滚）：在 `prepare_deferred_bindings` 的计数门之前、以及循环开始处各插一段 `std::env::var("JARDE_DEBUG_BINDINGS"/"JARDE_DEBUG_SSA")` 门控的 `eprintln!`，打印 `value`、`def`、`producer`、`anchor`、`site`、定义指令的 `writes`、每条 use 的 `(bci, block, 读到的槽)`；`cargo build -p jarde-cli --locked`。
2. `JARDE_DEBUG_BINDINGS=1 JARDE_DEBUG_SSA=1 ./target/debug/jarde-cli class-source --input <jar> --class <C>`，stderr 过滤 `DEBUG-GATE` / `DEBUG-SSA`（本目录 `dumps/` 即该输出）。
3. 探针双腿：`javac --release 8 -Xlint:-options -d v8 <C>.java` 与 `/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -d v8-javac8 <C>.java`，各自 `zip` 成 jar 后渲染（本目录 `renders/` 为 `--release 8` 腿；两腿 sha256 相同）。

## 8. 未验证 / 边界

- **未跑全门禁**：本次无源码改动，`cargo fmt`/clippy/workspace tests/corpus 指纹与 `HEAD` 基线同值，未重复测量（见 `tasks.md` 与交付说明）。
- **未冻结双足 jar fixture**：任务 1.2 的"双腿 fixture + README"属于实施产物；实施被阻断，仅冻结探针源码与双腿渲染（两腿一致）。
- 归档语料中 114 条计数诊断全为 "has 3 consumers"，是"1 次指令读 + 2 条平凡 phi 记录"的**一致性证据**，不是逐条重放证明。
