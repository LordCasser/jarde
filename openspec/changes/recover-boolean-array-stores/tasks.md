## 1. 冻结 JVM 与三方边界

- [x] 1.1 从[只读审计](../../evidence/java-syntax-2026-09-25/boolean-array-lowbit/analysis.md)核对 Java 8 原/补丁类的 SHA、真实 `bastore`/`baload`、`java -Xverify:all` 输出，以及 JADX/Jarde 原样失败阶段；root 独立复核两个 class 的字节差异和 8+4 行 JVM 输出。
- [x] 1.2 将 RawBool、Order 的受控补丁 class、源码、runner 与 expected 纳入永久测试 fixture，记录补丁 descriptor/opcode/Code/hash，复放 Java8 编译、严格验证及修前 Jarde 引用和 JADX javac 失败；生成过程只能写私有临时目录并逐字节核对永久产物。

## 2. 在已有数组消费处分支恢复

- [x] 2.1 在 `array_write` 的已证明 `[Z` + 实际 `bastore` 分支，对已呈现 B/C/S/I 值复用 `integer_low_bit_boolean`，以 0/1/2/3/负数/极值的原 JVM 对照和 byte[] 控制验证，不放宽其他位置。
- [x] 2.2 证明 array/index/value 各一次且保持次序、成功/null/越界/值生产者异常与原 class 相同；测试三个生产者和真实 store BCI 的来源，不因 helper 重复或提前求值。
- [x] 2.3 固定已证明 boolean 的直接写入、未知或不匹配组件/非整数值的保守拒绝，以及 default/all/replay 相同正文和预算/取消的原子停止；运行相邻 boolean field、窄数组写入/数组初始化器测试。

## 3. 完整类执行与独立验收

- [x] 3.1 root 对 RawBool/Order 原 class 和零引用 Jarde 完整类分别 Java 8 重编、`-Xverify:all` 执行，逐行比较值、trace、异常和状态；记录 JADX 原样 javac 阶段，不修补错误源码后续测。
- [x] 3.2 root 审读真实 `bastore` 准入、现有数组组件事实、最低位 AST、来源与执行次序，复跑本案及相邻测试；未知组件/字节数组/非整数拒绝的未解决架构债务只在 roadmap 分案。
- [x] 3.3 root 更新 fixture 索引和 corpus fingerprint，运行 Cargo/JDK、fmt、适当 Clippy、`git diff --check` 和 `openspec validate recover-boolean-array-stores --strict`；记录既存门禁缺口并清理所有私有 Cargo target。
