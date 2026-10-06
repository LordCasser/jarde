# 任务 1.1/1.2/2.1/2.2 证据：插桩定夺、实现、锚/对照/负例、corpus 差分（coder，2026-10-06）

实现落点：`crates/jarde-java/src/region.rs`（新增 `Walker::ladder_join` + `Walker::arm_forward_routes`
+ `ArmRoutes`/`LADDER_MAX_BRANCHES`，二分支 join 选举处新增一路读数，会合判据新增一条由该读数证明的
豁免）。`overlapping_owner` 检查本身**逐字未动**（树变非重叠，不是检查被放宽）。

## 1.2 插桩：BCI 56 的两个主张者（先于任何实现）

临时插桩 `JRE_LADDER_PROBE`（dump 完整 region 树 + 重复 owner；已还原，见
[`01-instrumentation-before.txt`](01-instrumentation-before.txt) /
[`01-instrumentation-after.txt`](01-instrumentation-after.txt)）：

```text
CB.loopElseIfRet（HEAD 5a9f32bf）：
[1] Loop header=7 tests=[(7, 9)] exit=Some(59) owns=[7,12,31,56,39,45,56,53]
  [1] If prefix=[] branch=12 join=None owns=[12,31,56,39,45,56,53]
    [1] Straight owns=[31,56]                 <- 第一臂走穿 56，把它收进自己的 Straight
    [1] If prefix=[] branch=39 join=None owns=[39,45,56,53]
      [1] Sequence
        [1] Straight owns=[45]
        [1] Fallback owns=[56]                <- 第二臂再遇 56（已 visited）→ 引注
P3LADDER overlap bci=56 claimants=[1, 1]
```

**两个主张者是同一个 Loop 区域**（`claimants=[1,1]`）：56（循环 latch，`56: goto 7`）先被外层 If 的
**then 臂**（`31..36: lo = m + 1; goto 56`）走查时收进 `Straight[31,56]`，随后外层 If 的 **else 臂**里
内层 If 的 then 臂（`45..50: hi = m - 1; goto 56`）再次到达 56，`region_at_inner` 的 `!visited.insert`
分支把它作为 `FallbackReason::Loop` 引注。Loop 区域因此在自己的 `blocks()` 里两次列出 56 →
`jre_region_ownership_overlap` 整方法引用。

**判别变量（与已恢复的 `CB.loopIfElseRet` 差分）**：两者 `ipdom` 都是 `None`（早退臂离开方法 ⇒ 无
后支配点），`forward_join` 都不成立，故 `join=None`。差别在**两臂是否共享一个块**：

| 形 | 分支 | 臂的走查 | 共享块 |
| --- | --- | --- | --- |
| `loopElseIfRet`（拒） | `ipdom=None`，join=None | 两臂都走到 frame boundary=7（header） | **是**：两臂的继续路径都经过 56（latch 有 36/50 两个前驱 ⇒ 独立 canonical 块） |
| `loopIfElseRet`（恢复） | `ipdom=None`，join=None | then 臂是 return 叶；else 臂走到 boundary=2 | 否：latch（`21: goto 2`）单前驱，已与 else 臂的块融合，无第二主张者 |
| `loopElseIfNoRet`（恢复） | `ipdom=Some(44)`，join=Some(44) | 两臂都停在 44 | 否：join 已选举，body walk 事后单独主张 44 一次 |

即：**形状是 `join` 未被选举 + 两臂在 latch 汇合**；`overlapping_owner` 的报错是正确后果，不是根因。
（`CB.loopElseIfRet` 与 bsearch 的 `P3JOIN` 探针：`branch=12 ipdom=None then=Some(31) else=Some(39)
ft=false fe=false frame_boundary=Some(7)`；`loopElseIfNoRet` 为 `branch=10 ipdom=Some(44)`。）

## 2.1 实现：阶梯自己的 join 读数（单层阶梯）

新增读数（`join_node.is_none()` 时求值，紧接 `continue_target_join` 之后）：

- `Walker::ladder_join`：分支在循环体内（`frame.loop_targets.last()` 存在、frame boundary 就是该循环的
  header/continue target），两臂各自沿**严格前向**（同 body、BCI 递增）normal 边在**该循环 scope 内**走出
  自己的路径集（`Walker::arm_forward_routes`，每块每边计费；终点只能是终止块、frame boundary、或调用者
  指定的 stop 块；出 scope / 嵌套循环头 / 超过 `LADDER_MAX_BRANCHES` 一律拒绝）；
