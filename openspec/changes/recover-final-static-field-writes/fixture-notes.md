# 夹具记录

本轮只准备 `p3-final-static` 夹具、`tests/p3_final_static.rs` 和本记录，没有修改生产代码、
`tasks.md`、census、fingerprint 或 README 总表。永久语料只有
`tests/fixtures/p3-final-static/v8/FinalStaticProbe.class`；helper 和 runner 只以源文件存在，
由 ignored JDK 测试在临时目录生成其 class。

## 固定 class

使用 `javac 23.0.1` 执行：

```text
javac --release 8 -g:none -d /tmp/jarde-final-static-20260923 \
  FinalStaticSupport.java FinalStaticProbe.java FinalStaticRunner.java
```

提交的 class 为 Java 8（major 52），1089 字节，SHA-256 为
`2bdb603ff8b3638d48256163fb729a0d5ba34a07a65161221b339ecb7ce59a45`。
`javap -v -c` 显示 8 个字段、5 个带 `Code` 的方法：`<init>`、`instanceValue`、
`readAfterAssign`、`snapshot` 和 `<clinit>`；字段 `constantValue` 带 `ConstantValue: int 17`，
`instanceValue` 是构造器写入的实例 final。`<clinit>` 的顺序是 `first`、`second`、无 debug 局部
`seed` 写入 `local0`、`local0_2`、分支两臂写入 `branchValue`，最后读取前三个字段写入
`afterAssign`。

`java -Xverify:all` 在两个独立 JVM 中运行 source-only runner 的原 class；`true` JVM 输出为：

```text
1:2:5:3:4:8:17
first,second,local,local0_2,branch-true
read=8
instance=41
```

独立的 `false` JVM 输出为：

```text
1:2:5:3:4:8:17
first,second,local,local0_2,branch-false
read=8
instance=41
```

原 class 与 JADX 的完整源码、两分支输出及修前 jarde 编译日志由 root 保存在
`openspec/evidence/java-syntax-2026-09-22/initialization-final/final-fixture-before/`；本记录只
复述夹具自身的固定字节和 runner 输出，不把 JADX 生成物放入永久 fixture。

## 修前红证据

使用当前已有的 `target/debug/jarde-cli`（未重建）执行完整类恢复：

```text
target/debug/jarde-cli class-source \
  --input tests/fixtures/p3-final-static/v8/FinalStaticProbe.class \
  --class FinalStaticProbe --policy single-class --format text --evidence all \
  --output <temporary>/recovered.java
```

修前输出把 `<clinit>` 中的写入写成 `FinalStaticProbe.first = ...`、
`FinalStaticProbe.branchValue = ...` 等限定形式，并把无 debug slot 0 命名成 `local0`。
将这份完整输出和 source-only helper/runner 用 `javac --release 8 -g:none` 编译，退出码为 1，
报告 7 个 `无法为 static final 变量 ... 分配值`：`first`、`second`、`local0`、`local0_2`、
`branchValue` 两臂以及 `afterAssign` 各一处。该失败保留为本 change 的修前红证据；测试没有
通过替换或删除生成方法来规避它。

## 测试边界

`tests/p3_final_static.rs` 通过 `Engine::class_source_with_evidence` 访问完整类入口，断言所有
5 个带 `Code` 的方法均为真实恢复结果。它检查 blank static final 的简单左值、同一字段的两个
分支写入、先写后读，以及生成局部不取 `local0` 或 `local0_2`。`RecoveryEvidenceRequest::all()`
必须实际选择全部来源；与 `essential()` 的正文逐字相同，但默认请求的 source map 为空。
完整来源的每个 segment 都必须带非空 BCI 和物理 member；这同时防止来源请求被静默降级。

ignored JDK 测试先编译原始 helper/runner，再把冻结的 `FinalStaticProbe.class` 覆盖回 original
目录，确保原侧运行的是永久 class 字节；recovered 目录则编译完整 Engine 输出。两侧均以
`-Xverify:all` 分别运行 true/false 两个 JVM，并比较计数、顺序、字段值、先赋后读值和实例
final 值。没有执行 Cargo；本轮只执行了 `rustfmt --edition 2024`、`git diff --check`、
`javac`、`java` 和 `javap`。

root随后独立运行 `cargo test --test p3_final_static --locked`，测试成功编译，结果2通过、1个预期失败、1忽略；唯一失败是clinit仍使用限定的blank final写入。完整来源与默认请求同正文的两项均通过，确认失败不来自测试基础设施。日志归档于基线目录`root-red-tests.log`。夹具阶段因此验收，生产尚未实施。

## 实施后验证

实现复用 `MethodIr` 同一份 `ClassFacts.fields` 借用；`field::Plan` 在已认领的字段形状上附带简单 static-final 证明位，按字段名建立一次临时声明索引，并将同一证明同时交给 `NameTable` 预占和 `FieldAssign` 发射。缺失声明、descriptor 不匹配、其它 owner、同名歧义、非 final 与 `ConstantValue` 变体均保留限定左值。静态读取、实例 final、accessor 与构造器仍走原有 receiver 路径。

验证命令及结果：

```text
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 cargo test -p jarde-java --lib
103 passed
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 cargo test -p jarde --test p3_final_static
4 passed (including the six negative declaration variants; one ignored)
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 cargo test -p jarde --test p3_final_static -- --ignored
1 passed
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo build -p jarde-cli
success
```

`field` 的新增证明索引、claimed walk 与名称收集均使用既有 `IrItems` 预算；
`simple_static_final_proof_charges_claimed_work` 在零额度下于真实 BCI 停止，
`simple_static_final_proof_honors_cancellation_before_indexing` 覆盖 proof 与名称收集的预取消。
report 通过既有 `stopped` 路径传播这些停止，未引入独立预算框架。

完整来源的 `<clinit>` 赋值 source map 固定覆盖真实写入 BCI `5,13,23,31,45,56,70`，并覆盖其生产者；默认证据的正文与完整来源逐字相同且不发布 source map。ignored 测试将完整恢复类以 `javac --release 8` 编译，并在两个独立 `java -Xverify:all` JVM 中比较 true/false 分支的字段值、调用次序、计数、先赋后读结果与实例 final 结果；输出与上面的原 class 记录一致。JADX 完整输出及原 class 对照沿用 `openspec/evidence/java-syntax-2026-09-22/initialization-final/final-fixture-before/` 的冻结证据。

`cargo clippy -p jarde-java --lib -- -D warnings` 只停在既有的
`crates/jarde-java/src/region.rs:1736` `clippy::type_complexity`；本项没有修复或放宽该既有问题。
