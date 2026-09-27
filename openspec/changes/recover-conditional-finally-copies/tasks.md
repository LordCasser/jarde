## 1. 冻结目标与可运行近邻

- [x] 1.1 在已冻结 Test14 基线上制作只改辅助方法的最小完整类，逐 BCI/opcode/异常行比较 `test()V`，原 class 与 pinned JADX 完整 Java 8 类 `java -Xverify:all` 七路径一致且不受 synthetic accessor 编译缺口影响。
- [x] 1.2 制作 verifier 有效的字段/调用/谓词不一致、自保护扩围、外部副本入口及原异常重抛改变近邻；记录每个原 class 行为或验证状态，确保拒绝测试有真实输入。

## 2. 有界双副本证明

- [ ] 2.1 为唯一 catch-all `[0,14)→31`、两份一次判空/可选调用清理和 void 正常出口建立私有 Guard 证书；定向测试逐 BCI 核两次字段读取、相同解析目标、空/非空臂及单次调用。
- [ ] 2.2 证明两份清理 CFG 的边与块所有权、无自保护和无外部入口，以及 handler 保存原 Throwable 并在清理正常完成后原值重抛；1.2 的所有有效近邻均不得获证，预算/取消要返回停止状态。

## 3. Region/Builder 的唯一结构化 finally

- [ ] 3.1 使用已有 FINALLY pass 和有界 Region 子走访恢复受保护正文与正常条件清理，检查双副本物理来源无重叠/遗漏；目标方法必须形成完整 `Try` 与 `If`，失败原子回退。
- [ ] 3.2 Builder 在同一 checkpoint 中输出唯一 `if (t != null) t.doFinally()` finally，保留每份清理内部的两次字段读取及异常副本来源；检查目标完整源码无引用、所有物理 BCI 可追溯，停止时不发布半成品。

## 4. 三方验收与回归

- [ ] 4.1 fresh CLI 重编原/JADX/Jarde 三份完整 Java 8 类，`java -Xverify:all` 七路径逐字比较，特别核字段置空及“正文/清理均抛错”最终传播清理异常；所有有效近邻安全拒绝。
- [ ] 4.2 运行既有直线 finally、Test12/Test13、共享 finally、具名 catch、TWR/monitor 回归，`cargo test -p jarde-java --tests --locked`、`cargo check --workspace --locked`、`cargo fmt --all -- --check`、OpenSpec strict 与 diff check；清理专用 Cargo target。
- [ ] 4.3 root 独立审阅配对 CFG/SSA 证书、Region/Builder 原子性、负例与三方运行，写主线验收记录并更新 CF-16 清单；仅勾销固定 Test14 切片。
