## 1. 冻结基线与匿名身份

- [x] 1.1 冻结 `Inner.this` Java 8 fixture、原 class SHA、JADX/Jarde 全量源码及重编/运行日志；`python3 openspec/evidence/java-syntax-2026-09-27/anonymous-inner-this/replay.py` 校验原/JADX 输出及确切 Jarde Java 8 构造器诊断。
- [ ] 1.2 扩展匿名候选证明，使 `EnclosingMethod`、owner 的准确方法身份、匿名 `InnerClasses` self row 与唯一分配 BCI 闭合；用方法身份变化、多点和 owner 范围不完整控制验证拒绝。

## 2. 捕获身份与来源投影

- [ ] 2.1 证明唯一 synthetic-final 外围字段与匿名构造器参数/`UninitializedThis` 写入的一一对应，并核对所有匿名方法读取与额外字段/局部捕获；使用真实 classfile 正例和字段/slot 污染负例验证。
- [ ] 2.2 仅向现有 `QualifiedThis` handoff 提交上述准确读取的物理 BCI/来源，断言方法体产生 `Inner.this` 且 read/field/constructor source-map 锚点一致；缺证据时检查保留 `this.this$0` 及物理来源。
- [ ] 2.3 在唯一 caller 分配点内联完整匿名类体，并只在原子成功投影中省略 `this$0`、匿名构造器和 pre-super 写入；用完整根源码 Java 8 重编验证不能通过移动该写入来消除编译错误。

## 3. 端到端拒绝边界与回归

- [ ] 3.1 覆盖第二分配、方法/类身份冲突、local capture、多个 synthetic 字段、匿名方法 fallback、跨范围引用、预算停止/取消；每个负例断言不发布部分源码投影且物理报告可查。
- [ ] 3.2 将 replay 增加独立 fixed 验证输出，检查匿名源码没有物理 `$1`/`this$0`，与冻结 class 分开重编并在 `-Xverify:all` 下得到 `true\n38\n`；确认 Jarde 不复用 DT-06a 父类实参规则。
- [ ] 3.3 执行匿名接口、具名成员捕获、DT-06a 无捕获父类实参及 Java 8 source-map 回归，确认投影身份和拒绝路径互不改变；运行 `cargo fmt --all -- --check` 与 `openspec validate recover-proved-anonymous-inner-this --strict`。
