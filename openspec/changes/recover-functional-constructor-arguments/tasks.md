## 1. 证明与实现

- [ ] 1.1 在构造参数物理依赖扫描中接纳动态调用，复用后续 lambda 证明；通过 Runnable/Comparator/Callable/原始 SAM/静态引用/多实参双腿完整恢复测试。
- [ ] 1.2 对含动态实参的构造闭合区间及 sole consumer 统一检查 handler 覆盖；以 allocation、factory、constructor、consumer 边界注入单测证明每个不一致均拒绝。

## 2. 反例与行为

- [ ] 2.1 冻结双腿 class/source，增加全类呈现以及未知 bootstrap、无关动态值、弃置动态实参、nullable bound-ref 的拒绝与 origin/bytecode 测试，既有 CST 等反例仍通过。
- [ ] 2.2 ignored replay 测试将整类输出剥离注释后编译，外部 Driver 在两个 javac 腿 -Xverify:all 与原 class 同输出；保留既有 legacy LG 锚及其他方法的对照。

## 3. 主线验收

- [ ] 3.1 root 独立审查 diff，复测源码/JADX/Jarde 全类行为，记录实际改进、泛型/绑定接收者保守边界及附近构造回归。
- [ ] 3.2 fmt、CI 同口径 clippy、两固定 seed workspace 测试、strict OpenSpec 校验及必要 corpus fingerprint 通过；证据落盘后提交推送、清理临时 worktree 与 Rust 构建残留，更新 handoff。
