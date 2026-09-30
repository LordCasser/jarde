## 1. 基线与负例

- [ ] 1.1 重放固定 T3/T1/C4：核 fixture SHA、主线 `jre_guard_unexplained_row` 基线与实验捕获（放宽后 `[38]` 未覆盖）；记录可重放基线。
- [ ] 1.2 构造并冻结至少五个 verifier 有效负例：catch-all 包围行（保持 Unexplained）、包围行只覆盖部分跨度、handler 块与 claim 交叠、handler 体含分支/循环（保持拒绝）、双 catch 子句（保持拒绝）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 容忍与子句呈现

- [ ] 2.1 `enclosing_clauses` 全跨度具名行分支（design 决策 1），守卫齐备；T3 两形 guard 通过；部分覆盖分支与 catch-all 拒绝逐字不变。
- [ ] 2.2 Plan 携带包围子句、Builder 发射 `catch (E e)` 与 handler 体呈现、覆盖账本记入（design 决策 2）；T3/T1.twrNamed/C4.twrNamed 完整恢复（叠加形状含 17a）；预算/取消原子回滚。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（TWR 家族、Catches 既有路径、17a 切片零回退）、fmt、CI 同款 Clippy 新码零新增、`openspec validate --all --strict`、diff check；磁盘纪律同前。
- [ ] 3.2 T3/T1/C4 三方对照：原 class/固定 JADX Java-input/Jarde `javac --release 8` 重编，`java -Xverify:all` 正常路径与注入异常路径（ISE 被 catch、清理异常传播、suppression 保留）逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核容忍守卫、子句呈现与覆盖账本，更新 CF-17 清单与巡查账本。
