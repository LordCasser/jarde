# EM-01 单成员声明验收

在 `origin/main` 85c3296f 之上重放 `replay.py --fixture single`。固定 JADX revision `2fb1b16386941660fda07e9017285aec40fcb37f` 及脚本中的三个测试哈希均吻合；原 class、JADX 与 Jarde 的完整 Java 8 根源码均以 `javac --release 8 -g:none` 重编，并以 `java -Xverify:all` 运行得到 `true:1`。Jarde 根源码恰有一次 `public static abstract class A`，构造器为 `A()`，抽象方法仍为 `test2();`；物理 child 可独立报告。`replay.py --fixture multi` 仍拒绝第二 child 与泛型成员形态，Jarde 根源码重编失败，原 class/JADX 输出仍一致。

`tests/member_family_identity.rs` 的单成员正例、错双向 relation、额外 child、child 字段、类/方法 Signature、注解、未证根使用与低输出预算负例通过；原 `static-member-basic` 构造型路径及家族来源断言通过。家族测试 16 项通过；`cargo check --workspace`、`cargo fmt --all -- --check` 与 `openspec validate assemble-proved-static-member-declaration-only --strict` 通过。

本片未覆盖接口/注解/枚举 child、第二 child、泛型父接口、跨根使用、桥方法、局部/匿名类及任意无 Code 方法。`outer_super_method_bridge_is_not_projected` 在未经改动的 `origin/main` 85c3296f 上独立复跑也失败，属于非静态成员桥路径的既有问题，留给独立任务处理。
