## 1. 固定整类与反例

- [x] 1.1 固化 Java 8 `int` 字段/数组后置返回旧值的自写完整类、helper/runner 和修前输出；核对 class SHA/Code 数、逐字节重编、原 class 与 JADX 全类 javac/验证/执行相同，Jarde 失败如实保留，并覆盖正常、null、越界、溢出及调用次数。Luna证据见 `postfix-lvalue/`；root将整个目录复制到临时位置独立重跑 `run_audit.py`：1228B/11Code、class SHA `7f9e8105…ff26893` 与原记录相同，原/JADX完整类19行全同，Jarde仅两个后置方法缺return，冻结CLI哈希前后相同。
- [x] 1.2 为异字段/异数组索引、非旧值返回、额外复制消费者、前置生产者跨独立效果、普通赋值、已支持简单字段递增与前置对照固定源码或 JVM 验证的受控变体；以 javap BCI、JVM 验证和三方输出判定可恢复/必须拒绝的边界，不把通用 `dup` 视为后置更新。Luna `boundaries/` 十个样本全通过 JVM 验证；root 独立复制到 `/tmp/jarde-postfix-boundary-root-u3FuN9` 重放，manifest 逐字节相同，原/JADX 全类运行一致，Jarde 当前整类 javac 失败的范围如实保留。

## 2. 最小表达式与数据流闭环

- [x] 2.1 在现有表达式 AST/emitter 加入字段/数组可写目标上的后置递增语义及最高绑定级别；用嵌套接收者/下标和来源测试验证 `target++`、括号及真实节点来源，不用整句 `Local` 文本。
- [x] 2.2 从 `ireturn`、store、算术、读取与复制的 SSA 身份核对同块旧值返回；字段只认同一 `I` 成员，数组只认已证明 `[I` 和同一数组/索引。八个冻结身份/效果负例和简单字段更新回归通过；合法跨 handler 负例由更早 Region 阶段拒绝，未误折叠，但不构成后置 handler 检查被直接触发的证据；不新增一般复制求解器。
- [x] 2.3 在唯一返回发射点认领可单次且不跨独立效果呈现的接收者、数组和索引生产者及完整更新链；整类 17 行运行输出确认调用次数/顺序、异常阶段与溢出结果。负例仍逐链引用原始 BCI，不重复发射效果。
- [x] 2.4 给真实读、复制、加法、写入、返回和子表达式保留 BCI/成员来源；默认/完整来源正文逐字相同，输出预算、预取消和深度界限均未交付半个更新产物。

## 3. 整类对照与主代理验收

- [x] 3.1 编译并执行完整 Engine/CLI 生成类，与原 class/JADX 的正常、异常和溢出输出逐行对照；字段递增、复合 `+=`、数组访问、普通赋值、延期生产者与拒绝来源定向回归通过。`p3_prefix_survival` 两条旧拒绝断言与现有已恢复的 `if`/`while` 输出冲突，单列测试预期债务；未手改生成方法。
- [x] 3.2 root 独立审查旧值身份、单次归属和拒绝来源，以当前 CLI 重放完整类及独立边界；复核 reader census、corpus fingerprint、fmt、受影响 Clippy、OpenSpec strict 和磁盘用量。reader 旧钉值和跨文件既存 Clippy lint 另案记录；具体结果见 `verification-root.md`。
