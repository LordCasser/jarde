# CF-16 单行仅异常完成 catch-all

固定输入为 `openspec/evidence/java-syntax-2026-09-27/cf16-finally/original/FinallyOnce.class`，SHA-256 `3ad6857285368c95c3176a520300c4abba09084443fe7fc08e8972330e9cedd2`，pinned JADX HEAD `2fb1b16386941660fda07e9017285aec40fcb37f`。`escaping()V` 的物理行是 `[4,15)→14 any`；BCI 14 的 `astore_0` 在范围内，BCI 15–20 的字段增量在范围外，BCI 23–24 重抛入口对象。原 class 由仓库中固定源码以 `javac --release 8 -g` 逐字节重编。

`handled-only/FinallyOnce.java` 只替换了无关 `handled()`，其余源码逐段保留；`replay.sh` 每次重编，并把它与固定原 class 的目标方法所有 BCI/opcode 和异常行逐项比较。它的完整类 `-Xverify:all` 末行仍输出 `state:1`。固定 Jarde 原整类另有 `main()` 恢复拒绝，因此运行三方完整源码采用本目录的最小 `FinallyOnce.java`：保留同一字段、`escaping()` 和 `count()`，把无关入口置于外部 Runner，同样逐项验证目标方法布局。最小类 SHA-256 为 `673b58ac1c3642972264e16fd752567dc99a4ad3ed735b4ec415e83a23f252f0`。固定原类的 `main()` 在 `java -Xverify:all` 下末行输出 `state:1`；固定 JADX 完整类、最小原类、该最小类的 fresh pinned JADX 与 fresh Jarde 完整类配 Runner 均输出 `java.lang.IllegalStateException:state:1`。Jarde 的 `escaping()` 只有一个普通 `catch (java.lang.Throwable)` 和一次字段增量，无 bytecode 引用。

`IdentityOnce` 是身份对照：原类及 Jarde 完整类的 Runner 均以引用比较输出 `true:java.lang.IllegalStateException:same:1`。固定 `FinallyOnce` 的原对象身份由 handler 入口 SSA store、末端 load 和 athrow 的同值证明；它本身没有对外暴露构造出的对象供运行时比较。

七个近邻均由 `PatchNeighbor.java` 从同布局最小类单点生成，经 `java -Xverify:all` 成功运行；Jarde 均未写出错误的 catch/finally。`NeighborRunner` 连续调用两次，使外部 handler 入口在第二次可观察：

| 近邻 | 两次路径结果 | class SHA-256 |
| --- | --- | --- |
| `normal-exit` | `returned:0`、`returned:0` | `c91575837cb9da46273e91b298347f4e9c0cfa4a792e9a33d91bddde05562542` |
| `range-shrunk` | `IllegalStateException:state:1` 两次；缩窄后 `new` 位于保护外 | `ba496efa7e63b05f993e1220fa0ff258acf482fc186251953fd3b40534dae5ac` |
| `range-expanded` | `IllegalStateException:state:1` 两次；字段读被自身范围覆盖 | `b6549848448e1ce588acf57050ccde8ebeaa8b709604acc912b23b4ce18bb57b` |
| `competing-row` | `IllegalStateException:state:1` 两次；具名行先于 catch-all | `92683b67137514044019f82c93f7bb44933d34f293e869c3d38636c6e1395c87` |
| `external-entry` | 首次 `IllegalStateException:state:1`，第二次 `NullPointerException:…:2` | `9160f623ffd44130197227eb15ed754ec7bf8c0e67857db634f4d3cd3be613cb` |
| `wrong-rethrow` | `NullPointerException:…:1` 两次 | `fdcef92d90d7b27341352f52a2385ef94d4d0d0160ffe3b611122273015b54ec` |
| `branch-cleanup` | `IllegalStateException:state:0` 两次 | `e481130a28482a3e86c9fe7224de9c4795d0b3c0361b5ada3873b359802989cc` |

定向 Rust 测试还检查固定方法全部 14 个物理 BCI 的来源、唯一块 owner、普通具名类型和已证 `Throwable` 的区分，以及预算与取消停止时文本和 source map 均为空。重放：`tests/fixtures/p3-exception-only-catchall/replay.sh /path/to/jarde-cli /private/tmp/cf16-exception-replay`。
