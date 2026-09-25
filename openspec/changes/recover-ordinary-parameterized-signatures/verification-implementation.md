## 类级投影验证（2026-09-24）

`src/class_source.rs` 从同一物理成员唯一 `Signature` 读取 reader 已解析的结构，并先核对每个参数、返回和物理 `Exceptions` 的擦除。普通参数化分支按结构递归拼写 base、class、array、exact、any、extends、super；每个节点按 `analysis_steps` 计费并轮询取消。类变量、含 `$` 或分段内类、未能定位的 type-use 注解整项拒绝。参数名与槽来自同轮 `GenericReturnCandidate`；无正文成员使用 descriptor 槽名。方法/参数注解、varargs 和普通 `Exceptions` 沿原位置写入。`NoBody` 的 abstract/interface/native 仅在原旗标给出合法分号声明时准入。原独立方法 `RecoveryReport`、物理 descriptor、属性及 body 均未改。

已恢复正文仅接受同轮 Program/SSA 证明的直接参数返回或布尔参数条件合流，且返回来源位置的完整泛型类型与声明结果相同；同轮侧证据会拒绝参数槽写入和其它语句/表达式。任何同类 `Methodref`/`InterfaceMethodref` 引用目标名或相邻重载时，投影保守拒绝并写 `generic_call_binding_unproved` marker。该 marker 保留原物理成员及擦除声明，不能单独视为重编类与原类语义等价：`Boundaries.bodyOverload` 回退为 raw 返回，原方法调用的字节码目标是 `choose(CharSequence)`；当前恢复调用源码保留显式 `(java.lang.CharSequence)` cast，重编后仍绑定这个 Methodref，运行值为 `overload=char-sequence`。其它没有该 cast 的调用可能改绑到 `choose(Object)`；要使这种目标的泛型声明也成功投影，需要独立的调用点静态类型和 Methodref 证明。本项不把拒绝态算作成功恢复。

定向测试：`CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/jarde-ordinary-generic-impl-target cargo test -p jarde --test ordinary_generic_projection --locked`，10/10 通过。覆盖冻结四个 identity 形状及 raw 对照、实例和 abstract/interface/native 无正文、`?`/`? super`、方法与参数注解、varargs/throws、类变量/type-use 拒绝、错误 cast、参数改写、同类重载调用拒绝、低预算/预取消、essential/all 正文及独立报告。实例及 abstract/interface/native 的原类与恢复类还分别以 Java 8 重编，并在 `java -Xverify:all` 下逐项比较 `getGenericParameterTypes/getGenericReturnType`。相邻 `generic_method_projection` 9/9、`generic_method_budget` 4/4、`class_source` 47/47，通过；reader 方法签名 3/3、query 元数据 35/35 通过。

使用新构建 CLI 在隔离目录 `/tmp/jarde-ordinary-impl-3.1` 对 `-g` 与 `-g:none` 两份 subject 各自生成完整 Jarde 类，逐份用 `javac --release 8 -Xlint:-options` 编译该类与同一独立 `ReflectionRunner.java`，再以 `java -Xverify:all` 执行。两种模式的原类与 Jarde 重编类输出完全一致：四个泛型参数/返回的反射名称保持 exact、extends、嵌套和数组结构，raw 方法仍为 raw，普通调用末行均为 `7`。独立 evidence 重放还确认 JADX `-g`/`-g:none` 的完整类经 Java 8 重编与同一 runner 执行后逐行一致。输入原类 SHA-256 分别为 `9bbfb43fb440b893be50dc7988c9180c7c1471b31f3b5e5d4bae650c696f8d3f`、`2da818875518d62fbed477cac53fe28195f9221f52c34fd57452a9db26679fda`；Jarde 完整源码 SHA-256 分别为 `fe7088a24f0470fadd1bc6d8be65bb2e82ca870d0d9522be72b495487957f220`、`ce044e233e357341e85a7237bec62e21814acb22783b71e9401156121e8a5e7a`，CLI 二进制 SHA-256 为 `f173cb2c36a6a5aa2ca22c1e905cfbc7116cdf8e571cab6d95fa1cd5ee074bee`。1.2 负例的最终哈希与 verifier 记录由独立 evidence 验收另记。

严格 Clippy 首先停在 `jarde-java` 既有 `useless_conversion`、`too_many_arguments`、`type_complexity`、`needless_option_as_deref` 告警；仅对这四类及既有 `collapsible_if` 使用命令行 `-A` 后，普通泛型定向 target 的 `-D warnings` Clippy 通过。

## 待验收任务

- 2.3 保持未勾：同轮 Program/SSA 的正文类型来源、错误 cast 和参数槽写入拒绝已有定向测试，但 1.2 负例的完整类重编、真实 Methodref 与独立报告对照尚待冻结 evidence 验收。
- 2.4 保持未勾：`generic_call_binding_unproved` 的同类调用拒绝已有定向测试，`Boundaries` 原类目标和当前重编运行值已核对；1.2 的正反例全套哈希、目标 descriptor 和 `-Xverify:all` 记录尚待独立 evidence 验收。raw 回退在其他无显式 cast 的调用处可能改变重载绑定，不能以此标记宣称语义等价。
- 3.1 保持未勾：本轮已完成原类/Jarde 在 `-g` 与 `-g:none` 下的完整类、独立调用方及反射运行；JADX 的同样结果已由 evidence 代理复跑，但其最终 1.1 记录和哈希尚待落盘与独立审读。
