# TestNestedLoops2 三方基线静态复核 v4（未执行）

v4 保留 v1/v2/v3，基于 v3 再明确区分编译失败时的 class 文件残留与成功编译产物：`check_compile_observation` 仍逐一校验磁盘 class 路径与 collector 的记录、以及每个实际 class pin；但 compile exit 非零时，结果只给 `class_paths_observed`，将 `class_set_exact` 置为 null，并且 `runtime` 必须 null。只有 compile exit 0 才计算 `class_set_exact` 布尔值；成功编译才期待真实 runtime command。Jarde 编译失败不运行，也不会被输出成 class-set 或 runtime 对齐成功。

## 对抗性静态复核结果

- Oracle 是固定的最大字符串长度判定，不是长度总和。Runner 九行固定输出；`[6,6]` 和 `[10,10]` 都必须 false，`[11,0]`、`[0,11]`、升序 `[3,11]`、降序 `[11,3]` 必须 true，空列表 false。null list 与 null element 仅由 Runner 捕获，均必须打印 `java.lang.NullPointerException`。两 JDK 原始完整类都必须通过精确九行 stdout、空 stderr、exit 0；任意异常或分类改变都会拒绝该 oracle。
- 成功的 JADX 两 profiles 和 Jarde 四腿都与同 JDK 原始 oracle 比较真实 stdout/stderr/exit。JADX 工具失败仍保留原始命令 streams 与输出 inventory；若失败命令仍产出唯一固定类源，按 collector 实际行为继续核编译/run，绝不漏掉此腿。
- 命令索引从捕获行重建连续 1-based 序列；条件 labels 对照 collector 的确切流程：original compile→仅成功 run→实际 class 文件存在时 javap；JADX command→实际输出唯一目标源时逐 JDK 编译、编译成功才 run；Jarde render→raw JSON 可解析且有完整 source text 才编译，编译成功才 run。无硬性 29 命令要求。`classfile` lookup 使用 `base / ... / (internal_name + '.class')`，没有 Path 与字符串相加。
- `compile_case`/checker 字段已对应核对：源码 pins 对应各 leg 实际 source 参数；command labels 为 `<leg>-compile` / `<leg>-run`；`class_paths` 与磁盘逐项相等，实际文件与 class SHA pins 相等；compile 失败时 run 字段/命令必须不存在、`runtime_comparison` 必须 null；真实 run 存在时按原 oracle 独立重算 comparison。
- Jarde 每个成功 report 都核完整物理成员及顺序（构造器和目标方法）、动态目标 owner/name/descriptor、flags、方法表索引、Code envelope、javap 与 classfile 全部物理 BCI，以及所有 source-map UTF-8 spans 和每个物理 BCI 的来源。逐成员记录实际 `quality`、`fallbacks`、`content`、`compile_status`，不要求 structured。Default/all 两份都有效时核 class 正文和各方法 text/map 相同。编译失败只留下实际 class 路径观测，无 runtime；编译成功也仅把运行是否相同作为真实结果记录。
- Receipt 由 v9 实际 JSON 静态确认：顶层 `status=passed_candidate_build_only`；`candidate_cli` 包含 path/bytes/SHA/mode，不含 status。collector/checker 比较顶层 status 与 `candidate_cli` 完整 path/bytes/SHA/mode。

本复核只读了 v1-v3 脚本、v4 postfix collector/checker 与指定 v9 receipt，运行 Python AST parse 和哈希/路径静态核对；没有运行 collector/checker、JDK、JADX、CLI、Cargo 或改共享仓库。baseline 尚待 root 实际运行；固定一例的 fallback 观察不会单独证明需要添加机制。

v3→v4 精确最小 diff 在 `checker-v3-to-v4.patch`（只将失败编译残留 class paths 标为 observed，compile 成功才给 `class_set_exact`）；v4 checker/collector 和引用脚本 SHA 见 `pins.json`。输入源和 Runner 沿用 `/private/tmp/jarde-cf11-nested-loops2-prep-luna-v1/`；等待 root 实跑前不冻结 CLI/JDK/JADX/输出参数。
