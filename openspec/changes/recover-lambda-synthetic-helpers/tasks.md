## 1. Same-run helper proof

- [x] 1.1 从当前类的精确 LambdaMetafactory 站点及 bootstrap implementation handle 产生 helper 候选，并以物理 owner/name/descriptor 与 `private static synthetic` 声明确认；拒绝错 owner、flags、descriptor 或 bootstrap。
- [x] 1.2 使用完整 helper Code、异常表与 AST 证明只读的直线表达式、0/1/2 个原始参数与 SAM 次序映射；比较物理 instruction BCI 集与 return/表达式 AST anchors 精确相等，分支、调用、额外/未消费指令拒绝投影。

## 2. Atomic class-source projection

- [x] 2.1 将可证明 helper body 投影到对应 lambda body，保留 helper 指令 origin 和 invokedynamic use-site；物理方法报告继续保留 helper。
- [x] 2.2 在完整物理类 census 中验证普通调用、ldc handle、精确 BCI/CP/bootstrap tuple、先闭合 ConstantDynamic/bootstrap 可达关系再检查实现 handle 反向使用；整组 helper 原子内联/省略，附带共享 BootstrapMethods、未投影站点、嵌套 Dynamic/back-edge、direct invocation 的负例。
- [x] 2.3 将类读取、候选、body 投影与提交接入 IR/output 预算及取消轮询；第二个 helper staging 的预算停止不会发布部分集合，并保留停止诊断。

## 3. Frozen semantic replay

- [x] 3.1 固定原始/JADX/Jarde 三方完整源与 source-only Runner，`javac --release 8` 重编并以 `java -Xverify:all` 对照 0/1/2 参数输出；baseline/fixed 分目录且确定性重放。
- [x] 3.2 负例覆盖复杂分支 helper、直接调用、未知/非实现 bootstrap handle、共享 bootstrap row 与 incomplete scan；完成 focused Rust 测试和 `openspec validate --strict`。
