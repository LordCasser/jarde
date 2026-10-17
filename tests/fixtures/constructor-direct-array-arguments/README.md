# 直接数组构造实参输入

`javac8/classes` 与 `javac23/classes` 来自完整 DirectArrayConstructorArguments.java，分别使用固定 Corretto8 `-source 8 -target 8 -g:none` 和 OpenJDK23 `--release 8 -g:none`。两者均明确使用空 classpath/sourcepath，包含所有嵌套类。命令、工具 SHA、全部源码/字节码和 raw 见 OpenSpec `recover-direct-array-constructor-arguments/results/direct-array-inputs-root-v3`。fixture-pins.json 固定每份 class SHA256；标准 corpus fingerprint 另核全部文件。

这是实现前的完整诊断类，源码内 String 特判说明属于旧基线假设。统一数组证明现以真实事实判定这些方法，不以其旧注释作为拒绝依据。twoArrayArguments 还包含一个独立的 AssertionError/StringBuilder 构造拒绝；整方法/整类的真实失败必须保留，不能从反编译结果删方法凑成正例。独立完整正向家族 CompleteDirectArrayArguments 的输入、driver 和14case oracle另存于该 change 的 complete-array-originals-root-v1。

`tf3-debug` 与 `tf3-no-debug` 为未改写 JADX TestFinally3.java 内部 TestCls 的真实 class，原源码来自 checkout2fb1b16386941660fda07e9017285aec40fcb37f，SHA0671606e6d7fee3aa1c8b8430758764eff60dfa00d4f0dc144006e0eb6808130。用 javac23 `-source 1.8 -target 1.8 -g`/`-g:none`，classpath 是真实测试 harness；该 harness major55，未声称 javac8 编译了它。完整源码、owner class、精确 CP pins、命令与 raw 见 tf3-original-inputs-root-v1。两份目标均保留原七方法，所有 Code/异常表相同。

这些是输入来源证明，不是反编译正确性声明；实际实现状态及完整源重编/隔离运行验收以对应 change 的 results 索引和 tasks 为准。

`same-owner` 两份完整类来自 SameOwnerArrayArguments.java：外层与两个数组元素使用同 owner 不同 receiver 的构造器，核 head0→ctor30、head10→ctor15、head21→ctor26。原编译与固定 identity/三次构造 oracle 见 results/same-owner-originals-root-v1。

`reverse-order` 保留两版 javac 的 natural/swapped 四份完整类。重写器只在无分支、无异常表和 Code 子属性的方法中于构造器前插入一条合法 SWAP，并调整 Code 长度；JVM8/23 验证后的原 oracle 为 trace12，natural 参数长度1/2，swapped为2/1。原输入与真实 raw 见 results/reverse-originals-root-v2。拒绝位置按实际首拒记录，不声称一定首拒在 SWAP 指令。
