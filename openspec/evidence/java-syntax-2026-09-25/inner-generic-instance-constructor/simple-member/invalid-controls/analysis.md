# 普通源码和缺失定义对照（1.2 的部分证据）

本目录仅用 Java 8 源码和 classpath 选择制作控制，不修改 class 字节或元数据。`build-evidence.sh` 以 `javac --release 8 -g:none -Xlint:-options` 编译既有 `../../fixture/{SimpleOuter,UseInner,InnerRunner}.java` 及本目录 `fixture/*.java`，在私有临时目录运行并删除该目录。`SHA256.txt` 冻结 class 与 jar；`UseInner.class` 和 `SimpleOuter$Inner.class` 的 SHA 分别为 `e44ce3a6eb4cba73ec50c530727d6764680d84e14c2d58d899083b002028ccae`、`7ce649089f60472371b1a2ca9308175d601dd3df259752dd2451d941459d2c4d`，与既有简单成员正例一致。

`EffectOrder.qualified` 是原限定构造的绿侧：`javap-EffectOrder.txt` 中 BCI 6 `Objects.requireNonNull` 先于 BCI 13 `mark("A")`，BCI 16 调用物理 `(Lnested/SimpleOuter;I)V` 构造器。`preparedBeforeCheck` 是**不同 Java 源级求值位置**的红侧：BCI 3 先 `mark("A")`，BCI 8 才显式 `requireNonNull`；随后 BCI 18 仍有 javac 为限定构造自动生成的检查，BCI 23 调用构造器。两者均在 `java -Xverify:all` 下运行；`effect-run.txt` 分别记录非空时 `qualified:7:A`、`prepared:7:A`，空值时 `qualified:null:`、`prepared:null:A`。这证明移动普通实参效果跨过 null 检查会改变可观察顺序；它**不是**“同一限定构造表达式内，自动 null-check 晚于普通实参”的 verifier-valid 反例。普通 Java 源码没有生成该反例，本目录不声称该拒绝门已有红/绿 proof-unit 断言。

`missing-target.jar` 的条目在 `missing-target-entries.txt`：含完全相同的 `SimpleOuter.class`、`UseInner.class` 和 `MissingRunner.class`，故意不含 `SimpleOuter$Inner.class`。完整 jar 的 `present-target-run.txt` 为 `made:nested.SimpleOuter$Inner`；缺失 jar 在 `java -Xverify:all` 下的 `missing-target-run.txt` 为 `missing:nested/SimpleOuter$Inner`，即解析时 `NoClassDefFoundError`，未见 `VerifyError`。`original-run.txt` 对完整 jar 仍给出 `7:IA`、`null:I`。调用方 `javap-UseInner.txt` 的 BCI 0 `new`、6 `requireNonNull`、13 `mark`、16 `<init>` 与原正例一致；缺失的只是选中 jar 中的目标定义。这个对照证明 classpath 缺失的物理事实，不证明当前 Jarde 已有“缺失目标”专门拒绝门。

JADX 1.5.6 对完整 jar 的源码在 `jadx-full/`，以 Java 8 重编成功；`jadx-effect-run.txt` 与 `effect-run.txt` 逐字相同，`jadx-original-run.txt` 与 `original-run.txt` 逐字相同。对缺失 jar，`jadx-missing/sources/nested/UseInner.java` 输出 `new SimpleOuter.Inner(simpleOuter, ...)`，`jadx-missing-compile.txt` 记录 Java 8 编译因找不到 `Inner` 失败。因此缺失环境没有可执行的 JADX 重编行为可对照。

当前工作树编译的 CLI（`jarde-cli-sha256.txt`，构建时 HEAD `a83b5572a93dd316dac1eb7697208fecd6a6fdbc`）由 `run-jarde.sh` 对两个 jar 及效果控制调用 `class-source --format json`。三个报告均完成运行；`UseInner.make` 在完整与缺失 jar 中同为 `fallback`、`explanation_only`，诊断 `jre_new_interleaved_effect` 均指向构造 BCI 0 与中间 `Duplicate` BCI 5。`EffectOrder.qualified` 同样在 BCI 0/5 拒绝；`preparedBeforeCheck` 在构造 BCI 12 与 `Duplicate` BCI 17 拒绝。Jarde 的现有拒绝早于目标绑定，因此这个运行**不能**归因于缺失定义，也不是缺失门的红/绿测试。原始 JSON 保存于 `jarde-*.json`。

后续已在 [字节错形控制](byte-variants/analysis.md)补上三个 verifier-valid 的独立 class 变体：目标关系错误、检查值与物理首参身份不同、同一构造站点的空值检查晚于普通参数效果。它们均有 `javap`、原 JVM、JADX 与当前 Jarde 结果；本页上述普通源码对照仍保留为不同源级求值位置的独立证据。OpenSpec 任务 1.2 要求各门在 Jarde 准入中有真实红/绿断言，而目前绿侧也被既有中间效果门拒绝，因此仍不勾选 1.2。

2026-09-25 后续已扩为五个字节错形，补上外层 class 的成员关系条目反例与保留完整检查形状的 SSA 身份反例；上段的 Jarde 拒绝与未完成状态是修前记录。现有[Root 验收](../../../../../changes/recover-proved-member-inner-construction/verification-root.md)用完成后的 CLI、直接 proof-unit、Java 8 重编和 JVM 行为核对这些门。
