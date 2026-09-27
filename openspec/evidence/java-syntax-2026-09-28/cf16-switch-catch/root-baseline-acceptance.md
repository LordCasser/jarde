# 固定 runTest 基线独立复核

root 在主线 `5bf6d917` 单独构建 Jarde CLI，并把 `replay.sh` 复放到独立临时目录。固定内部类 SHA-256 为 `d9b9cb676203d943ee3cf97d66e62dda2a637125d7f737be1e22ca275c209185`，最小完整类为 `ff95cdf7c8eb36aa92a5429ba3e17ee75ca48b02ee40b8ea89598c8cf05b92f9`；四个方法的 BCI/opcode/异常表逐项相等，`runTest` 的 `[11,61)→64`、switch BCI 12、transfer BCI 61、return BCI 79 与固定预期一致。

原 class 与 pinned JADX 在固定类和最小完整类各自的九路径上均通过 Java 8 重编、`java -Xverify:all` 运行且输出逐字一致。fresh Jarde CLI 对两类的 `test1/2/3` 都给出可呈现正文，唯独 `runTest` 报 `jre_region_ownership_overlap` 并整方法解释性拒绝；其非 void 源因此无法重编，JVM 运行未执行。根级复放的 `results.txt`、方法形状、四种负例形状及所有原/JADX 运行轨迹与冻结证据逐字一致。三个部分 arm catch 与一个真实 TWR 近邻均通过 Java 8 编译及 JVM 校验运行；它们只证明这些输入有效，不代替未来实现的拒绝测试。

脚本已增加固定输入 SHA 检查。此验收冻结的是实现前基线；临时诊断中 TWR 假候选和 BCI 61 未归属的成因另见 `switch-inside-catch-region-debt.md`。生成源码可编译与否、原始类可运行与否、Jarde 恢复质量在这里分别记录，不把固定 `runTest` 视为已恢复。
