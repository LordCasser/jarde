# DT-29 父类字段与物理 setter：主线独立验收

root 在包含 CF-10 主线合入后的 `d09f5dea` 构建 CLI（SHA-256 `b4fbe151eb2a8f10a01ee34002e6bea8339eb84b3aded2dc602842c606cd6f76`），独立执行 `JARDE_CLI=/tmp/jarde-root-cf10-array-target/debug/jarde-cli python3 openspec/evidence/java-syntax-2026-09-27/dt29-reference-cast-audit/replay.py`。固定 JADX HEAD 为 `2fb1b16386941660fda07e9017285aec40fcb37f`；重放后的 `outputs/` 与已提交证据逐字一致。原、JADX、Jarde 三份完整 private-field 类族均以 `javac --release 8 -g:none` 编译、`java -Xverify:all` 输出 `true:false`；接口类族三方仍输出 `runnable:ClassCastException`。完整输入 class SHA、源码和报告见 [`outputs/results.json`](outputs/results.json)。

`B` 同名 `visible` 字段保持 false；Jarde 对物理 `A.visible` 写入输出 `((dt29.PrivateFieldFamily$A) this).visible = arg1`，对私有字段保留 `A.access$002((A)this,arg2)` 的物理调用。`A.access$002` 自身准确恢复 `arg0.hidden = arg1; return arg1;`，没有将跨类 helper 调用误报为内联。独立报告的 source map 将 `B.set` BCI 2/7 和 helper BCI 3/6 对应到这些语句。选中父类 owner/descriptor 错误、accessor 重载歧义、helper 额外写入与方法预算停止均按固定负例拒绝。

root 检查通过：`cargo test -p jarde --lib --locked`（145 项）、`cargo test -p jarde-java --lib --locked`（233 项）、`cargo test -p jarde-java --test p3_patterns --locked`（68 项）、`cargo check --workspace --locked`、`cargo fmt --all -- --check`、`openspec validate recover-private-field-owner-casts --strict`。该验收仅覆盖单 setter 的准确父类公开字段与 private synthetic helper 闭环。

另外将旧 [`combined/inputs/FieldCast.java`](combined/inputs/FieldCast.java) 与同目录 runner/接口源码重新按 Java 8 编译，再用同一 CLI 恢复全部物理类并整体重编：原源码成功，Jarde 在 `FieldCast$B`、`$C`、`$D` 和根类仍有 `@bytecode`，根源码两处缺少返回语句，`javac` 退出 1。组合中的 protected/package-private 字段、跨对象或泛型接收者、字符串拼接等仍需分别定位；DT-29 整单元继续列为已证组合差距。
