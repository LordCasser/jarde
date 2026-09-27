# CF-04 嵌套整数条件值主线独立验收

root 将实现 `4ccae32e` 拣入本地主线 `f14a5113` 后，重新编译主线 `jarde-cli`，以固定 [replay.py](replay.py) 在全新临时目录重放；没有使用子代理的编译物。原 class、固定 JADX、Jarde 的**完整 `TernaryCases` Java 8 源码**均经 `javac --release 8 -g:none` 重编并以 `java -Xverify:all` 运行，13 行返回及副作用输出逐字相同。隔离 `TernaryBasic` 的三侧完整源码也均重编、验证运行 11 行一致。root 生成的六份源码 SHA-256 与实现分支 `after/summary.json` 逐项相同，Jarde `TernaryCases` 为 `521622836f41e80cc4c07e5587280e53e3d44f4f9f676e0adebb1add28184e0f`。

root 主线 `cargo test -p jarde-java --lib` 为 230/230 通过；`cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-proved-nested-int-conditional-return --strict` 通过。实现还以非直接 literal、额外 consumer 与 handler 边负例验证原子拒绝、物理来源定位；布尔 1/0 消费保留原门。验收仅覆盖固定样例的直接 int 常量、唯一 `ireturn` 条件值，任意表达式生产者与数值转换仍在 CF-04/CF-05 的待扩验边界内。

重放：

```sh
python3 openspec/evidence/java-syntax-2026-09-27/cf04-ternary/replay.py \
  --jarde /tmp/jarde-root-integration-target/debug/jarde-cli \
  --jadx /Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx \
  --jadx-checkout /Users/lordcasser/workspace/testzone/jadx \
  --out /tmp/jarde-cf04-root-replay-20260927
```

`--out` 须为空目录；临时目录仅保存独立运行产物，仓库的 `after/` 保存子代理的可审查原始证据。
