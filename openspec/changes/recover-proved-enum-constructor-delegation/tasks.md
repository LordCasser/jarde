## 1. 前置与三方证据

- [x] 1.1 从仓库外独立重放 [委托构造器证据](../../evidence/java-syntax-2026-09-25/enum-constructor-delegation/analysis.md) 的 `replay.sh`，核对 `-g`/`-g:none` 原/JADX Java 8 重编、`-Xverify:all` 值/效果/反射 `{2,3}`、Jarde javac exit 1 及源码 SHA 清单；2026-09-25 Root 从 `/tmp` 重放通过。
- [x] 1.2 基础 [recover-proved-enum-constants](../recover-proved-enum-constants/verification-root.md) 已由 Root 独立验收；从当前 CLI 对双构造器 `-g`/`-g:none` class 冻结仍拒绝整组投影的报告，见 [本阶段基线](verification-baseline-1.2.md)。
- [x] 1.3 构造委托目标、实参 `0`、隐式 name/ordinal 身份、额外效果、异常边及辅助方法不一致控制；能执行者先过 `java -Xverify:all` 并记录 BCI/SHA，不可验证者标为证明函数级控制，逐例核对未证不消隐。[九个 verifier-valid 三方负例](../../evidence/java-syntax-2026-09-25/enum-constructor-delegation/negative-controls/analysis.md) 均由 `javac --release 8` 或等宽 classfile 改写生成；原/JADX 均通过 `-Xverify:all`，异常 handler 路径实际执行，所有 BCI/class SHA 与 Jarde 基线编译拒绝均已记录。

## 2. 有界双构造器证明与投影

- [x] 2.1 沿基础 class-source 的同次候选/成员运行，证明两个常量分别精确绑定 `(String,int)` 与 `(String,int,int)` 构造器，且前者唯一以入口 `this`、原 name/ordinal、无效果常量 `0` 委托后者；对重复成员、错边/错实参、分支/handler、预算/取消做定向测试，无第二次不计费分析或文本解析。
- [x] 2.2 在现有同次 `recover_for_class_source` 恢复中增加最小构造器 AST 候选交接，不从已发射文本截句；证明终端构造器只剥除已证 `Enum(String,int)` 前缀，余下用户 helper 调用与字段保存完整且有序，并按 BCI/Code 核对候选。将物理 `Signature` 与剥前缀后的 `()V`/`(I)V` 源参数逐位置校验，测试额外效果或漏用均原子拒绝。[Root 独立验收](verification-root-2.2.md)
- [x] 2.3 在完整组证明后原子投影 `ZERO, ONE(1)`、无源参数 `this(0)` 与整数构造器正文；完整类用 `javac --release 8` 重编并以 `java -Xverify:all` 核对值、效果顺序和反射 `{2,3}`，源码不露注入 name/ordinal 或 `Enum` super 调用，也不退化成 `ZERO(0)`。[Root 独立验收](verification-root-2.3.md)
- [x] 2.4 核对默认/完整证据正文相同、原字段/两构造器/辅助方法的 JSON item/outcome/真实来源保留，method-only 不改变；紧预算/取消与 1.3 拒绝控制均无半成品类投影，普通 enum、普通构造器和普通静态初始化回归通过。[Root 独立验收](verification-root-2.4.md)

## 3. Root 独立验收

- [x] 3.1 Root 冻结修后 CLI，从原始两份 class 独立运行原/JADX/Jarde 的 Java 8 完整类编译和 `-Xverify:all` 对照，复验 1.3、来源/预算/取消、相关 reader/facade/jarde-java 回归、fmt、可归因 Clippy、`openspec validate --strict` 与磁盘占用；在 [最终验收](verification-root.md)记录未闭合的其它门禁并清理私有 Cargo target。
