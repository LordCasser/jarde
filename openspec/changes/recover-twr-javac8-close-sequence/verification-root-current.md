# 当前主线 root 复核（2026-10-09）

root 用 CLI9（`/private/tmp/jarde-generic-calls-candidate-v9-cli`，SHA256 `5bb2fcfdcf958839f0b1e060a2e55114510c6115cb225ec3279d7d7adf95e006`）实际重放两条冻结 TR 输入。`class-source --class TR --policy single-class --format json` 的完整 argv、exit、输入及双流 hash 在 [cli-run.json](results/current-main-2026-10-09/cli-run.json)，两条原始报告均 exit0。真 javac8 输入 SHA为 `7e4831366bb775adf65924b597ffbc1ba0bd136a6bd8e0d0a2baf8eb899fcda1`，javac23 Java8目标输入为 `1401152b546f9dd1528fc9683882ac9334b04944dba01a79ffb6892c6025660d`，与永久 fixture 一致。

真 javac8 的 `one()` 已恢复为单资源 TWR，`two()` 仍有拒绝标记。root 保留全部成员，使用 javac23 `--release 8`、空classpath/sourcepath和各腿新建classes目录编译完整生成源码；编译及运行不借用原输入。完整源码、命令、双流和hash见 [fullclass-run.json](results/current-main-2026-10-09/fullclass-run.json) 及相邻文件。

真 javac8腿完整重编 exit1：拒绝的 `two()` 缺少返回语句。因此该腿的完整类编译/验证运行验收未满足。javac23腿完整重编exit0，`java -Xverify:all` exit0，输出依次为 `closed:p`、`used:p`、`closed:qb`、`closed:qa`、`used:qaused:qb`。

旧 `roundtrip-single-resource/compile-and-run.txt` 删除拒绝成员后验证了 `one()` 路径，仍是有效的局部证据，但不满足任务3.1的完整类要求。主线 `82090226` 的实际 [CI 37892494665](https://github.com/LordCasser/jarde/actions/runs/37892494665) 四项通过，为既有负控制与常规门禁提供证据，不能替代失败的完整类腿。

据此，3.1–3.5不能整体勾选。历史全语料差分与通过的门禁仍保留；真javac8多资源形和CF-17/P08账本处置待独立推进。本记录不改任务勾选或生产代码。
