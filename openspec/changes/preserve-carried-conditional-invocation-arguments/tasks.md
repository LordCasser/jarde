## 1. 冻结传递事实

- [x] 1.1 保存单/双条件实参 Java 8 源码、class SHA、javap、原/JADX/Jarde 完整类编译及 4/6 路径执行、当前 BCI 22 引用与 class-source JSON；见[分析](../../evidence/java-syntax-2026-09-25/constructor-conditional-delegation/analysis.md)。
- [x] 1.2 在定向测试或验证记录中列出 BCI 12 → BCI 22 的 SSA 值/栈槽/同值 Phi 与实际前驱，确认首次拒绝点；冻结额外入口、值替换、独立效果、错误调用类型或前导身份至少三种 verifier-valid 控制。

## 2. 复用条件值证明

- [x] 2.1 扩展现有条件值证明，以有界、预算化的真实 SSA/CFG 链识别单一携带值至 `invoke*` 实参；保留直接 join 消费路径，不为构造器新增 Region。
- [x] 2.2 证明每次转接的输入身份、exact predecessor、可观察效果顺序、最终调用 descriptor/实参位置与一次消费；负例整体拒绝，预算/取消无半提交。
- [x] 2.3 Builder 原子登记被传递的条件表达式及来源，使两段条件图按 Java 左到右参数顺序求值；完整双条件构造器 Java 8 重编、六路径 JVM 与原/JADX 一致，单条件四路径保持一致。

## 3. 回归与交付

- [x] 3.1 运行本 change 正/负例、已有条件值 Phi、调用实参、构造器初始化、Region owner/来源与低预算回归；记录每条断言及拒绝/停止状态。
- [x] 3.2 运行格式、适用 Clippy、diff-check、OpenSpec strict；只清理本任务私有 Cargo target，记录共享 corpus 指纹债务但不混入修复。

## 4. Root 独立验收

- [x] 4.1 Root 独立构建 CLI，对冻结原/JADX/Jarde 完整类复编与 4/6 路径 `-Xverify:all`、来源覆盖、控制拒绝和二进制/class SHA 复核；见 [verification-root.md](verification-root.md)。
- [x] 4.2 Root 审读每跳 SSA 值身份、参数求值与效果顺序、原子发布/预算，运行定向回归并清理私有 target，记录裁决与剩余独立债务；见 [verification-root.md](verification-root.md)。