- 候选 = 两臂路径集交集中**唯一**的那个块（`unique_first_common`），且不得是 frame boundary、必须是该
  循环自己的块、必须过 `forward_join_predecessors`（入口全经本分支、至少一个非本分支前驱）、入边全为
  normal；
- 第二遍以候选为 stop 重走：两臂都必须**到达**候选，且都**不得**在未经候选的情况下到达 frame boundary
  ——这正是"if 之后的语句恰好是字节码会执行的路径"这一事实；
- `LADDER_MAX_BRANCHES = 1`：任一路径上最多一个分支（`else if` 这一步本身）——本片裁定的**单层**阶梯
  口径；双层阶梯的路径上有两个分支，读数不成立（负例保持拒绝）。

会合判据（`then_meets && else_meets` 那一处）新增豁免 `&& !ladder_elected`：该读数已经证明了判据要问的
事实（两臂要么经过 join、要么离开方法），所以不是放宽，而是把已证事实写进判据；其余判据、拒绝文本、
拒绝码逐字未动。

单臂形（`loopIfElseRet`）、无循环形（`noLoopElseIfRet`）、无早退形（`loopElseIfNoRet`）的判据零改动：
读数只在 `join_node.is_none()` 且**两臂都到达同一块**时成立，前两者交集为空、后者 join 已选举。

## 2.2 锚 / 对照 / 负例（双腿，javac 23.0.1 `--release 8` + 真 javac 8）

`results/02-corpus-sweep.out` 的自检 + `tests/recover_loop_else_if_early_returns.rs`：

| 形 | HEAD | 本片 | 证据 |
| --- | --- | --- | --- |
| `BS.bsearch`（全类） | 拒（BCI 56） | **恢复**（0 canonical/0 引注） | 方法文本逐字钉住 |
| `CB.loopElseIfRet` | 拒（BCI 56） | **恢复** | 同上 |
| `CB2.exitVal`（出口消费同形） | 拒（BCI 56） | **恢复** | 同上 |
| `CB.loopElseIfNoRet` | 恢复 | 恢复（**逐字节不动**） | 三条对照钉住 |
| `CB.loopIfElseRet` | 恢复 | 恢复（逐字节不动） | 同上 |
| `CB.noLoopElseIfRet` | 恢复 | 恢复（逐字节不动） | 同上 |
| `CB2.exitVal2` | 恢复 | 恢复（逐字节不动） | 钉住 |
| `LB.doubleLadder`（双层阶梯） | 拒（BCI 47） | 拒（BCI 47，逐字节不动） | 负例钉住 |
| `LB.tryLadder`（异常表跨越阶梯） | 拒（BCI 56） | 拒（BCI 56，逐字节不动） | 负例钉住 |
| `LB.switchInArm`（switch 混入阶梯臂） | 拒（cross-quote） | 拒（逐字节不动） | 负例钉住 |
| `LB.firstArmRet`（早退在第一臂） | 拒（cross-quote） | 拒（逐字节不动） | 边界钉住 |
| `LB.forLadder`（`for` 头 + 阶梯） | 拒（BCI 38） | **恢复** | 记录 + 钉住 |
| `LB.switchLadder`（switch 与阶梯同级） | 拒（BCI 75） | **恢复** | 记录 + 钉住 |
| `LR2.bsearch`（本片自带锚，全类可呈现） | 拒（BCI 45 形） | **恢复**（全类 0 引注） | 整类回放 |

**归档对照（实测）**：巡查归档渲染
[`jarde-CB.txt`](../../../../evidence/java-syntax-2026-10-05/binary-search-twopointer-patrol/results/jarde-CB.txt)
的四个成员与**当前 HEAD 基线**渲染逐字节相同（`loopElseIfRet` 的拒绝也在其中）——巡查记录与今天的
二进制一致，本片是第一个移动它的改动；`jarde-BS.txt` 除 `main` 外亦逐字节相同（`main` 的漂移发生在
本片之前：归档是部分呈现、基线是整方法引用，基线↔补丁逐字节相同，与本片无关）。

