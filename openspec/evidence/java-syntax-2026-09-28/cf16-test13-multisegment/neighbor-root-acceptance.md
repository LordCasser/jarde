# Test13 近邻负例独立复核

主线 `32155897af127badaa495979011d6c0838215400` 上由 root 单独构建 Jarde CLI，并复放 `negatives/replay-neighbors.sh` 至 `/private/tmp/jarde-test13-root-negative-replay`。`bash -n`、mutation 脚本语法检查和 `negatives/SHA256SUMS` 全部通过。五个变体的 `results.txt`、原 class/JADX/Jarde runner 输出以及模板 BCI/opcode/异常表形状检查，与提交证据逐字一致；五个原 class 均在 `java -Xverify:all` 下通过各自轨迹断言。

四个负边界分别证实：早退清理调用目标不等价、条件分支可绕过清理、扩展 catch-all 范围会使清理抛错后二次执行、替换保存的重抛异常会改变最终异常类型。固定 JADX 对前三种改变中的目标、分支以及重抛保持原始行为，但对扩展范围样本虽能重编，却漏掉第二次清理；这不能作为 Jarde 的等价规则。Jarde 当前对前三个变体整方法安全拒绝，对改重抛的变体局部拒绝，生成文本均可重编但不能通过原始运行断言。该证据只供后续 Test13 证书边界设计，尚非 Test13 实现验收。
