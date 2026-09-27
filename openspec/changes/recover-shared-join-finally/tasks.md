## 1. 固定测试基线

- [x] 1.1 固定 `TestTryCatchFinally.TestCls` 的 Java 8 原 class、JADX 默认/`--no-finally` 和当前 Jarde 完整源码，记录文件 SHA、三行异常表、三份清理、两条正常 goto、BCI 39 join、正常/异常/`check()` 运行及首个拒绝；保存可重放 fixture 与 verifier 有效的值不等价负例，见 `cf16-finally/fixed-test-triage-2026-09-28.md`。root 另用 pinned checkout binary 独立运行 `replay-baseline.sh`，三个主 SHA 与归档一致，脚本成功且 Cargo target 已清理。

## 2. 共用汇合点的共享证书

- [x] 2.1 在既有共享 finally 候选内证明新的互斥正常完成形态：精确三行异常表、两条有界正文到清理前、两个 goto 的唯一 BCI 39 join、catch-all 原 throwable 重抛和全部正常/异常边；旧双 saved return 证明不放宽。以 BCI/edge 断言及 verifier 有效的行或跳转变体核验拒绝。
- [x] 2.2 证明 BCI 10–12、23–25、32–34 的相同当前实例 `Z` 字段写入：完整字段身份、`this` receiver、常量 SSA 生产/消费、无中途入口与外部消费者；以 BCI 24 改为 `iconst_0` 的 verifier 有效变体断言不能合并。
- [x] 2.3 将 `[5,10)` 与 `[18,23)` 交给现有有界 Region 恢复，共享证书只拥有三份清理与 handler，保留 `[0,5)` 前置效果并在 `Plan::join` 续写 BCI 39–43；断言全方法唯一 region owner、无缺失/重复 BCI。
- [x] 2.4 在已有 Builder checkpoint 中输出唯一实例字段 `finally` 赋值、具名 catch 与独立的后续返回；来源图覆盖三份副本、两个 goto、三条异常行和异常重抛。若任何 Builder 部分无法呈现，整体回滚并给物理诊断。

## 3. 完整类与回归验收

- [x] 3.1 原/JADX/Jarde **完整 Java 8 类源码**分别 `javac --release 8 -g:none`、`java -Xverify:all`，正常、具名 catch 和 `check()` 逐项与原 class 一致；再用 verifier 有效且保持 `test(Object)` 三行/三副本布局的抛出 `Error` 变体触发 catch-all，核对原异常身份、字段赋值一次及重抛。固定 JADX `--no-finally` 控制模式记录三份源码赋值，不要求 Jarde 提供该开关。
- [x] 3.2 运行已验收的 `SharedFinallyCall`、`SharedFinally` 静态字段型、单出口 finally 与 typed catch 正反例；验证预算/取消原子性。`cargo test -p jarde-java --tests --locked`、`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`openspec validate recover-shared-join-finally --strict` 与 `git diff --check` 通过，清理专用 Cargo target。
- [x] 3.3 root 独立复放完整三方类、负例、来源和旧形态回归，记录验收后再更新 CF-16 清单；未覆盖的 `FinallyOnce` 及其他固定测试保持未追平。
