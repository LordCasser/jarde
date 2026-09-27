## 1. Same-run helper proof

- [ ] 1.1 从当前类的精确 LambdaMetafactory 站点及 bootstrap implementation handle 产生 helper 候选，并以物理 owner/name/descriptor 和 `private static synthetic` 声明确认；为错 owner、错误 flags、错误 descriptor、非 lambda bootstrap 写拒绝测试。
- [ ] 1.2 使用完整 helper Code 与现有 AST/effect/source-origin 证明只读的直线表达式、0/1/2 个原始参数与 SAM 次序映射；为分支、异常、调用、额外指令和部分读取写负例。

## 2. Atomic class-source projection

- [ ] 2.1 将可证明 helper body 投影到相应 lambda body，并保留 use-site、helper 指令与物理成员来源；验证参数位置、表达式位置与 source map。
- [ ] 2.2 在完整物理类成员引用 census 中证明所有 helper 使用仅为已成功投影的 implementation handle；整组 helper 与声明省略一次原子提交，普通调用、未知 handle、任意未消费引用均阻止省略。
- [ ] 2.3 将全类读取、候选检查、body 投影和 commit 接入既有预算/取消边界；用预算停止和取消对照验证没有部分 helper 内联/省略，报告保留停止来源。

## 3. Frozen semantic replay

- [ ] 3.1 扩展 `evidence/java8-lambda/replay.sh` 修后模式；原/JADX/Jarde 完整源码与 source-only Runner 通过 `javac --release 8`，以 `java -Xverify:all` 逐行对照 0/1/2 参数正例并验证输出不含 synthetic helper 声明/调用。
- [ ] 3.2 增加 helper 另有普通 invoke、未知句柄和复杂 body 的冻结反例，确认不得隐藏；完成适用的定向恢复测试与 `openspec validate --strict` 并记录证据。
