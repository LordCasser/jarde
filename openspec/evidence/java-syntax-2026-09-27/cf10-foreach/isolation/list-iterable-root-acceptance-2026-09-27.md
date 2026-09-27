# CF-10 独立调用上溯：主线验收

主线合入 `d0ab7edb` 后，root 检查生产差异：仅在 Java release 8、实参已呈现类型精确为 `java.util.List`、目标 descriptor 精确为 `java.lang.Iterable` 时，`invocation_argument` 走既有单次 `cast_argument` 路径。其它数组上溯、引用重载和未知用户类型仍走原有证据门；没有改动 foreach 识别或循环 Region。

root 独立构建当前主线 CLI（SHA-256 `e2c9e66b56ca335e6460cbfece233207eba8381d91c4eed472856893fbe34ab6`），重放固定 `ListToIterable.class`。所得完整 Jarde 源 SHA-256 为 `16da9f57b4555c3eebe8f5f64443764baec74137dbce4498db2d47a114aa86b7`，与实施分支归档相同；`main` 保留 `consume((java.lang.Iterable) java.util.Arrays.asList(...))`，没有 `@bytecode`。原始、固定 JADX 与 Jarde 完整源码分别经 `javac --release 8 -g` 重编、`java -Xverify:all` 验证运行，三者均只输出一行 `called`。定向 `array_invocation_widening` 6/6 通过，包括 BCI 19 实参 producer、BCI 22 调用来源、单次求值及预算/取消原子性。相邻 `p3_iterable_foreach` 5/5、`p3_loop_test_values` 4/4、`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`openspec validate preserve-list-iterable-invocation-widening --strict` 与 `git diff --check` 均通过；独立 Cargo target 已清理 1.7 GiB。

同一 CLI 重放组合 `ForeachCases` 时，`main` 已保留 `join((java.lang.Iterable) ...)` 且没有引用回退；`everyOther` 的步长索引循环仍引用。这确认两个 CF-10 缺口彼此独立，数组长度条件已另列 [窄 OpenSpec](../../../../changes/recover-arraylength-loop-condition/)，本轮不把 CF-10 整单元标为追平。
