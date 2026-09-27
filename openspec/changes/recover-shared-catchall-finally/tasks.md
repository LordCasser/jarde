## 1. 冻结基线与接缝

- [x] 1.1 冻结 `SharedFinallyCall`、Runner、Java 8 class、三方完整源码和 `javap` 三行异常表；原/JADX/Jarde 重编及运行结果见 `tests/fixtures/p3-shared-catchall-finally/README.md`。
- [x] 1.2 审计单出口 Guard/Region/Builder 的缺口，保留安全拒绝；具体验收与阻碍见 `openspec/evidence/java-syntax-2026-09-27/cf16-finally/shared-catchall-feasibility-root-2026-09-27.md`。

## 2. 调用型共享 finally 证书

- [ ] 2.1 为冻结的三行、两返回、三份相同无参静态调用和同异常重抛形成一个私有原子 Guard 证书；以 row 次序/范围/handler、目标差异、额外入口、返回与重抛关系、预算/取消负例验证拒绝。
- [ ] 2.2 在一个 checkpoint 内恢复 try `[4,21)` 与 catch `[26,30)` 有界正文并认领全部三行和副本；用定向测试验证无遗漏、无重叠，失败整候选回滚。
- [ ] 2.3 复用 `StmtKind::Try` 输出具名 catch 与唯一 finally，分别保留两个 saved-return 和 catch 参数定义；验证完整 `handled` 源码无引用，source map 覆盖三份清理指令，证书记账三条异常行。

## 3. 三方行为与回归

- [ ] 3.1 原/JADX/Jarde 完整 `SharedFinallyCall` 源码各自 `javac --release 8 -g:none`、`java -Xverify:all`；断言 Jarde 与原 class 均为 `normal:1 / caught:1`，单列 JADX `normal:2`。
- [ ] 3.2 重放已受证单出口 finally、具名 catch 和扩围异常行负例；运行定向 Rust 测试、`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`git diff --check`、`openspec validate recover-shared-catchall-finally --strict`，清理专用 Cargo target，交 root 独立验收。
