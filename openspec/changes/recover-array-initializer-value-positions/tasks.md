## 1. 取证与基线（root 已完成大半）

- [x] 1.0 六位判别、双重证伪、jadx 双解——实测归档。（root 已完成）
- [x] 1.1 插桩定位 "copy … has no proved local assignment" 发出处与 dup 值的 SSA 表示；转录存证据。
      → `results/01-gating-refusal-point.md`：两处拒绝点**不同**。位 1（`MD.partSet`）的候选**已证**、消费门已过（`0x53` 值位臂），拒在 commit walk 的整类跳过（`build.rs` 的 "a child whose sole consumer is an array store cannot stand alone"）；位 2（`MD3.bareIdx2`/`MD2.retPos`）的拒在 `prove_array_initializer` 的末位消费门——门读的是"最后一个元素存储之后的那条指令"，而下标先产生其索引（`iconst_0`），dup 值的**唯一读者**是更后的 `iaload`（`single_use=Ok(false)`）。子几何（外层初始化器元素）的每条 store 其数组操作数都由父级 `dup` 产生，实测逐条列出。
- [x] 1.2 重验基线：主线二进制渲染 MD/MD2/MD3（两位拒、其余恢复）；负例探针（双消费 dance 值）现状拒绝记录。
      → `results/02-gating-experiment.md` + `results/02-gating-experiment.out`（`02-gating-experiment.sh` 可复跑）：父提交（`bd6ba671`）下 MD.partSet / MD2.retPos / MD3.bareIdx2 三位整方法引注；`MD2.retPos` 的 fixture 注释写作"返回位"，字节码实为立即下标位，与 `MD3.bareIdx2` 同形同拒。负例 `AVN.twoReaders`/`AVN.discarded`（手工构造，`javac` 不发射）在两侧逐字节不变。另注：巡查期存档的 MD 文本来自更早的副本族切片之前，父提交的 `partSet` 已是整方法引注（值级守卫升级），本片以父提交实测为准。
- [x] 1.3 冻结 fixture：MD/MD2/MD3（javac23 `--release 8`）+ 真 8 腿（验证两版 dance 同构）入 `tests/fixtures/`，README 记编译命令与 SHA。
      → `tests/fixtures/recover-array-initializer-value-positions/`：巡查三源逐字节复制 + 本片 `AV`（同两位各进一步：变量/计算下标、长度接收者、双 dance、顺序位）+ 手工构造 `AVN`（`build_avn.py` 逐字节装配，三成员：单读者自检、双读者、丢弃第二消费方）+ `v8/`、`v8-javac8/` 双腿 + README（编译命令、答案表、每位形状）。

## 2. 实现

- [x] 2.1 按决策 1 泛化：dance 单写者 + dup 单读者即呈现于消费位（元素存储 RHS / 立即下标先行——主锚两位）；多读者保持拒绝。
      → `results/03-implementation.md`：`array_initializer_reader`（读者 = dup 值的**唯一 use**；中间运行段 = 该读者的另一操作数，用元素值同一条 `collect_expression_bcis` + `dependency_uses_stay_within` 判读，非此即保持不证）+ `array_initializer_consumer` 新增"下标/长度**接收者**位"臂 + commit walk 只跳过**子几何**（store 的数组操作数由本块 `dup`/creation 产生，`store_writes_into_a_constructed_array`）。既有判据、拒绝文本、`owned`/`element_sources`、引注走查（`quoted_bcis`/`deferred_producers`）与 postfix 核算检查逐字未动。
- [x] 2.2 呈现保持显式形（决策 2）；四位合格位与裸立即消费零改动。
      → 呈现即 `new int[]{7}`/`new int[]{9}[0]` 显式形（无简写美化）；四位合格位、裸立即消费、锯齿族由 `the_presenting_positions_keep_their_text_byte_for_byte` 逐字节钉住（14 个方法体），实测 moved 列表不含它们。

## 3. 验证与验收

- [x] 3.1 主锚：MD.partSet 与 MD3.bareIdx2 恢复；三类 `javac --release 8` exit 0、行为逐行一致（双腿）。
      → `results/dual-leg-replay.sh`（+ 测试内 ignored 回放 `the_recovered_text_compiles_and_runs_identically_on_both_legs`）：MD/MD2/MD3/AV 双腿（javac 23 `--release 8` 与 Corretto 1.8.0_432）整类剥离文本编译 exit 0、`-Xverify:all` 运行、与原 class 输出逐字一致：`MD 21/9/9/10`、`MD2 3/3/4/5`、`MD3 2/9/3`、`AV 7/9/3/3/1/8/2/3/6/3/9/8/10/17/8/9/3/true/2/1/2`；四类 0 引注、0 "not recovered"。
- [x] 3.2 零回退：四位合格位 + 裸立即消费 + 锯齿族逐字节不变；负例（双消费）仍拒；corpus 双腿扫描 diff 为空。
      → `results/05-corpus-delta.md`（+ `05-corpus-sweep.out`、`03-corpus-sweep.sh`）：2807 松散类 + 全部 jar 条目两轮扫描，moved=22：本片 fixture 17 + 巡查 `md.jar!MD` 1 + 另一族的 12（`recover-loop-else-if-early-returns` 双腿 8 与四个巡查 jar：CA2/CP7/NL/BS）——12 全部同因：dance 作**非末位实参**（`bsearch(new int[]{1,3,5,7}, 5)`），读者即调用、位于其余实参运行段之后。22 个全部**行为复跑**（`results/04-corpus-delta-replay.sh`：恢复的 `main` 作子类单元编译运行、与原 class 逐字同输出，双腿双编译器）；负例 `AVN` 两成员逐字节不变；先例族（CF/NEG/RC/RCN/ICM/ICN）两侧逐字节。oracle 腿 3/3 绿、无陈旧期望（`results/oracle-leg.out`）。两个 unrendered 候选为 `package-info`（两侧同一 `operation_target_not_found`，harness 事实）。
- [x] 3.3 门禁全量（基线以合并态为准；flake 家族单测复跑两轮判定）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + 再生 fingerprint。
      → `results/06-gates.md`：fmt clean、CI-exact clippy 0 警告 0 错误、workspace 全量（exit + `test result: ok` 计数 + 零 FAILED）、openspec 306、diff check clean、census `(811,3445,290,2145,8) → (820,3536,290,2161,8)`（+9 类 +91 体 +16 分支目标）、fingerprint +13 条目 0 删除。
- [ ] 3.4 root 独立复核：判据未放宽（单读者语义）、零回退实测；关闭 summary.md 登记行。（留 root）
