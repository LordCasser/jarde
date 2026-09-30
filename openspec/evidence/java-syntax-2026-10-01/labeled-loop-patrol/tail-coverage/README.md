# 标注循环尾段覆盖切片（`recover-labeled-loop-tail-coverage`）证据 — 2026-10-01

[巡查 README](../README.md) 两个缺口的实现侧证据：标注循环尾段的 discarded-call 语句被尾段走查接受（L2 零 `@bytecode` 残留、行为逐字一致），标签改为源码式拼写。基线 b6d14221；固定 fixture 与巡查矩阵见上级目录。

## 走查点定位（tasks 1.1，与 17a 启用点的关系）

设计文本猜测落点在 "region.rs 循环体/尾段核验"，**取证后修正**：region 层只决定形状（`region.rs` 模块文档自述 "does not emit text … the names and the statements all belong to `crate::build` and `crate::emit`"），标注循环尾段没有 `statement_free*` 检查点（该家族只在 guard.rs 的 synchronized/finally/TWR 有体语句上使用，17a 的启用点是其中 TWR 的那一处）。无体（非 guarded）路径上的实际语句核验点是 **`build.rs` 的 builder 逐指令走查**（`Builder::instruction` 的逐操作分派），其中 `pop` 的接受判据是 P3 2c.31 的呈现计划 `Builder::discarded_evaluations`：形状 (b) `invoke…; pop`（调用结果仅被本 `pop` 读取 → 调用语句即丢弃语句，`call_result_is_discarded`）与形状 (a) `produce; pop; invokestatic`（静态调用限定符）。它与 17a 的 `guard.rs::discarded_call_pop` 是同判据在两层各自的所有者（guarded 体证明 vs 无体呈现计划），本片**未复制判据、未动 guard.rs**。

**根因**：`discarded_evaluations` 原来在进入任何形状前要求 `pop` 在块内有 `next` 指令；而标注循环（含 `continue label`）的尾段块终止于尾 `pop`——外层循环的更新（`iinc; goto`）被已证明的 `for` 头拥有（`settled`），canonical block 在 `pop` 后断开（L2 的 BCI 66、T1 的 82、T2 的 94/116、T3 的 60 均如此，`javap` 逐例核对）。形状 (b) 只读 `pop` 之前的一条指令，`next` 是形状 (a) 独有的前置条件。**实现**：把 `let Some(next)` 守卫从形状检查之前移到形状 (a) 内——判据本身一字未改，泛化的只是启用面（块尾 `pop` 也进入形状 (b) 判定）。内层 while 的更新不被 `for` 头拥有时仍在体块内（T1 首版外层呈 `while` 形），`pop` 有 `next`，行为不变——这正是 17a 负例与 TWR 路径零变化的同一机制：TWR 体后邻 close 序列，`pop` 从不在块尾，TWR 测试家族（`p3_twr_discarded_call`、`p3_multi_resource_twr_geometry`、`p3_twr_saved_return_typing`、`p3_twr_return_tail` 等）与四个 N17a 负例前后零变化（整仓测试全绿佐证）。

## 变体族（tasks 1.2，javac 23.0.1 `--release 8 -g:none`，verifier 有效）

| 类 | 形状 | 实现前 | 实现后 |
| --- | --- | --- | --- |
| T1 `breakTail` | `continue outer` + `break outer` 双标签边 + 尾段链式 append（外层 `for` 头拥有更新，尾 `pop` 块尾 BCI 82） | 尾语句后 `@bytecode 82` 残留 | 零残留 |
| T2 `twoLabels` | 双层标注嵌套（外 `loop2`/内 `loop`）+ 两条尾段链式（BCI 94/116 均块尾） | 两处残留 | 零残留 |
| T3 `mixedTail` | 尾段混合 void 调用 + 链式调用（BCI 60 块尾） | 一处残留 | 零残留 |

