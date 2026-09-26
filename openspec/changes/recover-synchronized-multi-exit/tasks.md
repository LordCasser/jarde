## 1. 冻结完整类与边界

- [x] 1.1 冻结 `synchronized-multi-exit/` 自写 Java 8 完整类646B/4Code、SHA-256 `6658190aa7575f8be5095d71aa3ee07aa453d464666b1d34dea6a962950c45f2` 与 runner；原/JADX 三项 `-Xverify:all` 输出相同，Jarde 六处引用、整类缺return。root 复制到 `/tmp/jarde-sync-multi-root-4sq6hO` 独立重放，summary 逐字节相同。
- [x] 1.2 固定 javac Java 8 三臂额外出口负例：验证 class 可执行并记录退出 BCI；双臂专用证书拒绝其额外出口。证据位于 `evidence/java-syntax-2026-09-22/synchronized-multi-exit/negative-three-arm/`。

## 2. 受已证 handler 限制的区域闭环

- [x] 2.1 在现有 monitor 规则中证明两条正常退出、同一锁、共享重抛 handler、两段完整保护与所有可达路径恰好一次退出；保持单出口回归，三臂额外出口负例拒绝。
- [x] 2.2 用 `MonitorBranches` 局部映射让 Plan/Region 原子认领受保护 CFG 块和已认证 handler；builder 复用条件与返回发射，不做通用 handler 子 walk，并沿用 Facts 预算与取消检查。
- [x] 2.3 两臂在原返回消费点呈现各自值并单次认领生产者；三方完整类 `-Xverify:all` 输出一致，默认/完整 evidence 文本相同。

## 3. 整类执行与主代理验收

- [x] 3.1 完整 Engine/CLI 原样生成并编译/执行 fixture，对照原 class/JADX 三行及 1.2 负样本；`jarde-java` 216 项 lib 测试含单出口 synchronized、资源与 typed catch 回归，未手改生成源码或删除失败方法。
- [x] 3.2 root 独立审查 certified handler 边界、分支归属和拒绝来源，临时重建 CLI 重放整类；`cargo test -p jarde-java --lib --locked` 216 通过、`cargo check --locked --workspace`、fmt、OpenSpec strict 139/139 通过，三方 Java 8 重编验证运行逐字相同，来源 BCI 定向测试通过；全目标编译、reader census、corpus fingerprint 与严格 Clippy 的非 CF-19 阻断另记于 verification.md，临时 Cargo target 已清理。
