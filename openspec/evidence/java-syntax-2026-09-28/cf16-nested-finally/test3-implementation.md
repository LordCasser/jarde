# test3 三副本清理片验收记录

本片只修改共享 `Joined` finally 证书对清理效果的证明。三份清理的字段符号（owner/name/descriptor）、字符串常量、`StringBuilder.append(String)` 调用符号与调用种类完全相等；每份从入口 `Local(0)` 读取 `this`，通过本次字段读取取得接收者，经唯一调用消费，并由紧邻的 `pop` 消费调用结果。所有链按物理 BCI 和 SSA 栈槽核对，每个产出值在全方法中恰有一次使用。字符串和字段名不绑定固定测试字面量。旧布尔字段写入证书保留为独立私有 helper。

固定 `TestTryCatchFinally12$TestCls.class` 的 `test3` 行为由既有 `three-method-bytecode.txt` 定住：异常表 `[0,5)→18 NPE`、`[0,5)→42 any`、`[18,29)→42 any`；正常清理 `[5,15)` 与 `[29,39)` 分别由 BCI 15、39 跳到 BCI 55；handler BCI 42 保存原异常、`[43,53)` 清理、53–54 重抛。证书的行、边、handler、唯一 join 和排他 owner 证明保持原样。定向测试断言 `Plan::join=55` 且不在 `Plan::owned`，Region 的物理块各有唯一 owner，BCI 0–55 每条指令均有源码来源。

用 `python3 test3-mutants.py` 可从固定 class 重建一个等价正例和五个近邻反例。`replay-test3.sh JARDE_CLI OUTPUT_DIR` 重放原类、固定 JADX 和当前 Jarde 的最小完整类 `test3` 三路径，并对固定 class 和六个变体执行 `java -Xverify:all`。本次结果：原/JADX/Jarde 三路径逐行相同，分别为 `call-finally`、`call-npe-catch-finally`、`call-iae-finally`；固定类与六变体均通过 JVM 验证和执行。等价正例将三份清理参数都改为 `-catch`，仍恢复唯一 finally，证明证书比较参数而非绑定固定字面量。五个反例的区别是首份常量、首份字段（新增同类型 `sb2`）、首份调用目标（StringBuilder 的 CharSequence 重载）、首份结果消费（`astore_2`）、catch-all 行终点扩到 BCI 15；定向 Jarde 测试对五者都不发布 `finally`，保留字节码拒绝。参数不等变体的正常结果是 `call-catch`，另四者的正常结果与固定类相同。

`replay.sh` 的全类九路径仍在 Jarde 的 `test1` 第一条断言失败，因为 `test1/2` 两副本外层清理尚属任务 3。本片的 `test3` 三路径与独立后续块已通过，任务 1.2 的 `test1/2` 负例、任务 3、4.1–4.3 仍未完成。
