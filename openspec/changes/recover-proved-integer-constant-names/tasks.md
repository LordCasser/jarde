## 1. 同次字段候选

- [ ] 1.1 从类字段已有 typed `ConstantValue` 读取中建立唯一、可写的同类 `static final int` 值→字段候选；以重复值、错 flags/descriptor、无属性、不可写名和截断字段表负例验证均不选别名。
- [ ] 1.2 仅在完整类源码恢复中把候选与同次已恢复方法 AST 关联，方法级恢复保持数字；以 CF-12 类级/单方法输出和低预算测试验证边界。

## 2. 有界源码投影

- [ ] 2.1 在整数 switch 的已证 key 与该 case 直接 `return` 的整数叶上匹配唯一候选，检查词法名称冲突并用 AST/现有发射器原子写出字段名；以 `case LOW:`、`return HIGH;`、原 key/返回 BCI 来源及冲突负例验证。
- [ ] 2.2 保持 String/enum/char switch、非直接返回、同值歧义和未恢复方法现有输出；以定向回归及预算/取消测试验证不发布半份投影。

## 3. 三方验收

- [ ] 3.1 重放 `cf12-integer-switch` 固定 class、JADX 与 Jarde 完整 Java 8 源码，11 行 `-Xverify:all` 输出逐行一致，并保存修后源码、CLI SHA 和正反例证据。
- [ ] 3.2 运行相关 switch/常量回归、`cargo fmt --all -- --check`、适用 crate check、`git diff --check` 与 `openspec validate recover-proved-integer-constant-names --strict`；记录结果并清理独立 Cargo target。
