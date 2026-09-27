# EM-01 单成员主线独立验收

root 审阅并合入静态抽象成员声明实现后，保留原有“静态无捕获目标必须获证”的状态类型，避免为本切片新增可空目标；将本切片 child 的物理 class flags 限定为 Java 8 `ACC_PUBLIC|ACC_SUPER|ACC_ABSTRACT`。重新构建 CLI（SHA-256 `2602d9b1e41dbc1e0eb21012b799eeaac7563077610c57d97139535ff4737593`），独立运行固定 [replay.py](replay.py) 的 `single` 与 `multi` 两组。固定 JADX 提交及三项测试哈希一致，完整输出和日志在 [root-replay/](root-replay/)。

`single` 的原 class、JADX、Jarde 完整 Java 8 根源码均重编、`java -Xverify:all` 输出 `true:1`。Jarde 根内只声明一次 `public static abstract class A`，写 `A()` 及 `abstract int test2();`；物理 child 保持独立报告。错误双向关系、第二 child、额外字段/Signature/注解、未证根使用与低输出预算拒绝，原 `static-member-basic` 构造型路径不回归。`multi` 的原 class 与 JADX 重编运行通过；Jarde 仍未装配 `Shape.I`、`Shape.A`、`Generic.A`，完整根源码编译失败。这两个扩展形态继续作为 EM-01 独立缺口，不借单 child 成果宣称整个单元追平。

主线定向声明测试及两个原静态成员测试通过，`class_source` 85 项、workspace check、fmt 和 OpenSpec strict 通过。`member_family_identity` 全文件为 16/17：唯一 `outer_super_method_bridge_is_not_projected` 在未修改的主线基线已同样失败，属于已单列的[非静态成员桥债务](../../jadx-feature-inventory-2026-09-27/outer-super-bridge-baseline-debt.md)，不并入本切片。
