## 1. 实参源码类型证明

- [x] 1.1 在现有桥调用闭包内逐点证明普通实参是成员方法的直接形参载入、生成源码的非泛型静态类型与桥参数一致；用 `Arg` 正例及 `NarrowArg`/`null`/中间值负例的目标测试验证。
- [x] 1.2 保持捕获接收者、桥调用顺序和异常覆盖证书不变；运行现有 `Outer.super` 桥正负测试，确认显式同型 `other` 仍拒绝。

## 2. 同名候选不可适用性

- [x] 2.1 在完整所选环境中有界检查同名候选的 arity、签名、varargs、异常与严格 class 父链；以 `pick(Arg)`/`pick(NarrowArg)` 的目标测试证明后者不适用，依赖缺失和可适用重载仍拒绝。
- [x] 2.2 只在所有调用点与候选证明完成后发布家族投影和桥隐藏来源；用无部分投影、预算停止及完整引用闭包测试验证。

## 3. 固定对照与回归

- [x] 3.1 将旧 `outer_super_method_bridge_is_not_projected` 断言改为证据支持的正向投影/来源断言，并以受控 `other` 负例覆盖真正需要拒绝的接收者；运行 `member_family_identity` 目标测试。
- [x] 3.2 重放 [EM-12 脚本](../../evidence/java-syntax-2026-09-27/em12-super-dispatch/replay.py)，使普通样本及重载样本的原 class、JADX、Jarde **完整源码**均通过 `javac --release 8` 与 `java -Xverify:all`，分别输出 `20:10:1:3` 和 `number`；更新摘要与验收报告。
- [x] 3.3 运行项目相关 Rust 测试、`cargo fmt --check`、`cargo check --workspace` 及 `openspec validate prove-outer-super-overload-binding --strict`，记录剩余泛型/接口重载边界。
