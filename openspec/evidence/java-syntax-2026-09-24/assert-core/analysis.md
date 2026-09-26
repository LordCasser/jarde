# 断言合成开关的最小三方对照

本文记录 2026-09-24 的历史基线；`recover-conditional-values` 合入后的当前结果和完整重放见 [2026-09-26 基线](replay-2026-09-26.md)。下文的 Jarde 编译失败不代表当前主干。

`AssertCore.java` 用 javac 23.0.1 的 `--release 8 -g` 编译为 SHA-256 `71753fbec0b66a6511bbb2b66855c50589060b729bc963cb6cd78377b75f4aaf` 的 class；`AssertRunner.java` 只用于执行，不交给反编译器。样本把 guard 和 detail 调用分别计数，方法本身没有复杂 boolean join、异常处理或资源结构。原 class 经 `java -Xverify:all -ea` 输出 `1|0;bad|2|1`，经 `-da` 输出 `0|0;0|0`。`replay.py` 在独立输出目录重编、核对两个 class SHA，并对原/JADX/Jarde 完整类分别编译和执行。

JADX 1.5.6 的完整类源码编译成功，两种 JVM 模式输出均与原类相同，但把源 `assert guard(ok) : detail();` 展开成显式 `if`/`throw`。它的 `/* synthetic */` 只是注释：重编后的 `$assertionsDisabled` 字段 flags 为 `0x0018`，原字段为 `0x1018`。Jarde 的完整类源码也完整写出了 `check` 中的条件、guard 门控和 `AssertionError((Object) detail())`，其他普通方法可编译；唯独 `<clinit>` 把 `desiredAssertionStatus()` 的双臂值汇合留成空 `if` 与 BCI 13 引用，合成 final 字段没有赋值，javac 只报这一项错误。因此 Jarde 无可执行的重编 class，不能声称它通过两种断言模式。

`patch_wrong_owner.py` 只把原 class 的 `<clinit>` BCI 0 的 `ldc #8` 改成 `ldc #35`，即把 `AssertCore.class.desiredAssertionStatus()` 改为 `StringBuilder.class.desiredAssertionStatus()`；Code 的长度、所有 assert 方法和字段 flags 不变。补丁 class SHA-256 `af3c1190da44c938138da327a6232220acd3b8e8161808e2d9d517f11a4e71dc`。同时启用 `AssertCore` 的断言、关闭 `StringBuilder` 的断言时，原 class 输出 `1|0;bad|2|1`，补丁 class 经验证器输出 `0|0;0|0`。JADX 对补丁类准确写出 `StringBuilder.class`，完整源码重编验证后同样输出 `0|0;0|0`；Jarde 保留该类字面量，却仍因 final 字段未赋值而编译失败。若仅凭合成字段名和方法里的 `if`/`throw` 改写成源级 `assert`，这个补丁就会被误编译为读取当前类断言状态，改变行为。

实现归属：当时的**编译失败**属于 `recover-conditional-values` 的双臂栈 Phi→`putstatic Z` 问题；该 change 后来已修复并验收。源级 `assert` 语法及重新生成 `ACC_SYNTHETIC` 仍需另行证明合成字段、唯一 `<clinit>` 写入、所有读取、当前类 status 来源及条件/detail 求值门控的整组关系；该跨成员投影不属于条件值 change，也不能由它的成功自动推出。

冻结文件包括两份 class、原源码、当时的 JADX/Jarde 完整主体源码、两份 `javap`、补丁与 `replay.py`。当时两个 Jarde 类均 `javac_exit=1`，两个 JADX 类均 `javac_exit=0`；当前重放的两个 Jarde 类均 `javac_exit=0`，行为详见 [新基线](replay-2026-09-26.md)。
