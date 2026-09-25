# Root 独立验收（2026-09-25）

## 结论和边界

已证明的非泛型 `Outer` + 泛型非静态 `Inner<V>` 调用现在可恢复为 `outer.new Inner<>(args)`。这沿用先前非泛型成员关系与调用点 SSA/null/顺序证明，新增的只是目标类自有单变量 `V`、构造器源级 `Signature` 与物理 descriptor 尾部的擦除对齐，以及 AST 的 diamond 拼写。对 `UseObject.make(Outer,int):Object`，Jarde 源码是 `return arg0.new Inner<>((java.lang.Object) java.lang.Integer.valueOf(argument(arg1)));`；原 class 与重编该方法后的 caller 在 `java -Xverify:all` 下均打印 `minimal.Outer$Inner:1`、`null:1`。

这比本地 JADX 1.5.6 在**相同 Object 返回调用**上的实际结果更好：JADX 把它写成不能用 Java 8 编译的 `new Outer.Inner(outer, Integer.valueOf(argument(...)))`，把物理合成外层参数当普通实参。[原/JADX 双调试变体证据](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/non-generic-outer-generic-member/analysis.md)可由其 `replay.py` 重放。JADX 对返回 `Outer.Inner<Integer>` 的邻近 `Use` 则可正确恢复限定构造，不能把该错误泛化到所有泛型成员调用。

## 物理证明与拒绝

Root 复核了 `src/member_inner.rs`：目标与外层各自的 `InnerClasses`、非静态字段和 prologue 首参写入仍先被证明；非泛型外层自身有 `Signature` 则拒绝。泛型分支只接收目标唯一类 `Signature` 所声明的 `V extends Object`、`Object` 父类与无接口、无 type-use 注解，准确公开构造器唯一且其签名为 `(TV;)V`。reader 的类签名擦除证明检查物理类头；方法签名擦除只读目标类已证作用域，并针对从物理 descriptor 剥离已证外层首参后的源级尾部，非泛型旧分支仍为 `generic_diamond=false`。重复、缺失、错变量、错界、错擦除、重载、预算与取消均有定向测试。

[负控证据](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/non-generic-outer-generic-member/invalid-controls/analysis.md)由 `invalid-controls/replay.py` 重放。变更类变量名、构造器签名擦除或 `InnerClasses` outer 的三份 class 均可 `-Xverify:all` 执行，轨迹同原 class；由源码加第二构造器的变体也可执行，调用点物理 descriptor 仍选原构造器。Root 把冻结 caller class 与各目标 jar 合并后，用[验收脚本](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/non-generic-outer-generic-member/after-jarde/replay.py)重新运行 Jarde：原目标完整呈现一条 `new@1`；三份属性错形、第二重载及缺目标均无 `new Inner<>`，该站点记录 `presented=false`、保留 BCI 0 的拒绝。缺目标原 JVM 按预期 `NoClassDefFoundError`，不作为可执行等价负例。

这些错形目前在公开报告中落到一般的 `jre_new_interleaved_effect`（中间 `dup`）拒绝，物理元数据的具体失败原因只在目标证明单测可辨。这不造成错误源码，但诊断归因尚不精确；按架构债务另记，不把报告改造混入本 change。

## 重编、报告与来源

[Root 可重放脚本](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/non-generic-outer-generic-member/after-jarde/replay.py)从已提交 Java 源以 `javac --release 8 -g:none` 重建含 caller 的冻结 jar，以当前 Jarde CLI 的 `class-source --format json` 获取报告，并核对必要、全部、`0..4` BCI 证据选择的正文完全相同。`UseObject.make` 为 Java/structured，`new@1` 记录 `head=0, dup=3, constructor=17, arguments=[5,14]`；源码映射包含限定值读 BCI 4、实参 BCI 14、构造根 BCI 17。预算 `class_headers=2` 时 CLI exit 4，报告 `partial/budget_exceeded` 且无 `new Inner<>`。

复杂的 `UseObject.main` 因现有结构恢复边界为 fallback，因此整份 `UseObject` class-source 不宣称可编译。脚本只将 Jarde 产出的 `make` 方法替换到原 caller 的其它未改变成员中，以**原始 Outer jar**为 classpath 重编并运行，正/null/参数效果轨迹逐行等于原 class。另建仅含 `make` 的 `UseSimple`，其[Jarde 完整类源码](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/non-generic-outer-generic-member/after-jarde/generated/UseSimple.java)对同一原始 Outer jar 可直接 Java 8 重编；独立 Runner 的原/Jarde 输出均为 `minimal.Outer$Inner`、`null`。前者检验复杂实参的运行顺序，后者检验完整 class-source 的可编译闭环。

Jarde CLI SHA-256：`3d9efdc215ebc85d0b0e06b34034f976ad8c26bcb62c2d8c9c9d3239796c1fcb`。固定输入、JADX 源/诊断、各 JVM trace、Jarde JSON 和重编 Java/诊断/trace 均在上述证据目录；[Root SHA 清单](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/non-generic-outer-generic-member/after-jarde/sha256.txt)记录主要输入输出。脚本使用临时 Java 编译目录并清理。

## 门禁与磁盘

Root 使用私有 `CARGO_TARGET_DIR=/tmp/jarde-generic-root-target`。`jarde-java --lib` 180/180、`jarde --lib` 25/25、`jarde --test class_source` 47/47、`p3_new_value` 4/4、`p3_ordinary_new_invokes` 2/2、`d1_evidence_selection` 8/8；最终 `member_inner::tests` 再跑 3/3。`cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-proved-generic-member-call-sites --strict` 均通过。

严格 `cargo clippy -p jarde-java -p jarde --all-targets --locked -- -D warnings` 仍因共享工作树既有 `jarde-java` 库 17 项警告及测试 1 项 `map_clone` 失败，位置在 enumswitch/region/report/build/reuse 等周边模块，见[严格输出](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/non-generic-outer-generic-member/after-jarde/clippy-strict.txt)。本 change 新增的 `src/member_inner.rs` `unnecessary_map_or` 已修正；对上述既有库警告类别作定向允许后，`jarde-java` 与 `jarde` 库 Clippy `-D warnings` 通过，见[定向输出](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/non-generic-outer-generic-member/after-jarde/clippy-filtered.txt)。未把相邻 lint 债务混入本次语法更改。

私有 Cargo target 在验收后由 `cargo clean --target-dir /tmp/jarde-generic-root-target` 移除 9,277 个文件、约 3.7 GiB，目录已不存在；agent 自己的构建 target 另清理约 1.1 GiB。证据目录约 1.1 MiB，项目根目录没有残留 `target`；清理后文件系统约有 81 GiB 可用。
