# 验收重放

`root/summary.json` 与逐类输出是 root 的最终 23 类 × 两编译器 × 基线/候选完整重放；`local-gates/` 是已通过的本地门禁。`acceptance-manifest.json` 标识验收时的源文件与 CLI 二进制 SHA-256；临时 CLI 路径不属于提交产物，系统重启或临时目录清理后需重建。

仅允许 root 串行构建，使用共享 target，并保持至少 20 GiB 可用空间。若要重新比较基线，可从已提交历史导出源码，不创建占用分支的工作树：

```sh
archive_dir=$(mktemp -d /tmp/jarde-field-baseline.XXXXXX)
git archive 2dea3217 | tar -x -C "$archive_dir"
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  CARGO_TARGET_DIR=/Users/lordcasser/workspace/projects/jarde/target \
  cargo build --manifest-path "$archive_dir/Cargo.toml" -p jarde-cli --locked
cp target/debug/jarde-cli /tmp/jarde-generic-baseline-cli
rm -rf "$archive_dir"
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo build -p jarde-cli --locked
cp target/debug/jarde-cli /tmp/jarde-generic-final-cli
python3 openspec/changes/prove-generic-field-write-source-types/results/replay-root.py \
  --baseline /tmp/jarde-generic-baseline-cli --candidate /tmp/jarde-generic-final-cli
```

脚本的 JDK home 是实际 Corretto 8u432 与 OpenJDK 23.0.1。其他环境需先明确编译器，再修改 home；不能把两个 `--release 8` 调用称为两个真实 JDK。重放完成后清理共享 Cargo target。`generic-holder-write-boundaries/replay.sh` 是冻结前 14 族原源码/JADX/Jarde **基线** 的脚本，不能将候选 CLI 冒充其基线输入。

`follow-up/raw-receiver` 是额外的保守质量退化取证，只做过源码/字节码及完整编译对照；不计入 23 类行为验收，不宣称 JVM 行为或字段反射一致。该接收者成员类型能力另行立项。
