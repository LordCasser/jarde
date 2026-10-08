# Root 独立验收

与构造实参片共同验收；完整过程和结果见 [根验收记录](../recover-functional-constructor-arguments/verification-root.md)，原 class/JADX/旧 Jarde 差异见 [冻结证据](../../evidence/java-syntax-2026-10-08/bound-reference-creation-timing/README.md)。

两套 JVM 的无 check 原 class 均创建成功、调用 NPE；正式呈现安全拒绝，NoCheck/NoStand 的工厂、捕获、构造与消费者来源集分别精确为 `{0,3,4,5,10,13}` 与 `{0,1,6}`。这项覆盖独立返回位，修复了构造实参片之前已存在的错误。

this/稳定分配/String 常量正例保持；直接 String 通过完整 KnownBoundPositive 整类重编回放。Class 字面量的额外 dup/getClass/pop 仍拒绝，作为能力边界排队，不混入创建时机修正。Unknown-frame 常量读取复用原 helper，没有改变显式已命名 Object 或普通局部变量的类型规则。

最终门禁、CI 和清理情况记录在共同验收结果与 handoff.md；未将 JADX 的错误语法作为语义依据。

主线收尾：全部实现与 root 证据已合入推送；实现分支已解除占用并删除，辅助 worktree 均 detached。cargo clean 移除 19.3 GiB，保护的干净工作树归档被 Codex 拒绝，未绕过。收尾 CI 的最终状态以 handoff 核对命令和远端 run 为准。
