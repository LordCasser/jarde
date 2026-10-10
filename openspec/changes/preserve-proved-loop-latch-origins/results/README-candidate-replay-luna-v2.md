# 候选回放准备记录 v2

此版本只准备候选回放脚本，尚未运行。它面向 root 后续冻结的新 CLI 和 metadata；必须用 `--cli`、`--cli-sha256`、`--metadata`、`--metadata-sha256` 显式传入，不会默认借用旧候选。

脚本固定读取已接受的 `one-arm-loop-controls/baseline-root-v1`，重做两套 JDK 的原始类、JADX default/none 和 Jarde default/all 完整类矩阵。它把整类编译失败记录为独立观察：三个未变 fallback 方法导致的失败必须与已保存基线诊断一致，不能算作方法来源映射验收通过，也不会虚报运行结果。只有 noPrefix 的方法级来源映射按该片目标检查：保留所有旧来源，并在原 while span 上新增唯一 derived BCI 14。

相对 v1，本版本修复了首个 render profile 访问尚未创建列表行的问题，并将 javap 方法身份键转换为 JSON 字符串键。Proposal、design 和 delta spec 是阻断式语义上下文固定项；`tasks.md` 会记录准备时和回放时的哈希，但仅作过程信息，以免正常勾选任务令语义验收失效。v1 原文件与证据保持不变。

静态检查仅做 Python 语法编译、冻结基线 helper/记录结构检查和 JSON 可序列化检查。没有运行本脚本，也没有运行 JDK、JADX、Jarde CLI、Git 或 Cargo。root 冻结新候选 CLI 和 metadata 后，可按以下形式执行：

```sh
uv run --with blake3==1.0.11 python3 openspec/changes/preserve-proved-loop-latch-origins/results/prepare-candidate-luna-v2.py \
  --cli /private/tmp/<frozen-candidate-cli> \
  --cli-sha256 <candidate-cli-sha256> \
  --metadata openspec/changes/preserve-proved-loop-latch-origins/results/<candidate-metadata>.json \
  --metadata-sha256 <candidate-metadata-sha256>
```

输出目录固定为 `candidate-replay-root-v2`，若已存在会拒绝覆盖。新 CLI 或 metadata 尚未由 root 冻结；这份准备记录不代表候选已验证。
