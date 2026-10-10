# 候选完整类重放准备

`prepare-candidate-luna-v1.py` 只准备并重放 `PlainOneArmLoops` 的完整类对照。它沿用已接受的 [one-arm baseline collector](../../../evidence/java-syntax-2026-10-10/one-arm-loop-controls/prepare-baseline-luna-v1.py) 中冻结的解析、Java 命令记录和源码映射校验函数：脚本先按 SHA-256 校验旧 collector，再用 Python AST 只提取导入、常量、函数和类定义，不执行旧脚本的顶层采集流程。新输出固定在 `candidate-replay-root-v1/`，不会改写旧 baseline。

运行时必须由 root 显式提供刚构建并冻结的候选 CLI 路径、CLI SHA、metadata 路径和 metadata SHA。metadata 的实际接口沿用已存在记录中的 `cli_path` 与 `cli_sha256` 字段，并逐字核对二者与参数；候选 CLI 不会复制进输出。脚本同时锁定旧 collector、原始 Java/Runner、JDK manifest、JADX launcher、旧 baseline 的 manifest/inventory、旧 CLI/metadata，以及本 change 的 proposal/design/tasks/spec 哈希。若任何固定材料不一致，会停止在 preflight，并将失败原因写进新目录。

重放矩阵为双 JDK 的 2 个原始类、4 个 JADX default/none 类和 4 个候选 CLI default/all 类。JADX 输入 jar 仅含 javac23 编译的物理目标类。每个完整类都在空 classpath/sourcepath 下按 Java 8 编译；只有编译成功才执行 `-Xverify:all`。脚本重新采集每个类的 `javap` 身份与 BCI，并保存命令 argv、raw stdout/stderr、生成文档和闭合 inventory。

候选来源验收按方法逐项进行：完整生成类文本和五个方法正文必须逐字匹配冻结 baseline；`prefixWhile`、`loopAndTail`、`takenArm` 与构造器的物理 source map 必须相同；`noPrefix` 必须保留全部既有来源，只增加一个属于准确物理方法的 derived BCI 14，并且该来源 span 非空且覆盖实际 `while (`。默认/all 的正文与所有方法 source map 必须一致，`noPrefix` 必须覆盖包括 BCI 14 在内的全部物理指令。

完整类编译和运行是独立的产品观察。已冻结 baseline 中另外三个单臂方法仍是 fallback，并导致整个生成类缺少返回语句；因此这片预期仍是四次整类编译失败且零次运行。脚本比对候选编译诊断与冻结失败记录（忽略源文件绝对路径差异），只把该结果记录为观察，不把它算作来源修复失败或整类运行成功。任何 method text/map 范围扩大、BCI 来源不准确、默认/all 不一致或意外的整类结果都会留在 candidate 记录中并使 acceptance 失败。

准备状态：未运行 JDK、JADX、候选 CLI、Git、Cargo 或编译/运行命令。root 执行时使用真实冻结参数，例如：

```sh
uv run --with blake3==1.0.11 python3 openspec/changes/preserve-proved-loop-latch-origins/results/prepare-candidate-luna-v1.py \
  --cli /private/tmp/<frozen-candidate-cli> \
  --cli-sha256 <candidate-cli-sha256> \
  --metadata /absolute/path/to/candidate-cli.json \
  --metadata-sha256 <metadata-sha256>
```
