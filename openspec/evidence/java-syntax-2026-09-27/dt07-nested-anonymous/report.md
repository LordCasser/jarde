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


## 实现后回放

使用修复后 `jarde-cli` SHA-256 `c87341a955a89d3439eb0cb795b22a424452da3356b4d1462ac43b800856d74e`，按同一固定输入执行 `replay.py fixed`。原源码、固定 JADX 源码与 Jarde 根级发行源码均通过 `javac --release 8 -g:none` 和 `java -Xverify:all`，输出均为 `1`。Jarde 的五个物理 class-source 查询全部成功；根 JSON 的 `anonymous_interface_projection.state` 为 `projected`，根源码含 `new p.Factory() {` 与嵌套 `new p.Action() {`，未泄漏 `Nested$1` / `Nested$1$1`。发行集合只编译 `Action.java`、`Factory.java`、`Nested.java` 与固定 Runner；物理匿名子类保留在 `fixed/jarde/physical/` 供单独查询，不作为发行源文件编译。原始修前拒绝及物理全集不能重编的摘要保留在 `baseline/summary.json`。修后完整日志、三方源码及摘要见 [fixed/summary.json](fixed/summary.json)。 连续两次固定回放的 `summary.json` 逐字一致。

修后重放命令：

```sh
python3 openspec/evidence/java-syntax-2026-09-27/dt07-nested-anonymous/replay.py fixed \
  --jarde /absolute/path/to/jarde-cli \
  --jadx /absolute/path/to/fixed/jadx \
  --jadx-checkout /absolute/path/to/jadx-checkout \
  --out openspec/evidence/java-syntax-2026-09-27/dt07-nested-anonymous/fixed
```

实现验证还运行了 `cargo test -p jarde --test class_source --locked`（80 项通过）、`cargo check --workspace --locked`、`cargo fmt --all -- --check`、`git diff --check` 和严格 OpenSpec 校验。新增拒绝覆盖重复内层分配、外部 grandchild 构造引用、父实现描述符与 `EnclosingMethod` 不匹配、读取 immediate-parent 捕获字段、不完整内层方法、额外字段/构造效果，以及预算耗尽和取消；所有拒绝均未发布内联表达式。关系范围固定为两级，未实现任意深度递归。
