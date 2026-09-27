## 1. 固定测试基线与边界

- [x] 1.1 固定 pinned `TestTryCatchFinally12.TestCls` 三方法 Java 8 class、BCI/opcode/异常表，和同布局顶级最小类的原/JADX/Jarde 三方源码、九路径行为及首拒绝；提供可重放脚本并由 root 核运行记录与 SHA。
- [ ] 1.2 为 `test3` 和 `test1/2` 分别冻结 verifier 有效的参数不等、异常行扩围、出口/handler 身份反例；核原 class 行为与证书拒绝边界，不把无法通过 verifier 的改字节样本计入负例。

## 2. 三副本共用清理

- [ ] 2.1 在现有共享 finally 证书内证明 `test3` 三份当前实例 `StringBuilder.append(String)` 清理：精确字段/成员/常量、入口 `this`、SSA 局部栈消费和唯一 `pop`；正例获证，参数/目标/消费变化负例拒绝，预算/取消停在原子边界。
- [ ] 2.2 复用现有 `Joined` 正常完成与 `Plan::join`，让 `test3` 的具名 catch、一次 finally、独立 BCI 55 后续块被唯一 Region owner 认领；测试断言所有物理 BCI 来源和三行异常表，最小完整类的 test3 三路径与原 class 一致。

## 3. 两副本外层清理与内层 catch

- [ ] 3.1 对 `test1/2` 增加与旧保存返回证明互斥的两副本/正常 join 证书：两行表、外层完整保护内层 catch、每条正常/异常路径恰好一次等价追加、原 Throwable 重抛；核范围扩围/绕行/不同参数负例安全拒绝且旧证书不回退。
- [ ] 3.2 用既有有界 Region 递归恢复内层具名 catch 和 `test1` 的 `-out`，只在证书闭合时允许嵌套 `Try` 作外层 finally 正文；测试核全部受保护 BCI/块唯一 owner、无漏边和预算/取消回滚。
- [ ] 3.3 在已有 Builder checkpoint 内输出 `test1/2` 的唯一外层 finally、内层 catch 与后续 return，清理副本和 handler 行作来源；最小完整类六路径与原 class 一致，任何表达式或声明失败都不发布半个结构。

## 4. 三方验收与回归

- [ ] 4.1 对固定三方法和最小完整类重放原/JADX/Jarde Java 8 重编、`java -Xverify:all` 九路径；核清理一次、顺序、异常对象/类型、所有物理 BCI 来源和固定 JADX 默认三处 finally；保留 `runTest`/family 独立差距的记录，不宣称整个固定类追平。
- [ ] 4.2 复跑既有 CF-16 单出口、共享调用/字段/布尔赋值、typed catch、CF-18、资源/monitor 及全部定向反例；`cargo test -p jarde-java --tests --locked`、`cargo check --workspace --locked`、`cargo fmt --all -- --check`、`openspec validate recover-nested-finally-append-exits --strict`、`git diff --check` 通过并清理专用 Cargo target。
- [ ] 4.3 root 独立审阅副本等价、异常派发、SSA/Region owner、源码来源、九路径三方行为和负例，记录验收后更新 CF-16 清单，仅标记已验证的固定三方法与最小完整类。
