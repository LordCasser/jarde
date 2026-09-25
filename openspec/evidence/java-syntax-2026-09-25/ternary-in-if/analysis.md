# `if` 条件中的三元表达式：Java 8 独立探针

## 探针与冻结输入

探针参考本机 JADX 的 [`TestTernaryInIf2.java`](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/conditions/TestTernaryInIf2.java)：两个 `String` 字段各自比较，null 分支比较另一对象的字段，非 null 分支调用 `String.equals`；这两个三元条件依次嵌在 `if` 中。独立顶层类在 [`TernaryInIfProbe.java`](TernaryInIfProbe.java)，[`TernaryInIfRunner.java`](TernaryInIfRunner.java) 覆盖两边都为 null、非 null 相等、各字段一边为 null 和字段不相等，共八条路径。

使用 OpenJDK `javac 23.0.1` 执行 `javac --release 8 -g:none -Xlint:-options`。类文件为 major 52，长度 470 bytes，SHA-256 `1d633a6b4b6d1408d6e6039ac81302ec175217bac7bd610da32d77f2c17041d1`；Runner 长度 1118 bytes，SHA-256 `9928d9e1805a3d945332c4a7734533888979c894d19d3c6314025d26dd5ce10b`。完整反汇编见 [`javap.txt`](javap.txt)。原 class 在 `java -Xverify:all` 下八行都符合期望，结果见 [`original-run.txt`](original-run.txt)。

## JADX 对照

使用本机 JADX 1.5.6：`jadx -q --no-imports -d OUT TernaryInIfProbe.class`。原始输出保存在 [`jadx-raw.java`](jadx-raw.java)。为在默认包中编译这个独立类，仅去掉 JADX 为无包 class 虚构的 `package defpackage;`，并把自身参数类型限定名 `defpackage.TernaryInIfProbe` 改回 `TernaryInIfProbe`；结果为 [`jadx.java`](jadx.java)。它经 `javac --release 8 -g:none -Xlint:-options` 编译，Runner 在 `java -Xverify:all` 下的八行与原 class 逐字相同，见 [`jadx-run.txt`](jadx-run.txt)。

此 probe 上 JADX 没有行为错误；它把三元嵌套 `if` 写成等价的嵌套 `if` 与早退，没有还原 `?:`。本地对应实现位于 `jadx-core/src/main/java/jadx/core/dex/visitors/regions/maker/IfRegionMaker.java` 的 `checkForTernaryInCondition` / `mergeTernaryConditions`，以及 `regions/IfRegionVisitor.java` 的 branch ordering；它只在两条路径的支配边界吻合时构造 `IfCondition.ternary`。本 probe 的 javac 控制流形状没有触发这种合并，所以不能以该测试期待的 ternary 行式来判断这个运行样例的正确性。

BCI 62 的两个具体 owner、物理 CFG 边和 join 选择过程见 [region-trace.md](region-trace.md)。

## Jarde 结果与边界

用当前工作树的 `jarde-cli class-source --input TernaryInIfProbe.class --policy single-class --class TernaryInIfProbe --evidence all` 生成 [`jarde.java`](jarde.java) 和 [`jarde-report.json`](jarde-report.json)。构造器为 structured；`bothMatch` 没有恢复出语句，报告以 `jre_region_ownership_overlap` 引用整个方法：BCI 62 在完成的 Region tree 中拥有超过一个 owner。Jarde 的 `syntax_status` 是 `not_java`。对完整类直接执行同样的 `javac --release 8 -g:none -Xlint:-options`，因 `bothMatch` 没有返回语句而失败；错误及退出码保存在 [`jarde-javac.log`](jarde-javac.log) 和 [`jarde-javac.status.txt`](jarde-javac.status.txt)。没有 Jarde 重编 class，因此不能报告它的 Runner 行为。

该失败先于 Boolean conditional 表达式构建。`region.rs::overlapping_owner` 在 completed Region tree 校验发现 BCI 62 重复归属，随后用全方法 fallback 替代结构化树；`Builder::prepare_conditional_regions` 收到的已是 fallback，无法处理其中各个分支。另一个区别是 `javac` 将“条件三元式的 Boolean 结果”直接编译成若干 `ifnonnull`、`ifeq`、跳转和早退，没有栈上 Boolean Phi；现有 `build.rs::prove_conditional_value` / `build_conditional_value` 处理的是一个 `Region::If` 的双直线臂值与唯一 Phi consumer，不能作为把多段条件 CFG 合并成 ternary condition 的现成机制。

Root 另冻结了一个**非 1/0 返回控制**：只把原 class 唯一尾序列 `04 ac 03 ac` 的 BCI 62 `iconst_1` 改为 `iconst_2`，保留 470 字节与所有分支/异常表结构，得 [`TernaryInIfProbe-return2.class`](TernaryInIfProbe-return2.class)，SHA-256 为 `f14ff75f4fcfe128b138d1b5a800a53b6c7e4d15aac96e6925d53adde25e1a0a`。[反汇编](return2-javap.txt)确认 62 为 `iconst_2; ireturn`、64 仍为 `iconst_0; ireturn`。以原 Runner 在 `java -Xverify:all` 下执行成功，前两条原本返回 true 的路径实际变为 false，其余六条仍为 false；逐行记录见 [`original-root-run.txt`](original-root-run.txt)与[`return2-root-run.txt`](return2-root-run.txt)。这是一项合法 class 的拒绝边界：不能只因共享两个终结块就把 BCI 62 猜为 Java `true`。

这说明该形状需要单独处理 Region 分支所有权与条件合并边界；不属于已证明 0/1 返回缩写的修补。上述**实现前基线**的 CLI SHA-256 为 `b66a1735968be4d41e93b61fd0d5ee022d7f0136ea8ab26a4e7384746a57e5d2`，私有 Cargo target `/tmp/jarde-ternary-in-if-target` 已清理。后续实现与三方独立验收见 [`recover-shared-terminal-boolean-returns`](../../../changes/recover-shared-terminal-boolean-returns/verification-root.md)；基线文件保留以供对照。
