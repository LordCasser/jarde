## 1. 静态组证明

- [ ] 1.1 在现有类源码装配中复核接口初始化路径、同轮候选及普通类字段 flags，提取仅需一次的共享证明；普通类只计静态运行时字段，实例字段不计覆盖。
- [ ] 1.2 用唯一 `<clinit>`、完整字段/方法表、严格写入顺序、字段身份/RHS 读取、无额外效果/异常/ConstantValue/前向读证明整组准入；定向负例覆盖缺失/重复写、额外效果、歧义定义、预算与取消。

## 2. 原子投影与验收

- [ ] 2.1 复用现有 fragment/writer，在全部字段发射成功后一次性附到声明，按证明顺序写出并省去已消费的 `<clinit>`；实例字段、构造器、物理报告与来源不变。
- [ ] 2.2 执行 [EM-06 replay](../../evidence/java-syntax-2026-09-27/em06-field-init/replay.py)，比较原/JADX/Jarde 完整 Java 8 源码编译及 `java -Xverify:all` 运行，确认静态链内联而实例负例不提前；运行接口初始化回归、适用 Rust 测试、workspace check、格式检查及 `openspec validate promote-proved-class-static-field-initializers --strict`，记录剩余形态。
