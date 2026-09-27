# CF-16 主线独立复核

在主线 `31beb853` 的 CF-16 审计合入前，root 用当前保存的 CLI `/tmp/jarde-root-cli-cf08` 对归档原 class 独立重放；生成的 Jarde 完整源码 SHA-256 `00659ba64e863aa59e29bd6cf2570abfa3da23c77adccff40054e92cff44d203` 与审计归档逐字节相同。root 从归档输入重新以 `javac --release 8 -g` 编译原 class，class SHA-256 与归档均为 `3ad6857285368c95c3176a520300c4abba09084443fe7fc08e8972330e9cedd2`。

root 分别重新编译原源码、固定 JADX 完整源码和归档 Jarde 完整源码。前两者 Java 8 编译成功；Jarde 在 `handled` 缺少返回语句处失败，且生成文本明确引用 `@bytecode`。`java -Xverify:all` 原类输出 `normal:1 / caught:arg:1 / state:1`，固定 JADX 输出 `normal:2 / caught:arg:1 / state:1`。因此这里是 Jarde 的安全拒绝和 JADX 的正常路径语义错误，不存在可声称一致的三方运行结果。CF-16 仍为已证差距，后续实现必须以原 class 的副作用次数及 handler 所有权为验收基准。
