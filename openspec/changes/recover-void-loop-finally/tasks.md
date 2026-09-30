## 1. 固定证据和有效近邻

- [x] 1.1 重核固定 Test2 class/source SHA、`test(OutputStream):void` 双行异常表、三处正文循环及原 class/原转写/JADX Java-input 的既有行为矩阵；以 `cf16-test2-loop-finally/replay.sh`、`javap -v` 和 `java -Xverify:all` 记录可重放基线。（基线提交 b3a01e24 上 replay.sh 全流程退出 0，拒绝诊断与证据目录提交记录一致；见 verification/README.md）
- [x] 1.2 构造至少五个 verifier 有效近邻，覆盖清理目标/接收者、资源定义重写、handler 自保护扩围、Throwable 重抛改写、正文循环额外入口或出口；逐项运行 `java -Xverify:all` 并记录 class SHA 和变更前 Jarde 的安全拒绝。（六个近邻，neighbors/verify.txt 与 neighbors/baseline/、neighbors/recovery/）

## 2. 双行 void finally 证明

- [x] 2.1 在既有 Guard/Plan 的单正文双副本 finally 中增加独立 void 完成证明，核两行半开区间、自保护绑定、清理位于保护外、最终 return、所有物理块及 canonical 边；以固定类命中和 1.2 的行/CFG 近邻拒绝测试验收，旧完成形态不放宽。（guard.rs `prove_void_loop_finally` + `FinallyCompletion::Void`；回归 crates/jarde-java/tests/p3_void_loop_finally.rs）
- [x] 2.2 以 SSA 证明 BCI 8 的 local 2 同一资源到达两份 `close()`、调用目标相同及 local 12 原 Throwable 重抛；以 1.2 的资源/目标/异常改写近邻、预算和取消测试验收，不能仅比较指令文本。（同上证明的 SSA 链；近邻 different-receiver/different-target/resource-definition-rewritten/throwable-rewritten 与预算/取消测试）

## 3. 有界正文和源码交付

- [x] 3.1 在既有 Region walk 中完整纳入受保护的三处正常循环，并在正常清理前结束；以三个回边、全部受保护块、旧 CF-16 循环/保存返回测试及拒绝路径的 visited 回滚验收。（region.rs Void 路由进 bounded_shared_finally_body + void_loop_finally 行所有权传播；owned 块集回归断言）
- [x] 3.2 在既有 Builder 中证明跨保护段的 local 2 声明并原子写出一次 `try/finally`、一次 `close()`、void 结尾；以完整类 `javac --release 8`、全部目标 BCI 来源、无 fallback、预算/取消无半成品及有效近邻验收。（build.rs Void 完成分支 + void_loop_lead hoist + aaload 元素类型；run-behavior.sh 编译并逐路径对照）

## 4. 对照和主线验收

- [x] 4.1 以 fresh CLI 重建完整 Jarde class-source，和固定原 class、原 Java 8 转写、JADX Java-input 对照零/多类、第一/后续写入失败、关闭失败及双重失败；所有版本运行 `java -Xverify:all`，结果、清理次数及异常身份逐路径一致，记录输出 SHA。（verification/fixed.run.txt 等四份逐字节一致，behavior-sha256.txt）
- [x] 4.2 重放 1.2 的有效近邻和已验收的 Test3/4/5/11/13/EmptyFinally；运行 `cargo test -p jarde-java --tests --locked`、workspace check、fmt、OpenSpec strict、CI 同款 Clippy、diff check，记录结果并清理专用 Cargo target。（verification/ 各日志；216/216 strict；484/0 测试；replay 与验证的临时 target 均由脚本清理）
- [x] 4.3 root 独立复核双行表、三循环/SSA/异常边、来源和四方行为，更新 CF-16 账本；只标记固定 Test2 JVM 切片，不外推 DX/DEX、Test5/9 或其他编译 profile。（root 于合并主线 823b4bd1 复核：固定类 0 not-recovered、三循环+唯一 finally+一次 close、源码 SHA 59d5c8c… 与实施记录一致、497 测试全绿、fmt/clippy 新码零告警、openspec strict 通过）
