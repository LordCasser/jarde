# 验证记录

## 实现与验收范围

本项恢复当前普通类 `<clinit>()V` 中已证明的 blank static final 简单字段赋值。字段身份、flags、descriptor、唯一名称和 ConstantValue 缺席来自同一 MethodIr 的借用字段头；`field::Shape.simple_static_final` 是写入拼写与名称预占的共同依据。FieldAssign 的可选 receiver 表达简单字段名，其余字段读写继续使用既有路径。没有新增 pass、字段解析器或初始化上提机制。

root 已审读字段证明、名称分配、构造位置、report 停止路径和 emitter。字段索引、claimed 遍历、属性检查及名称扫描均接入既有 Budget/StopReason。失败声明不通过名称猜测进入新路径；局部 `local0`、已占后缀 `local0_2`、debug 名和 free_name 共同避让简单字段名。缺失声明、不匹配 descriptor、非 final、其它 owner、同名歧义、低 IrItems 与取消均有回归。

## 完整类执行

固定输入为 `tests/fixtures/p3-final-static/v8/FinalStaticProbe.class`，1089 bytes、5 个 Code，SHA-256 `2bdb603ff8b3638d48256163fb729a0d5ba34a07a65161221b339ecb7ce59a45`。ignored JDK 测试执行冻结 class 与实际恢复的完整类，两条分支结果一致。

root 使用最终 CLI（SHA-256 `7527b03abc1e487121d204672b52043ca5f3aa1148c6d65b136e36f2bbe1f4aa`）独立重放 FinalOrder、FinalBranch 两分支、FinalLocalCollision 和 FinalSelfRead。五次执行全部为零引用，原样 javac 通过，原 class/JADX/jarde 的字段值、计数与顺序一致。证据在 `../../evidence/java-syntax-2026-09-22/initialization-final/root-after/`。

额外 FinalFailure 审计覆盖正常初始化、第一次调用失败、第二次调用失败，并在同一 JVM 中再次访问类。六行结果三方相等；初次失败的 ExceptionInInitializerError 保持 cause 身份，随后访问得到 NoClassDefFoundError，初始化效果未重做。该 class 为 369 bytes，SHA-256 `57f5007bafe529f8800e3fe94216c3d16b20e957dcea7de7d8a43a47bbb146dc`，完整恢复零引用且 javac 通过。源码、runner、生成脚本和日志在 `initialization-final/failure-after/`。

## Root 检查

| 检查 | 实际结果 |
| --- | --- |
| `cargo test -p jarde-java --locked` | 181 通过：103 unit、32 recovery、46 patterns |
| final-static/declaration-handoff/alias-field/field-increment/new/invocation/eval/recovery-entry | 38 通过，2 ignored 不计入 |
| `p3_final_static -- --ignored` | 1 通过，完整冻结类两分支执行一致 |
| reader 全 fixture 遍历 | 94 class / 579 Code / 75 handler / 236 branch 或 switch target / 8 subroutine，通过 |
| fingerprint | 232 文件；新增 13，既有 219 无变动、无删除；5 项检查通过 |
| `cargo fmt --all --check` | 通过 |
| 全仓 OpenSpec strict | 39 通过、0 失败 |
| 严格 clippy | 仅报告既有 `region.rs:1736 type_complexity`；未加入 allow，不声称门禁全通过 |

命令、完整日志、修前 census 失败和修正后的通过记录归档于 `initialization-final/root-regressions/`。新增 13 个指纹输入来自 bitwise、deferred-value-order、floating-constants；它们的 Rust 红测试已真实运行，仍属于待实现的独立 change，不能把语料冻结解释为语法恢复完成。

接口 FinalInterface 仍因接口初始化的声明形式而无法重编译，已保留独立证据；本项没有将其上提到字段初始化器。全异常出口初始化器、接口/枚举呈现与既存区域 clippy 债务均保持独立，不混入普通类 blank final 写入修复。