原 class 运行输出（`java -Xverify:all`，`results/run/*.run`）：T1 `00,10,11,12,13,T1;`、T2 `00,01,02,W0;10,20,O0.00,01,00,01,`、T3 `<1>E1.<2>E2.`（源码 `T{1,2,3}.java`、冻结 class `original/`，SHA 见 [results/sha256.txt](results/sha256.txt)）。实现前后恢复文本在 `results/before/`、`results/after/`：diff 仅标签拼写与目标残留两行注释（T2 另有双标签）。固定 L1/L2/L3 复放：fixture SHA 与 `../results/fixture-sha256.txt` 一致；`fixture/*.jarde.java` 即实现前基线，`results/after/L*.jarde.after.java` diff——L1 仅两处标签拼写、L2 标签 + BCI 66 残留移除、L3 **零 diff**（无标注对照不变）。

## 标签拼写（design 决策 2）

`build.rs` 的自由函数 `loop_label(header_bci) = format!("jarde_loop_{header_bci}")` 换为 `Builder::loop_label`：按本方法语句写出序首次认领的标注循环取 `loop`，其后依次 `loop2`、`loop3`……（与命名表 `arg{n}`/`local{n}` 的序号发明惯例同构；标签在 Java 有独立命名空间，不与局部名查重）。每方法一张 `header → label` 表，随 `FinallyCheckpoint` 快照/回滚（推测性 finally 体写出失败时不留下已消耗的序号）。T2 钉死双标签：内层先被认领（其 `continue` 语句写出序在前）→ `loop`，外层 `loop2`。三个既有测试文件的 `jarde_loop_{4,5,8}` 期望同步为 `loop` 并在文件头注明（改名是本片有意行为）。

## 三方对照（tasks 3.2）

原冻结 class / 固定 JADX（`jadx-cli` dev，`--no-res`）/ Jarde `class-source` 三腿各自 `javac --release 8` 重编后 `java -Xverify:all` 逐路径（JADX 腿按其 `package defpackage;` 声明以全限定名运行；run 输出 `results/run/{jarde,jadx}/`，SHA 见 [results/sha256.txt](results/sha256.txt)）：

| 类·路径 | 原 class | Jarde | JADX | 备注 |
| --- | --- | --- | --- | --- |
| L1（两行） | `a3940f06…` | 同 | 同 | |
| L2（`00,10,`） | `3e87c820…` | 同 | 同 | 巡查冻结行为逐字保持 |
| L3 | `d5471484…` | 同 | 同 | 无标注对照 |
| T1 | `13f3d359…` | 同 | 同 | |
| T2 | `cdc63e41…` | 同 | `f5495d82…`（JADX 自身偏差） | 双层标注 `continue outer` 被 JADX 改写为内层 while 的 `break`：i≥1 的迭代落入外层 `O` 尾段，多出 `10,20,O1.`/`10,20,O2.` 行——原 class 为行为基准，Jarde 腿与原类逐行一致 |
| T3 | `90d88888…` | 同 | 同 | |

## 门禁（tasks 3.1，实际命令与结果）

- `cargo test --workspace --tests --locked --no-fail-fast`：**2739 通过 / 0 失败**（270 个目标；含本片新 `tests/p3_labeled_loop_tail.rs` 5 项与三个改名同步文件；最终记录跑在最终文件状态上取的）。整仓跑期间先后出现两次与负载相关的既有 flake，均与本片无关并按"计时类 flake 重跑判定"规则排除：`bulk_recovery_delivery::one_declaration_bounds_the_librarys_own_presentation_too`（4 worker 档 `budget_stops` 为空；两棵树隔离 13 次全绿、本片四个 fixture 类文本零变化）与 `p4_plugins::the_plugin_plane_leaves_the_structural_planes_own_answer_untouched`（两次运行的唯一 diff 是 `elapsed_millis: 0` vs `1`；隔离 5/5 绿）。两者登记为独立观察项。
- `cargo fmt --all -- --check`：通过（新增测试文件三处先行格式化）。
- clippy：`.github/workflows/ci.yml` 完整 30 项 `-A` 清单 + `-D warnings`（`--workspace --all-targets --all-features --locked`）：0 警告。
- `openspec validate --all --strict`：**229 通过 / 0 失败**（任务文本写 230 项，主线现存量即 229，全部通过；差额属任务撰写时的计数漂移，与校验结果无关）。
- `git diff --check`：干净。
- 磁盘纪律：构建/测试前 `df -h /` 检查，13Gi 时先 `cargo clean`（释放 16.1GiB）再跑 clippy 与整仓测试。
