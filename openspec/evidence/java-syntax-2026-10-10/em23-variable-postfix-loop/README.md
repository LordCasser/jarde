# EM-23：增强 for 内条件局部自增基线准备

本片只准备 `TestVariablesDefinitions2.TestCls.test(List<String>)` 的一个最小完整类对照。目标类保留 null guard、增强 for、条件 `i++` 和最终返回；Runner 顺序覆盖 null、空列表、含两个空串的混合列表、再次调用及含 null 元素的异常路径。原类在每个 JDK 上的退出码、stdout、stderr 是 oracle；没有将手算输出写成验收条件。

输入：

- `inputs-prepared-luna-v1/VariablePostfixLoop.java`，SHA-256 `17147219c9e524d18d62063f932b1ebb3ae976cb6f3bd3fddd2bfd8a58889199`
- `inputs-prepared-luna-v1/Runner.java`，SHA-256 `4408a0d8dea6b761b35546e314dc9961cf515ddeb31920eb5665524598f03102`
- `results/prepare-baseline-luna-v1.py`，SHA-256 `2e35326ebe7b2b60bd47b60419375088bc36142e053ea41f086f3cf534743089`
- `results/prepare-baseline-luna-v1.py` 使用固定 JDK manifest `recover-nested-int-array-compound-updates/results/controls-v1/manifest.json`（SHA-256 `ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec`），并要求调用者传入候选 CLI 和 metadata 路径及 SHA-256。当前准备目标为 `/private/tmp/jarde-int-array-names-cli-v2`（SHA-256 `51e17991cbf7cb6cebb43d2ea3b66dfe655522e1f8c3da9a69b05ba47f698067`），metadata `recover-int-array-constant-names/results/candidate-cli-v2.json`（SHA-256 `dd2a5a0d5731ca5fa5a12e87c9e344c6faaf269c82bc128f935b57e8681ec282`）。

执行脚本会以 Java 8 source/target、空 classpath/sourcepath 和新 classes 目录，准备原类双 JDK、JADX 两配置双 JDK、Jarde default/all 双 JDK的完整源码重编与 `-Xverify:all` 运行。JADX 配置沿已接受 collector 的 `default/none` 命名；Jarde 证据模式为 `default/all`。只允许因 JADX package 而给 Runner 添加包前缀，目标源码按生成内容原样编译。脚本保存 argv、三元 raw 流、完整类集合、原 class BLAKE3、`javap -p -c -s -v` 与物理方法 BCI、Jarde physical method/source map 事实、source/class hashes 和闭合 inventory；运行结果写到新的 `baseline-root-v1/`，拒绝覆盖已有目录。

脚本沿用 `em22-mask-condition-next/results/prepare-baseline-luna-v1.py` 的已接受单类收集流程，仅替换类/方法身份及静态呈现观察。原 class raw 是唯一 oracle；Jarde 输出是否拼成 `i++` 只记入 manifest，不影响语义成功判定。本片不预测原始输出、不声称有产品缺口、不要求追平 JADX 文本，也不扩展 EM-23 其它形状。

本次只做准备，没有运行 collector、Git、Cargo、JDK、JADX 或 Jarde CLI。请 root 审阅脚本及输入后自行执行。
