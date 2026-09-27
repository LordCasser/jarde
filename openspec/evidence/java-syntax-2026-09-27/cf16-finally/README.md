# CF-16 finally 审计（2026-09-27）

Jarde 基线：`1f5c6f13f59309d0eb0b4d6b36baa1b4c5a97913`，分支 `codex/cf16-finally-audit`。固定 JADX checkout `/Users/lordcasser/workspace/testzone/jadx`，HEAD `2fb1b16386941660fda07e9017285aec40fcb37f`，工作区干净。

## 固定测试和实现

| 文件 | SHA-256 | 断言实际覆盖 |
|---|---|---|
| `TestFinally.java` | `7a2b5770db341b0fcb9f618d2cbb7ad3429d98be03ff98c608ffba59f54d63c2` | 检查一个 finally、主要表达式和不出现特定伪变量；没有 `check()` 运行语义。 |
| `TestFinallyExtract.java` | `92d8ecdd30061be26ad6d7026f9ab6a9547e69e5dbb0051068eff2f4dedcb33f` | debug 文本结构断言；`check()` 只执行正常的 `test()` 并断言 result 为 1。no-debug 测试检查 Throwable handler、清理语句和 rethrow 文本。 |
| `TestTryCatchFinally.java` | `22fda761dd9762341665e11491d169109efe56938dbbc2fb38104d0d32571b18` | `check()` 验证正常输入及 catch 路径都返回 true；文本也分别检查 try、catch、finally 与字段写入。另一个测试关闭 finally extraction，检查 cleanup 被复制三次。 |
| `TestTryCatchFinally12.java` | `c2dff99146ac9f301fdc8a343567ac3b468307a32ba64e8b005ea3123a2d6a6d` | 强运行断言：9 种 nested try/catch/finally 与异常类型组合检查顺序化字符串；文本检查三个 finally。关闭 extraction 时检查三份 finally 更新。 |
| `TestTryCatchFinally18.java` | `5c4258a41736ff9c55748ad661e83ddeac61cd2c7049798cfb867f254cad1a28` | Smali/profile 测试，禁用编译；文本检查 catch/finally。Java 8 profile 的 `testJ8` 标为 NotYetImplemented，不能算通过基线。 |
| `MarkFinallyVisitor.java` | `9999700ad951b8a309c786bf8efbb3d5d169b1696b25a2682a584b6d6e6db63b` | 识别异常 handler 中与正常 try 路径重复的 finally 指令；合并前检查 handler/scope/指令模式。 |
| `ProcessTryCatchRegions.java` | `0a382bc7e189742410b7a1b93f90703f60dfd7cc1256e06ddca4da0ccff6225d` | 将 handler region 与保护范围拼成 try/catch/finally 区域。 |
| `RegionGen.java` | `8b80ef618487e38cb09a2bbd81b70de0dd096996a321a0c9dcf55e1dc011d4ee` | 输出 `TryCatchRegion` 的结构。 |

所有 SHA 均从固定 checkout 逐文件核对。

## Java 8 正常/异常路径回放

`input/FinallyOnce.java` 用静态计数器让 finally 的副作用可观察：`handled(false)` 在正常返回时执行清理，`handled(true)` 经过 typed catch 返回，`escaping()` 抛出的异常由外层捕获。原始 class、JADX 完整源码和 Jarde 完整 class-source 分别尝试 Java 8 重编：原始与 JADX 编译成功；Jarde 因其 `handled` 方法整体未恢复而在 generated line 21 报“缺少返回语句”。三方不能构成完整运行对照，不能把 Jarde 的拒绝当作语义通过。

原始 `java -Xverify:all` 输出：

```
normal:1
caught:arg:1
state:1
```

JADX 生成源码可编译、可验证，但输出为：

```
normal:2
caught:arg:1
state:1
```

具体多执行一次发生在正常 return 路径：JADX 源码第 13 行在 try body 保留 `cleanupCount++`，第 20 行又输出 `finally { cleanupCount++; }`。异常 catch 返回路径和外抛路径都计一次。这个样本的主要行为差异是 finally 重复执行，而不是文本格式不同。

`javap -c -v` 的物理异常表与路径：

- `handled(Z)Ljava/lang/String;`：保护 `[4,21)` 的 typed handler BCI 31 为 `IllegalArgumentException`；同范围 catch-all handler BCI 65；catch body `[31,55)` 也转向 catch-all BCI 65。正常路径的 cleanup 在 BCI 21–26，catch-all copy 在 BCI 65–74。
- `escaping()V`：catch-all 表项为 `[4,15) -> 14`；handler BCI 14–24 执行 cleanup 后 rethrow。

Jarde 对 `handled` 是安全拒绝，没有输出错误运行结果；其物理诊断为 `local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice`，拒绝完整方法。对 `escaping` 的诊断位于 BCI 14：`the exceptional path repeats code the normal path also runs — the finally copy javac emits ... this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one finally`。汇编表项和 BCI 保存在输入 class 中，可用 `javap -c -v` 复核。

这是已证的 finally 缺口：JADX 文本测试没有执行计数器等价检查，Jarde 对 duplicated exceptional copy 的结构选择安全拒绝，但一个短小可终止样本已将恢复失败定位到上述物理 handler。独立处理可先针对 catch-all handler 的直线 cleanup：证明所有正常/handler copy 的指令、覆盖范围、handler owner 与效果顺序一致，再折叠成一个 finally；若任一副本不能唯一配对，继续拒绝。这里不实现修复、不新增 OpenSpec。

输入 SHA-256：`5b712ee4532579ae16fefb8f378b1be452dccb128f1687f380f1ee109e7396a1`。完整输入、原始 class、JADX/Jarde 源码、运行结果及 Jarde javac 日志保存在本目录。
