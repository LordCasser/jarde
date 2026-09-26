## 1. 证据与边界

- [ ] 1.1 核对冻结 `package-info.class` 的头、表、包注解和来源，并以 `python3 openspec/evidence/java-syntax-2026-09-27/package-info-basic/replay.py` 复现原/JADX 可编译运行、Jarde 基线失败。
- [ ] 1.2 构造至少一个非标准空接口或不完整证据负例，确认候选不能仅凭简单名/空成员通过；用针对性测试核验明确拒绝和物理事实保留。

## 2. 源码投影

- [ ] 2.1 在现有类级装配中实现完整标准形状和注解可写性的有界证明；用正例与类头、成员、注解、预算/取消负例测试验证拒绝边界。
- [ ] 2.2 原子写出注解在前的 `package p;` 源文件，省去类型头与成员段；用源码与来源断言确认普通类、物理身份和来源未变。

## 3. 三方验收

- [ ] 3.1 用冻结 jar 与完整源码集重放原版、JADX、修后 Jarde 的 `javac --release 8` 和 `java -Xverify:all`，断言三方 `p.Check` 均输出 `true`，并保存稳定日志与修后源码。
- [ ] 3.2 运行相关 `class_source` 测试、`cargo fmt --all -- --check`、`cargo check --locked --workspace`、`openspec validate recover-proved-package-info-source --strict`；清理临时 Cargo 编译残留并更新 EM-04 账本状态。
