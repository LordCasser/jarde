# Corpus manifest failure and exact refresh

完整 workspace v3 使用修正后的单次 stat scanner，从头执行；62 批已实测通过，第 63 批中 P5 benchmark、bulk billing 与 container lookup 都通过，唯一失败为 corpus fingerprint 没登记本片六个实际 class。失败 command index64、exit101、guard_stop null、逻辑 target 峰值551252520bytes，最终 source pins 全恒同。该失败不计完整通过，不做计费表修订。

root 随后实际执行已有 `p5_corpus_fingerprint::regenerate_corpus_fingerprint`，1/0/0；保存 manifest before/after，独立确认其余顶层分类字段恒同、2062 旧 file entries 恒同，只有本片六个完整 class 新增，最终2068。独立 Python blake3 核每新增文件长度与digest，SHA256 也记入 `independent-fixture-digests-root-v1.json`；Git diff 恰30新增行。生成器之外没有改测试逻辑、fixture bytes 或预算常量。

完整 v4 从头重跑，不借用 v3 的62成功批。v4 输入 pins 额外显式包含运行时通过 FS 读取的 `tests/fixtures/corpus-fingerprint.json`。本目录的复制清单逐文件记录原始字节/SHA；v3 仍是失败观察，不能拿来接受产品完整 workspace 或 CI。
