# P5 billing 与 fingerprint 局部闭环归档草稿

此目录只提供归档脚本和说明。归档由 root 在最终门禁完成后执行；本草稿没有运行门禁，也没有生成归档。

脚本将保存以下证据目录的原始文件：

- `/private/tmp/jarde-p5-billing-delta-audit-root-v1`：collector 实际 90 cmd / 43 methods 的 delta 审计、build receipts 与 guarded/boolean-context 证据。
- `/private/tmp/jarde-p5-billing-recorder-root-v1`：历史 recorder execution 和原始 stdout/stderr。
- `/private/tmp/jarde-loop-field-fingerprint-root-v1`：历史 interrupted fingerprint execution 与原始输出。
- `/private/tmp/jarde-fingerprint-interruption-recovery-root-v1`：恢复后的 execution、原始输出和状态记录。
- `/private/tmp/jarde-loop-field-fingerprint-input-audit-root-v1`：输入 inventory。
- `/private/tmp/jarde-p5-billing-accept-luna-v1`：未来只读验收材料。
- `--collector-dir` 指定的 collector 私有源码、pins 和 README v2（预期 SHA-256 前缀 `133cfe…`）。
- `--final-gate-dir` 指定的 `/private/tmp/jarde-ordered-field-final-check-root-v6` 完成目录，递归收集 receipts/raw、root summary、root runner、spec 和 acceptors。
- 仓库快照 `tests/p5_bulk_corpus.rs` 与 `tests/fixtures/corpus-fingerprint.json`。

脚本按字节原样归档文件，不改写历史记录；跳过大于 20 MiB、含 NUL 字节或常见二进制扩展名的文件，以免打入巨型 CLI。源码和 pins 应保持小型文本文件。归档 SHA-256 会在成功结束时打印。

历史 `interrupted` 记录仍是 interrupted；恢复记录只证明恢复阶段产生的材料。归档不把它们改称成功，也不重新解释或重写历史。delta 审计中语义完全不变；仅 3 个 Guarded 方法分别为 +13、+13、+18。

这份归档只证明本地 P5 billing / fingerprint 的局部证据闭环。它不替代双 seed 复跑，也不代表自身 CI 已通过。root 会单独制作最终 full-workspace 归档。

root 在两个输入目录齐备后执行：

```sh
python3 /private/tmp/jarde-p5-closure-archive-prep-luna-v1/archive_p5_closure.py \
  --collector-dir /private/tmp/实际collector私有目录 \
  --final-gate-dir /private/tmp/jarde-ordered-field-final-check-root-v6
```

输出默认写到 `/private/tmp/jarde-p5-closure-archive-prep-luna-v1/p5-billing-fingerprint-closure.tar.gz`。如需指定位置，追加 `--output /path/to/archive.tar.gz`。
