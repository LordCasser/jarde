# CF-16 共用 catch-all：主线可行性验收

root 审阅 `3ae5cba4` 的证据提交；该提交只增加冻结 Java 8 夹具、三方源码/字节码与 OpenSpec 1.1/1.2 的可行性记录，**没有修改 Rust 恢复逻辑，也没有完成 2.x/3.x**。独立以 `javac --release 8 -g:none` 重编 [`SharedFinally.java`](../../../../tests/fixtures/p3-shared-catchall-finally/SharedFinally.java) 及 Runner，得到的 class 与冻结 `v8/SharedFinally.class` 逐字节一致；原 class 经 `java -Xverify:all` 为 `normal:1 / caught:1`。使用固定 JADX checkout `2fb1b16386941660fda07e9017285aec40fcb37f` 的已安装 CLI 重新反编译该 class，源码与冻结 JADX 文件逐字节一致；其完整源码重编后输出 `normal:2 / caught:1`。冻结 Jarde 源码因 `handled` 缺返回而无法重编，不能算作行为等价。

另将清理改为单次 `cleanup()` 调用的 [`SharedFinallyCall.java`](../../../../tests/fixtures/p3-shared-catchall-finally/SharedFinallyCall.java) 独立重编：class 与冻结字节逐字节一致，原类仍为 `normal:1 / caught:1`；冻结 Jarde 源码同样因整方法引用而缺返回。故字段自增的 `iadd` 不是唯一阻碍。审阅 [`README.md`](../../../../tests/fixtures/p3-shared-catchall-finally/README.md) 确认三条异常表 ordinal、两条保存返回和共用 handler 的物理 BCI 已分别列出；现有 Guard 证书仅有一条 row/返回，Region 只为单 row 所有权发证，Builder 的 finally 分支固定空 catches。要恢复此形态，需先形成不可分割的私有多 row/多完成出口证书和两个受限子 Region 的原子所有权，再消费现有 `Try` AST，不能只松开旧边界。

`openspec validate recover-shared-catchall-finally --strict` 通过。CF-16 在 71 项账本中继续是已证差距；本次结论是保留当前安全拒绝并明确下一阶段内部合同，不是语法恢复完成。
