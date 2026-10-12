# P5 billing 与 fingerprint 局部闭环归档草稿 v2

此版本只准备脚本和说明，没有执行最终门禁或生成归档。v1 文件保留供审阅，不再作为执行入口。

v2 递归保存历史 delta audit、recorder（含原始 stdout/stderr）、recorder command invocation/spec 记录、fingerprint interrupted/recovery、输入 inventory、只读验收材料、collector 私有源码/pins/README v2，以及 final-gate 目录内的 root focused runner、spec、acceptors、receipts/raw 和 root summary。它也保存两个固定 source snapshots；其他当前测试 source snapshots 可通过重复的 `--snapshot` 追加。

只排除所有输入目录中的 `__pycache__`，以及 final-gate 目录下文件名恰为 `candidate-cli` 的唯一巨型 CLI。其他文件完整纳入，包含二进制和大文件。脚本为每个 tar 成员生成 `*.tar.gz.manifest.json` sidecar，记录归档成员名、原始绝对路径、字节数和 SHA-256。随后重开 gzip tar，逐成员复核字节数、SHA-256 和原文件内容。归档或 sidecar 已存在时会停止，不覆盖。

归档前会在 final-gate 下查找唯一一个文件名包含 `root-summary` 的 JSON，并要求它记录：`status` 为 `commands_passed_pending_independent_acceptance`、`target_absent` 为 `true`、`source_before` 与 `source_after` 各一项且相等。root summary 中记录的 CLI 可以与 product binary 相同；此条件不构成全 workspace 验收，也不代表独立 acceptor 已完成。

历史 `interrupted` 仍按原样保留，不能据此声称成功。局部/P5 归档不替代双 seed 复跑或自身 CI；root 另行归档最终 full-workspace 结果。

root 在 `/private/tmp/jarde-ordered-field-final-check-root-v6` 完成后执行：

```sh
python3 /private/tmp/jarde-p5-closure-archive-prep-luna-v1/archive_p5_closure_v2.py \
  --collector-dir /private/tmp/实际collector私有目录 \
  --final-gate-dir /private/tmp/jarde-ordered-field-final-check-root-v6 \
  --snapshot /绝对路径/当前第三个测试source.rs
```

`--snapshot` 可重复指定。默认归档和 sidecar 写入 prep 目录；脚本最后打印归档路径、归档 SHA-256、manifest 路径和成员数。
