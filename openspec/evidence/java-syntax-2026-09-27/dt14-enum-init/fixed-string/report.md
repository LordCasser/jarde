# DT-14 标量 String enum 实参回放

基线：`bf604701`。实现只接受准确 `(Ljava/lang/String;ILjava/lang/String;)V` 私有构造器、对应单 String Signature、唯一同 owner String 实例字段写入，以及完整有序常量组。字面量沿用 DT-11 的 ASCII 解码与 Java 转义；条件表达式沿用 int 三元证明的同类 `()Z` 调用、`ifeq/ifne` 极性、双 arm、单次 `goto` 和构造器汇合。String 标量与 `String...` 数组证书独立。

运行 `JARDE_CLI=target/debug/jarde-cli python3 openspec/evidence/java-syntax-2026-09-27/dt14-enum-init/replay.py fixed-string`。固定 JADX checkout 为 `2fb1b16386941660fda07e9017285aec40fcb37f`，使用 `javac --release 8` 与 `java -Xverify:all` 重编、运行完整源码及 runner。逐项输入 class hash、JADX/Jarde 源码、javap 和编译/运行结果见本目录。

| 输入 | 原 class、JADX、Jarde 的一致输出 |
| --- | --- |
| `StringTernaryInit` | `string-ternary=A:B:2` |
| `AlternatingStringInit` | `alternating=1:B:3:3` |
| int 三元控制 `TernaryInit` | `ternary=1:20:1:3` |
| 字面量控制 `LiteralInit` | `literal=1:20` |
| 无实参控制 `PlainInit` | `plain=2:true` |
| 自定义初始化控制 `CustomInit` | `map=2:true:true` |

`cargo test --lib --locked`：148/148 通过。String 专用测试覆盖标量字面量、正反极性及计数、错 descriptor/字段/构造器效果、额外 arm 调用、嵌套 branch、错 branch/goto 目标、错消费者、非 ASCII、第二常量失败、handler、预算/取消原子停止；同时检查已投影类的物理字段、构造器、`<clinit>` 和方法源码映射仍可查询。原有 int、`String...` 与普通 enum 测试同轮通过。

`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`git diff --check`、`openspec validate recover-string-ternary-enum-arguments --strict` 均通过。完整 `cargo test --locked` 的一个既存集成测试目标 `tests/bulk_recovery_cancel.rs` 编译失败：它引用仓库当前不存在的 `BulkProbe` 和 `BulkRecoveryRequest::with_probe`；该问题与本次 enum 改动无关，未混入修复。
