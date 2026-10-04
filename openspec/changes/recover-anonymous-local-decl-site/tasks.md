## 1. 取证与冻结

- [ ] 1.1 以 `anonymous-super-args/AnonymousSuperArgs$1` 为锚重放现状：完整源集 `javac --release 8` 退出 1、`main` 分配点的 verbatim 呈现、原 class `java -Xverify:all` 事件日志（`openspec/evidence/java-syntax-2026-09-27/anonymous-super-args/` 既有基线之上补本片前后对照）。
- [ ] 1.2 冻结正负例：局部声明初始化形正例（`AnonymousSuperArgs$1` 本身）；声明类型不可重拼（左端为不可拼写类型/数组/嵌套泛形）负例；初始化值非唯一分配点负例。

## 2. 实现

- [ ] 2.1 站点扫描接受局部声明初始化形（分配表达式为唯一初始化值），保持直返形与 `recover-anonymous-mixed-super-capture` 的判据逐字不变。
- [ ] 2.2 emitter 增加赋值左端声明类型重拼（匿名子类名 → 父类源码名），不可证明时保持物理文本；池形类型名结构反射陷阱判据适用（最终文本含 `$` 时保持拒绝）。
- [ ] 2.3 `project_class_source_anonymous_super` 根方法门放宽，捕获证明与参数角色划分复用 `recover-anonymous-mixed-super-capture` 的通道。

## 3. 验收

- [ ] 3.1 `anonymous-super-args` 完整源集 `javac --release 8` 通过、`java -Xverify:all` 事件日志逐行一致；全部既有匿名正负例（含本片锚 `anonymous-super-mixed-direct` 与六个 mixed refusals）零回退。
- [ ] 3.2 门禁：fmt、ci.yml 逐字 clippy、`openspec validate --all --strict`、corpus 双腿扫描（差异应仅赋值初始化形）、全仓测试。
