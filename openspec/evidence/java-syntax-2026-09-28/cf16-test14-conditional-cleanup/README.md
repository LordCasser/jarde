# CF-16：固定 Test14 的条件 finally 基线

固定 JADX checkout 为 `2fb1b16386941660fda07e9017285aec40fcb37f`。固定测试 `TestTryCatchFinally14.java` 的 `TestCls.test()` 要求唯一 `.doSomething();`、唯一 `finally`、唯一 `.doFinally();` 以及两次 `!= null` 条件。本目录的目标方法正文逐句保留该测试形态，只让被调用的辅助方法可观察：`doSomething()` 可把字段 `t` 置空或抛错，`doFinally()` 可抛另一个错。因此七路径能区分“在 finally 重新读取字段”与“沿用第一次读取的接收者”，以及最后抛错覆盖原抛错。

`javac --release 8 -g` 编出的目标 class major 52，SHA-256 为 `8857b84944f1a0c8ec0d8805d2ba0e7430dfadfda7629b4a14ea4064eb1d4ece`。`test()V` 的异常表只有 `[0,14)→31 any`，正常清理为 BCI 14–28 的字段读取、判空、调用和转移，异常清理为 BCI 31–47 的保存原 Throwable、重新读取字段、判空、调用、加载原值并重抛；BCI 48 返回。这个形态与先前 Test13 的五行分段保护范围不同。

在主线 `df205b29` 构建的 fresh Jarde CLI 与固定 JADX 上运行 [replay-baseline.sh](replay-baseline.sh)：原 class 与固定 JADX 的完整 Java 8 类均重编、`java -Xverify:all` 七路径逐字一致：无字段时无效果；正常为 `body,finally,`；正文把 `t` 置空时只有 `body,`；正文抛错时清理仍执行；清理自身抛错时 `IllegalArgumentException:finally` 覆盖原 `IllegalStateException:body`。Jarde 对目标 `test()V` 保守拒绝，首个缺口在 BCI 0 的 catch-all 形态，后续块由 normal-flow view 报为未覆盖。当前完整类另含辅助 accessor 缺口，不能把整类编译失败单独归因于 `test()`。

现有 `guard::finally_copy` 可以识别保存异常/重抛的外形，但 `prove_finally_copy` 要求正常与异常清理都是直线指令序列、正常端保存返回值且同块返回；它不能证明此处各有一次条件分支并在两个路径上重新读取字段。已有 `Shape::Finally { structured: true }` 只允许受保护正文分支，不表示清理自身的分支。因此下一步需要先设计两个清理 CFG 的有界等价证书、条件/字段读取的 SSA 身份和所有入口出口，再决定是否沿用现有 `Region::Guard`/`finally_body` 和 Builder；不能简单放宽 `cleanup_sequence` 或合并两次字段读取。`FinallyOnce.handled/escaping` 另有独立差距，不与本样本混同。

脚本每次重编目标 class 并逐字比较冻结 class，核 pinned JADX HEAD、三方基线状态和原/JADX 七路径；输出目录由调用方指定。此处不把 CF-16 记作追平。

## 1.1–1.2 最小完整类与 verifier 有效近邻

[`minimal/TestTryCatchFinally14.java`](minimal/TestTryCatchFinally14.java) 保留 `TestCls.test()` 方法及所有字段声明，只将 `doSomething()` 中清空 `owner.t` 的私有跨嵌套类访问改为调用公开辅助方法 `owner.clearT()`，避免 Java 8 编译器在 `TestCls` 上生成不能由当前 Jarde class-source 输出复现的 synthetic accessor。额外的 `doFinallyAlt()` 只供调用目标负例使用。目标 `test()V` 与已冻结输入在 BCI、助记符、符号操作数及唯一异常行上相同：异常表仍只有 `[0,14)→31 any`；正常清理位于 `[14,31)`，handler 清理位于 `[31,48)`，最终返回位于 BCI 48。原冻结目标 class SHA-256 为 `8857b84944f1a0c8ec0d8805d2ba0e7430dfadfda7629b4a14ea4064eb1d4ece`，最小完整 fixture 的目标 class SHA-256 为 `356108017a0d24be26105ea227e93e3089a8832411ba938d3e3ece4193ed35e0`；两者的 `test()V` 代码与异常表逐项一致。

[`replay-fixtures.sh`](replay-fixtures.sh) 使用固定 JADX checkout `2fb1b16386941660fda07e9017285aec40fcb37f`，重编原 fixture、最小 fixture 和完整 JADX 源码，检查 classfile major 52、目标方法的 BCI/opcode/符号操作数与异常表，确认固定 JADX 输出唯一 `finally`、唯一 `.doFinally();` 和两个 `!= null` 条件。原类、最小类和 JADX 完整类均用 `java -Xverify:all` 运行，七条输出逐字一致：

```text
null=ok
normal=body,finally,ok
clear=body,ok
body-throw=body,finally,IllegalStateException:body
clear-throw=body,IllegalStateException:body
finally-throw=body,finally,IllegalArgumentException:finally
both-throw=body,finally,IllegalArgumentException:finally
```

脚本从最小正例生成六个独立 near-miss class，写入本目录 [`negatives/`](negatives/)，并用 `java -Xverify:all` 验证及执行。每个变体只改动有限的目标方法事实；完整七路径输出保存在 [`expected/`](expected/) 并由脚本逐字比对，class SHA-256 固定在 [`negatives/SHA256SUMS`](negatives/SHA256SUMS)：

| 变体 | 修改 | 运行证据（至少一条） |
| --- | --- | --- |
| `field` | 正常清理第一次读取改为另一同类型实例字段 | `normal=body,ok` |
| `call` | 正常清理调用改为 `doFinallyAlt()` | `normal=body,alternate,ok` |
| `predicate` | 正常清理 `ifnull` 极性反转 | `null` 与 `clear` 路径均以 `NullPointerException` 结束 |
| `self-protected` | catch-all 终点扩到正常清理调用之后，使副本落入自身保护范围 | `finally-throw=body,finally,finally,IllegalArgumentException:finally` |
| `external-entry` | 正文判空分支新增到正常清理非空臂的入口 | `null` 路径以 `NullPointerException` 结束 |
| `throwable` | handler 重抛前把原 Throwable 替换为 `null` | `body-throw` 路径以 `NullPointerException` 结束 |

重放命令为 `replay-fixtures.sh [OUTPUT_DIR]`。root 在提交 `0f5aafa9e03ee639f41a2ae0f323e6652487243a` 上独立运行脚本，原/JADX 七路径相同、最小类目标方法同布局、六个负例的 JVM 验证及逐字节 hash 检查全部通过；OpenSpec strict 与 diff check 通过。Jarde 生成结果仍是前述安全拒绝，尚未声称目标方法已恢复；实施验收时仍须核对该拒绝测试及 1.2 近邻拒绝行为。
