# EM-11：已证明引用上溯的实施复验

本页接续[实施前审计](report.md)。调用正文仅在选定 Java 8 环境证明引用上溯和目标声明的唯一源级绑定时，于原实参位置写目标类型 cast。当前有两个证明来源：选定输入类头的父类/接口链，以及受 Java 8、parent-first classpath、无覆盖或转换且选定输入无同名定义约束的 `ArrayList → List` 标准库事实。证明按调用 BCI、呈现类型和目标参数类型交付；`cast_argument` 保留生产者来源并附上调用 BCI，不重算实参。

使用 `javac 23.0.1 --release 8 -g:none`、固定 JADX checkout `2fb1b16386941660fda07e9017285aec40fcb37f` 及本分支 Jarde CLI 重放 [replay.py](replay.py)。脚本固定六份 JADX 测试/生产源码哈希、输入源和 class 哈希，以及三方生成源码哈希。可从仓库根复现：

```sh
cargo build -p jarde-cli --target-dir /tmp/jarde-em11-target
python3 openspec/evidence/java-syntax-2026-09-27/em11-overload-binding/replay.py \
  --jadx /Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx \
  --jadx-checkout /Users/lordcasser/workspace/testzone/jadx \
  --jarde /tmp/jarde-em11-target/debug/jarde-cli \
  --out /tmp/jarde-em11-replay
```

[`observed-after/results.json`](observed-after/results.json)和同目录源码、编译及运行日志是这次运行的持久证据。`em11` 的原源码、JADX 完整源码和 Jarde 完整源码都用 `javac --release 8` 编译成功，再以 `java -Xverify:all` 运行成功；三方逐字相同的 stdout SHA-256 为 `d7546a995eff5923c4ebd8ef430d17106f92583b401311674d95f465bc835333`。前 3 行覆盖同类及继承 `ArrayList/List` 重载、显式 `null` 与数组选择；第 4 行 `mid` 是输入类 `HLeaf → HMid` 的目标声明；第 5 行 `created=2` 对应两次各自只构造一次的调用。Jarde 输出可见同类、继承 `List` 和输入 `HMid` 三处目标 cast；三处 cast 的 source map 分别保留生产者 BCI 15/39/4 和调用 BCI 18/42/7，脚本逐一断言。独立 `null`/数组切片三方也均通过，stdout SHA-256 为 `09c80e4ab488735411bc929003f1442d277a93a23f030215bf2f6c88e9b8646c`。

负例完整原源码运行输出 `base\ninterface\n`；分别省去父类和接口定义的选定 JAR 使 Jarde 在各自调用的 BCI 7 拒绝，原因仍为 `no safe reference conversion evidence`，见 [`MissingCalls.java`](observed-after/source/missing-hierarchy-jarde/em11negative/MissingCalls.java) 和 [`MissingInterfaceCalls.java`](observed-after/source/missing-hierarchy-jarde/em11negative/MissingInterfaceCalls.java)。这不是合法完整源码的运行对比，而是缺失层级证据时不发明 cast 的拒绝检查。

`cargo test -p jarde-java --lib` 的 223 个测试及 `cargo test -p jarde --lib` 的 136 个测试均通过；其中既有父类证明测试覆盖预算和取消停止路径，新增 `reference_overload_tests` 对同形参重复声明、不同返回型和 bridge 候选验证拒绝；`openspec validate preserve-proved-reference-overload-binding --strict` 通过。`generic_call_binding_unproved` 仍在集合重载的 Jarde 声明注释中独立可见。固定 JADX `@NotYetImplemented` 的精确构造器文本断言与 Smali 合成访问器不属于这次完整 Java 8 源码验收。
