## 1. 固定整类证据与拒绝边界

- [x] 1.1 将 `compound-assignments/write-only/` 的自写 Java 8 class、runner 和必要 helper 收成永久 fixture，保留 class/Code 数与 SHA-256；回放修前原/JADX/Jarde 的整类 javac、`-Xverify:all` 和七行执行对照，确认 Jarde 六行错义且没有删除方法或改生成正文。
- [x] 1.2 构造并运行同一成员以外字段、不同数组/索引复制、多消费者 `dup`/`dup2`、普通 `=`、非 `I` 字段/数组、数组空引用/越界与同左值快照的合法边界；记录栈数据流/BCI 与原 class 执行，明确拒绝需保留来源引用，不能以当前可编译错值作为验收。

## 2. 闭合语义、证明与发射

- [x] 2.1 在既有字段/数组赋值 AST 和统一 emitter 加最小赋值运算语义，只给已证明更新写 `+=`；用接收者和索引各有一次副作用的发射/来源测试验证没有重复文本或新预格式化表达式节点。
- [x] 2.2 在现有字段读写事实、SSA 身份和调用延期路径上证明一块内 `I` 字段 `dup/getfield/RHS/iadd/putfield` 更新，认领整条效果链；以字段普通/同左值改写/null、异常与异成员拒绝测试验证调用一次、旧值快照及 `getfield` 先于 RHS。另用 JVM 合法独立效果插在接收者→dup、dup→read、add→store 的变体验证间隙拒绝与完整来源。
- [x] 2.3 在现有数组读写事实、SSA 身份和调用延期路径上证明一块内 `[I` 的 `dup2/iaload/RHS/iadd/iastore` 更新，认领数组/索引和 RHS；以数组普通/同元素改写/null/越界、异索引及额外消费者测试验证单次求值与检查先于 RHS。另验数组/索引→dup、dup→read、add→store 的独立效果间隙不被错误移动。
- [x] 2.4 用默认/完整 source-map 请求核实正文相同、接收者/索引/读/加/写/RHS 的真实 BCI 与 Fieldref 来源；正文、来源预算和取消分别触发既有停止结果，拒绝链保留生产者来源。

## 3. 完整执行与主代理验收

- [x] 3.1 使用完整 Engine/CLI 原样生成并编译 fixture 的完整类，与原 class 和 JADX 对照全部七行；另执行 1.2 边界、局部 `+=` 和已有 `field++` 回归，不以删掉失败方法或手改生成文本代替验收。
- [x] 3.2 root 独立审查 SSA 复制身份、RHS 延期与拒绝来源，在重新构建的冻结 CLI 上重放整类与独立边界；运行受影响 Rust/Java 测试、census/fingerprint、fmt、clippy 和 OpenSpec strict，逐项记录既存门禁债务与结论。见 `verification.md`。
