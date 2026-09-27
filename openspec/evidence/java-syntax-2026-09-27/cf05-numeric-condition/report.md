# CF-05：布尔条件的数值结果与窄类型调用

固定队列项为 [CF-05](../../jadx-feature-inventory-2026-09-27/control-flow.md)。[replay.py](replay.py) 固定 JADX 提交、四个原测试和 `ModVisitor`、`FixTypesVisitor`、`InsnGen` 的 SHA-256。`TestBooleanToInt`、`TestBooleanToByte`、`TestBooleanToLong` 是 Smali 的源码片段断言，不是完整 Java 源码运行证据；`TestCast` 对 byte/short 重载调用有源码断言。这里另构造 Java 8 夹具，把可由 `javac` 生成的条件数值选择、字段分支和重载调用编成真实 class，再比较原 class、JADX、Jarde 的**完整类源码**。这不声称覆盖三个 Smali 原件的全部非 Java 编译输入。

[baseline/summary.json](baseline/summary.json)记录两个隔离类。`ConversionBasic` 覆盖 `boolean ? 1 : 0` 的 int/long/byte/float/double 返回；原 class、固定 JADX、Jarde 完整源码均通过 `javac --release 8 -g:none`、`java -Xverify:all`，10 行数值完全一致。Jarde 已用现有条件值和返回适配路径表达这些简单形状，无需为它们新增机制。

`ConversionCases` 加入 `write(byte)`/`write(short)` 的重载选择、两种窄字段与常量边界。原 class 与固定 JADX 的完整源码重编、验证运行 20 行一致；Jarde 完整类在 `byteField`、`castShort`、`shortField`、`shortConstant` 四方法保留物理引用，因缺少返回而编译失败。`castByte` 已正确选择 `write(B)`，`asInt/asLong/asByte/asFloat/asDouble` 也正确。`javap` 显示 `castShort`/`shortConstant` 的两臂是直接可表示的 int 常量，消费者分别是 `(S)I`；Jarde 当前在调用参数门拒绝 `int` 到 `short`。`byteField`/`shortField` 的字段型一臂与 int 常量一臂先在条件值类型门拒绝，尚未走到调用参数门。不能把这两个阶段混作一个“加 cast”补丁，也不能仅凭 JVM 栈上都是 int 就猜源级 byte/short。

架构上，现有 `build.rs::byte_conditional_argument` 已能在调用点按 byte 形参和**每个叶子本身**证明 byte 字段或 byte 范围内常量后加局部窄化；`short` 同类形状没有对应接受路径。混合字段在更早的 `build_conditional_value` 中因 `Expr::Conditional` 无可证明 Java 结果类型被原子拒绝。下一步应先在既有条件值证书、字段 descriptor 与被调用方法 descriptor 上找齐类型/唯一消费者证据，再决定是局部推广现有数值条件规则，还是需要带目标上下文的最小类型证明；绝不能把任意 int 条件值强转为 short/byte，也不能改变重载目标。JADX 的 `ModVisitor.makeBooleanConvertInsn` 构造带数值 1/0 叶的 ternary，`FixTypesVisitor.fixBooleanUsage` 修正使用处类型；可借鉴其入口，但本项目仍须逐叶证明 Java 窄化合法及物理调用签名。

原夹具起初把两个字段写为 `final`，Jarde 类装配同时输出字段常量初值和构造器赋值，暴露了另一个声明/初始化问题；现行固定夹具改为普通字段，使 CF-05 的四处失败只对应条件值/调用窄化。那项字段问题另列架构债，不纳入 CF-05 实现范围。

重放：

```sh
python3 openspec/evidence/java-syntax-2026-09-27/cf05-numeric-condition/replay.py \
  --jarde /tmp/jarde-root-integration-target/debug/jarde-cli \
  --jadx /Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx \
  --jadx-checkout /Users/lordcasser/workspace/testzone/jadx \
  --out /tmp/jarde-cf05-fixed-replay-20260927-v2
```

`--out` 必须为空目录。脚本当前固定原 class、JADX 完整 `ConversionCases` 及三侧完整 `ConversionBasic` 的成功结果，同时保留 Jarde 的四处原子拒绝作为修前事实；待窄 OpenSpec 实现时再提高完整 `ConversionCases` 的修后门槛。

root 将 CF-04 整数条件尾返回合入主线后再次重放，两类三侧源码 SHA-256 均与 `baseline/summary.json` 相同，四处 CF-05 差距仍独立存在。
