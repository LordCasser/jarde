# EM-24 数值字面量首片对照

基线固定为 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 与 Jarde CLI SHA-256 `8f1f0012324350e4fc65c7fef4b3e3835102e6d4fda00df280b466270c98e727`。`TestNumbersFormat` 的 AUTO/DECIMAL/HEXADECIMAL 三个断言、`TestSpecialValues` 的特殊值断言和 `TestFloatValue` 的 `0.55f` 断言均实际运行；测试 SHA-256 固定在 [replay.py](replay.py)。本轮 Java 8 夹具只测默认格式的源码语义，未把 JADX 的可配置格式选项误作 Jarde 运行能力。

[input/em24/](input/em24/) 将 byte、short、int、long 的负数及上下界，float/double 的 `0.55`、负零、标准 NaN、正负无穷、最小次正规数、最小正规数与最大值放入六个完整方法。共同 Runner 对整数输出数组值，对浮点输出 `floatToRawIntBits`/`doubleToRawLongBits`。命令如下；[acceptance/summary.json](acceptance/summary.json) 包含输入与输出哈希、三方编译/运行状态，源码和日志也完整保留在该目录。

```sh
python3 openspec/evidence/java-syntax-2026-09-27/em24-numeric-literals/replay.py \
  --jarde /tmp/jarde-cli-accepted-dt31 \
  --jadx /Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx \
  --jadx-checkout /Users/lordcasser/workspace/testzone/jadx \
  --out openspec/evidence/java-syntax-2026-09-27/em24-numeric-literals/acceptance
```

原 class、固定 JADX、Jarde 完整源码均通过 `javac --release 8 -g:none` 与 `java -Xverify:all`；六行数组值和原始浮点位模式完全相同。第二次独立重放的 `summary.json` 及两份反编译源码逐字节相同。JADX 在默认 AUTO 模式把常见边界写为 `Integer.MIN_VALUE`、`Float.MIN_NORMAL` 等符号；Jarde 对整数输出有类型的十进制值，对有限浮点从 class 原始位模式输出可精确回读的十六进制字面量，对标准 NaN 和无穷用有类型的常量除式表示。因此源文本不同，但这些差异在当前夹具中未产生编译或位模式差距。

架构上，Jarde 已把浮点常量保留为原始位模式，在 `ConstantValue::float_presentation`/`double_presentation` 处判定特殊值，再由现有 AST 与 emitter 写精确字面量；无需为了此首片增加新的字面量机制。JADX 的 `TypeGen.literalToString` 与 `StringUtils.formatNumber`/`formatFloat`/`formatDouble` 负责格式化；`formatNumber` 在十六进制负数上按位宽截取并补类型转换，`formatFloat`/`formatDouble` 对 NaN 使用 `isNaN`。后者没有保留 NaN 的符号或 payload，不能照搬作任意 class 常量的等价证明；Jarde 现有非标准 NaN 位模式拒绝边界保留。

本首片未验 `TestNumbersFormat` 将数组连续赋给 `Object` 字段的效果、JADX 十六进制/十进制选项、`TestFloatValue` 的数组元素 `/=` 更新及非标准 NaN 位模式。它只把 EM-24 从未测移到部分已测，不能据此判定整个单元已追平，也没有可证明必须新增机制的编译或运行差距。