**剥离回放**（`cargo test --test recover_loop_else_if_early_returns -- --ignored`，实测 1 passed，
两腿两 javac 全绿）：`LR2` **整类**剥离→`javac --release 8`/真 javac 8 编译 exit 0→`-Xverify:all` 运行
输出 `2/-3` = 原 class 输出；`BS.bsearch`/`CB.loopElseIfRet`/`CB2.exitVal`/`LB.{switchLadder,forLadder}`
的方法文本 + 原 fixture 调用序列编成单元，输出分别等于原 class 的对应字段（`2/-3`、`1`、`-3`、`0/-1`）。
原 class 自身输出（实测钉死，两腿相同）：`BS` `2/-3/[1, 2, 3]/102334155`、`CB` `1/4/-1/50`、
`CB2` `-3/1`、`LB` `1/1/0/0/1/-1`、`LR2` `2/-3`。
（`BS`/`CB`/`CB2`/`LB` 的 `main` 被拒 ⇒ `void` 空体写保留符号 `jarde_refused_body()`，整类剥离**按设计**
不能编译；故整类回放落在本片自带的 `LR2` 上，锚的方法级回放落在原调用序列上——两处都实测。）

## corpus 差分（`02-corpus-sweep.sh`，自检先行，输出 `02-corpus-sweep.out`）

- 自检：已知正例 `CB` 1 → 0 overlap 拒绝；已知负例 `LB` 的 `BCI 47`/`BCI 56` 两条句在**两个二进制**上
  都在（overlap 4 → 2、cross-quote 2 → 2）；三条对照成员在两个二进制上逐字节相同；每条渲染先断言
  jarde 自述头。
- 语料：`openspec/evidence` + `tests/fixtures` 全部松散 `.class`（2736）+ 全部 jar 的 `.class` 条目
  （738）= 3474；13 个渲染不出（12 个有意损坏/字节补丁的负例 fixture、2 个 `package-info`；脚本逐个
  列出，与上一片记录的 13 一致）。
- 结果：**moved = 11**，全部是解锁形且全部 overlap 拒绝下降：巡查归档 `bs.jar!BS.class`（1 → 0）
  + 本片新 fixture 单元（`BS`/`CB`/`CB2`/`LR2` 各 1 → 0、`LB` 4 → 2，两腿共 10 个）。
  **语料其余任何类零移动**（含 `CB2` 之外的巡查类、`twoPtr` 等既有形）。
- 全语料 canonical-overlap 拒绝句总数 36 → 23（−13 = 上述 11 个类中 `BS/CB/CB2/LR2` 各 1、`LB` 各 2、
  `bs.jar!BS` 1）。

## 残余边界（如实登记，供 root 裁决）

1. **早退在第一臂**（`LB.firstArmRet`，即 `if(v==k){return m;} else if(v<k){…} else {…}`）：两臂**不**
   在 latch 汇合（返回臂是终止叶），本片读数不成立 ⇒ 保持拒绝（cross-quote，逐字节不动）。proposal 的
   MVP 措辞是"单层阶梯 + 单一早退臂"，此朝向属于该措辞的边界外沿；本片未覆盖，登记为后续。
2. **`switch` 与阶梯同级**（`LB.switchLadder`）恢复：读数只读阶梯自己的两臂路径，不检查同级语句；
   "switch 混合保持拒绝"在**switch 混入阶梯臂**（`LB.switchInArm`）上成立并已钉住。
3. **`for` 头 + 阶梯**（`LB.forLadder`）恢复：join 是 update 块，增量按 while 体末语句呈现（行为一致，
   回放实测）。proposal 未列举该形，按 spec 的 MVP 判据（单层/无异常表/无 switch）它在覆盖范围内。
4. 异常表跨越阶梯（`LB.tryLadder`）保持拒绝，但其拒绝是 `Try` 区域**内部**的同族 overlap（`Try` 自己
   两次列出 56），不是 try 专属判据；如实记录。

## 复现

```sh
# 锚/对照/负例（fixture 见 tests/fixtures/recover-loop-else-if-early-returns/README.md）
target/debug/jarde-cli class-source --policy single-class \
  --input tests/fixtures/recover-loop-else-if-early-returns/v8/CB.class --class CB --format text
cargo test --test recover_loop_else_if_early_returns --locked
cargo test --test recover_loop_else_if_early_returns --locked -- --ignored   # 需 JDK

# corpus 差分（需 parent commit 5a9f32bf 的基线二进制）
BASE=/path/to/baseline/target/debug/jarde-cli PATCHED=target/debug/jarde-cli \
  sh openspec/changes/recover-loop-else-if-early-returns/results/02-corpus-sweep.sh
```
