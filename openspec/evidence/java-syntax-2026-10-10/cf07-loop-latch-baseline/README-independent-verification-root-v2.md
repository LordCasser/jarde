# CF07 独立验收脚本 v3

`verify-baseline-root-v3.py` 只读取 `baseline-root-v2/` 的冻结清单、源码、class 文件、JSON、javap 文本和原始命令流，并核对实际 JDK、JADX、Jarde CLI 文件的 SHA-256。它不调用这些工具，也不改动输入。

由 root 审阅后执行脚本时，结果写入本目录的 `independent-verification-root-v2.json`；如果结果文件已存在，脚本会拒绝覆盖。脚本依赖当前 Python 环境中的 `blake3` 模块，不会安装依赖。

脚本逐项核对 118 个 inventory 文件、29 条命令及其原始 stdout/stderr、10 个编译/运行 case、独立 class 输出集合、空 classpath/sourcepath、同 JDK 原始运行输出、Jarde JSON 与生成源码、class-byte BLAKE3 owner，以及四个方法的物理 flags/BCI 和 source-map owner/BCI/span。default/all 两种渲染会比较全文、方法文本和 source map。

基线已记录的 source-map 缺口是 `andWhile@15`、`counted@20/@30`、`lastIndexOf@25`。物理 javap 分别观察到 `goto 15→2`、`goto 20→27`、`goto 30→6`、`goto 25→5`。这些缺口按原始观察记录；结果不声称完整 BCI 覆盖，也不声称 candidate 已验收。

此脚本尚未执行；本 README 只说明其预期输入、检查和输出位置。
