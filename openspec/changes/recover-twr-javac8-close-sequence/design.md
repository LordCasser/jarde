# Design：真 javac 8 TWR 关闭序列的几何证明

## 决策依据（root 已实测/读码，全部在 [TWR 巡查](../../evidence/java-syntax-2026-10-04/twr-javac8-codegen-patrol/README.md)）

1. **现拒绝链**：真 javac 8 的 `one()`/`two()` 渲染 `jre_region_exception_edge`（6）+ `jre_region_uncovered_blocks`（4）——region 层在 guard 的 `fn guarded`（guard.rs:12103 起）证明 TWR **之前或并行**就拒绝了（哪一道先、region 与 guard 的先后序——**root 未插桩**，task 1.1 定夺）。
2. **两腿几何差**（指令级实录，巡查三之三）：主路径 javac 8 = `aconst_null; astore_2` + 守卫式 close（`17: ifnull 46` / `21: ifnull 42`）；javac 23 = 无条件 close。异常表 javac 8 `one()` 5 行（3 Throwable + 2 any），any 目标 `53` 是抑制链块，javac 23 表中**无对应块**。primary 异常经 `athrow` 重抛被 `48-55→53` 的 any 行再捕获。
3. **guard 证明是 CFG/异常表驱动**（巡查第三节：fn twr 及其周边 3700-3860 段 opcode 字面量 = 0，六项 CFG 机制逐一核实）——即扩展必须以**结构事实**表达，不是认字节码序列。
4. **行变体路线已排除**（三之三分岔判定）；extend-Resources（A）vs 新变体家族（B）是留待插桩的分岔。

## 决策 1：插桩先行（task 1.1，不得预设方向）

实现者须先用插桩/增量实验回答三问，转录存证据目录：
- **Q-i 第一道门**：`jre_region_exception_edge` 与 `jre_region_uncovered_blocks` 各自由 region 层哪一判据发出、guard 的 `fn guarded` 是否被到达？（若 region 先拒，本片落点在 region 层的异常边/覆盖块判定，guard 扩展其次——落点与巡查第三节的"guard 层"预判可能不符，以插桩为准。）
- **Q-ii A/B 分岔**：现有 `Shape::Resources` 构造（`fn guarded` 内，锚点名 `Shape::Resources {`）的六项机制中，哪些在 javac 8 形上不成立？逐一列出后判定：不成立的判据能否以"新增几何事实"（抑制链块集、primary 槽、双守卫）**加严**而不**放宽**既有语义？能 → 方向 A；不能（任一判据必须放宽才能容纳）→ 方向 B（新变体，判据与 javac 9+ 形**并列**而非共享）。
- **Q-iii 抑制链的语义等价证明**：javac 8 的 `addSuppressed` 链在源级 TWR 里**不可见**（编译器合成），呈现须与 javac 9+ 腿同构（不含 addSuppressed 文本）——验证呈现路径（emit 侧）无需改动即可达到。

## 决策 2：判据只加严不放宽（零回退的不变量形式）

无论 A/B：**javac 9+ 形的既有判据逐字不动**。方向 A 允许"新增"对抑制链块的结构事实（这些块在 javac 9+ 形上不存在，新判据对其恒假、零影响）；方向 B 并列新变体。任何对既有判据的**放宽**（哪怕一行）= 停手条件 (b)。

## 决策 3：呈现与行为

- 主锚 `TR.one()` 真 javac 8 腿恢复为 `try (TR local1 = new TR(arg0)) { … }`（与 javac 23 腿同构），0 引注；整类渲染源集 `javac --release 8` exit 0，`java -Xverify:all` 运行输出与原 class 逐行一致（巡查 fixture 有 baseline）。
- 对照正例 `TwrAudit`（138 指令/1 `aconst_null`，真 javac 8）——巡查第二节已证其为 javac 9+ 指纹（cross-compiled），本片不动其行为，作为零回退锚之一。
- 负例（健全性，各冻结）：削一条 `any` 行的合成探针（抑制链覆盖不完整 → 拒）；守卫顺序错乱（`ifnull` 目标互换 → 拒）；抑制链断头（any 目标块不可达 → 拒）。每条证明"新几何事实是必需的"。

## 决策 4：无版本判据

两种产物 major 均 52；认"守卫式 close + 空槽 + 重抛回路 + 抑制链 any 行"这一**几何事实集合**。diff 审查出现 `java_release`/`major_version` 分支 = 停手条件 (c)。

## 验证标准（可证伪）

1. **主锚**：`TR.one()` 双腿（真 javac 8 / javac 23）都恢复同构 TWR 呈现、0 引注、整类可编译、行为逐行一致（真 javac 8 腿修复前 6 引注/整方法拒）。
2. **零回退**：javac 23 腿渲染逐字节不变；既有 `Shape::Resources`/finally 家族全部测试绿；`TwrAudit` 行为不变；corpus 双腿扫描差异仅真 javac 8 TWR 形类。
3. **负例**：三条削弱探针各保持拒绝。
4. **门禁**：全量测试（基线以合并态为准；flake 家族单测复跑两轮判定）、fmt、CI-exact clippy、openspec strict、`git diff --check`、fixture 后再生 fingerprint。
5. **插桩转录**：Q-i/Q-ii/Q-iii 的插桩记录与方向选择理由存证据目录（root 验收按此复核）。

## Open Questions

1. region 层与 guard 层的先后序及第一道门归属（Q-i，插桩）。
2. 方向 A 所需"新增几何事实"在 `fn guarded` 既有结构里的承载方式（新增块集字段 vs 既有 `cleanup` 行集扩展）——实现时定，但不得放宽既有字段语义。
3. `two()` 双资源（5× any）在 MVP 后的扩验顺序（Non-Goal，登记到账本）。
