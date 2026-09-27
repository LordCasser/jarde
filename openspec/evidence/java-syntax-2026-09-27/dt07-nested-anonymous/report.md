# DT-07 双层匿名接口审计

基线使用固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f`，Jarde CLI SHA-256 为 `635bac16a53f4ce8edb776656dda212d9e6b277d62fb1d54836bb4b1d484701d`。完整重放命令：

```sh
python3 openspec/evidence/java-syntax-2026-09-27/dt07-nested-anonymous/replay.py baseline \
  --jarde /absolute/path/to/jarde-cli \
  --jadx /absolute/path/to/fixed/jadx \
  --jadx-checkout /absolute/path/to/jadx-checkout \
  --out openspec/evidence/java-syntax-2026-09-27/dt07-nested-anonymous/baseline
```

原输入及 JADX 完整源码均通过 `javac --release 8 -g:none` 和 `java -Xverify:all`，两侧输出 `1`。Jarde 根源码拒绝 `anonymous_interface_child_additional_use`；完整物理源码因内层 child 的 `this$0` 写入位于 `super()` 前而无法 Java 8 重编，因此没有 Jarde 运行结果。第二次基线重放的摘要与 [baseline/summary.json](baseline/summary.json) 逐字一致。

证据与精确边界见 [analysis.md](analysis.md)；OpenSpec 草案为 [inline-proved-nested-anonymous-interfaces](../../../changes/inline-proved-nested-anonymous-interfaces/proposal.md)。该结果只证实 `TestNestedAnonymousClass` 的两层匿名接口返回形态存在可闭合的最小差距。`TestAnonymousClass12` 的匿名类继承、外层字段访问与多层捕获超出本次切片，不能因该夹具通过而推断已追平。
