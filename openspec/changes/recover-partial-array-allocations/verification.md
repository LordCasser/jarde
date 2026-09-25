# 验证记录

本次只实现部分数组创建的总维数事实，未处理数组重载协变、初始化器或其它独立债务。

## 生产与单测

- `cargo test -p jarde-java --lib decode::tests -- --nocapture`：14/14 通过，包含 `anewarray` 数组组件、`multianewarray` 前缀、零 rank 与超 rank 拒绝。
- `cargo test -p jarde --test p3_partial_array_allocation -- --nocapture`：2 通过，1 个 JDK 对照测试按预期 ignored。
- `cargo test -p jarde --test p3_partial_array_allocation -- --ignored recovered_partial_array_class_matches_original_runtime --nocapture`：JDK 对照通过。
- `cargo test -p jarde --test p3_array_access -- --nocapture`：11/11 通过。
- `cargo test -p jarde --test p3_deferred_value_order -- --nocapture`：2 通过，JDK 对照保持 ignored。

## 69 项核心执行对照

命令：

```text
JARDE_AUDIT_STAGE=core-no-overload \
JARDE_AUDIT_CLI=target/debug/jarde-cli \
python3 openspec/evidence/java-syntax-2026-09-22/numeric-conversions/partial-array-allocation/core-no-overload/run_audit.py
```

结果保存在 `openspec/evidence/java-syntax-2026-09-22/numeric-conversions/partial-array-allocation/core-no-overload/summary.json`：

- class：777 B，69 条运行用例，SHA-256 `487389b770dbfdd226ae9ae3ff87c9d6be9df4e7df01781505f5d2a55e10e8f2`。
- jarde：0 条 `@bytecode`，完整源码 `javac --release 8` 与 `java -Xverify:all` 通过，运行输出与原 class 相同。
- JADX：完整源码编译、`java -Xverify:all` 通过，输出与原 class 相同。
- CLI SHA-256：`7152f4358111c313e0641256f2b118ea233d8ddbdae32d58b92f72e62f377b92`。

## root 独立验收

root 冻结上述 CLI 为 `/tmp/jarde-cli-partial-7152`，复核其 SHA-256，并用 `JARDE_AUDIT_STAGE=core-no-overload/root-7152`、`JARDE_AUDIT_CLI=/tmp/jarde-cli-partial-7152` 独立重放同一审计脚本。独立结果见 `core-no-overload/root-7152/`：777 B class SHA 与69项完全相同；jarde 0 引用、完整源码 `javac --release 8` 成功、`java -Xverify:all` 成功，逐项与原 class/JADX 一致。root 亦审读 `anewarray` 组件 rank+1、`multianewarray` 已分配前缀与 descriptor 总 rank 的界限、AST 类型/写出以及尺寸求值顺序。

永久测试新增真实 Code BCI 的来源断言：`anewarray` 和 `multianewarray` 创建点有直接锚；基本尺寸加载以及有副作用的两个尺寸调用仍可由表达式追溯。默认/all 正文一致。root 复跑永久普通 2/2、JDK 运行对照 1/1、`p3_array_access` 11/11、`p3_array_types` 9/9、`p3_alias_field` 3/3、`p3_local_rewrite` 7/7、`p3_deferred_value_order` 普通 2/2。reader fixture census 100/668/81/236/8 通过，fingerprint 5/5 普通项通过（1 个显式再生项 ignored），259 项输入沿用既有钉值。`cargo fmt --all -- --check`、`git diff --check` 与全部 OpenSpec 50/50 strict 通过。完整的72项原始探针含数组协变下的重载选择，继续作为独立边界；不纳入本项。
