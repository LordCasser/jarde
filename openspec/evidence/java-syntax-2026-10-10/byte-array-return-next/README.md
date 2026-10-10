# byte 数组直接返回对照

目标锚为 JADX TestArrayFill3 的 `byte[] test() { return new byte[]{0, 1, 2}; }`。本次实际使用 javac 8/23 的 Java 8 class 文件，不冒称覆盖上游 ECJ_J8/ECJ_DX_J8 编译配置或 Dex 输入。

root 实际执行 prepare-baseline-luna-v2.py：35 命令、10 条完整源码编译运行腿（原程序 2、JADX 4、Jarde 4）全部成功。重复调用分配不同数组，修改旧返回数组后再次调用仍返回原值；原/JADX/Jarde 的 exit/stdout/stderr 逐字一致。Jarde 默认/all 正文一致，完整方法 source map 的范围、owner 与全部 javap 指令 BCI 核对通过。只允许给 JADX 默认包的 Runner 添加 package 前缀，目标源码原样重编，使用空 classpath/sourcepath 和独立输出类集。

04:36 UTC root 实际执行独立 verify-baseline-luna-v2.py，退出 0，接受 127 个闭合文件，结果为 results/byte-array-return-independent-acceptance-v2.json。其 manifest SHA256 为 113b6eeeedd92d20d2d9f3ad8e6e18fc1998537ca117f35b6a7cdb2ec244f658，inventory SHA256 为 057e751872b807576a8e9635216160132e855e63d14da55c75bdfce2a21fd2e1。执行 argv、时间、退出状态和原始输出在 results/baseline-execution-root-v1 与 results/independent-execution-root-v1。

结论：现有 typed NewArray 恢复已覆盖此窄形状，无需新增机制或实施 change。EM18 仍部分已测，71/612 分母和整单元完成计数不变。未执行的旧 verifier v1 原样保留；v2 修正 original 无 runner_class 及 javap 8 无成员摘要的脚本假设。
