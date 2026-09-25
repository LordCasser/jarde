## 1. 固定 Store 与 local 的直接/间接 use 边界

- [x] 1.1 为冻结正例增加测试，断言 Phi 直接 use 只有 BCI 21 的 `istore_1`，Store 后 local 的 BCI 22/26 读取均绑定同一变量；以定向测试验证区分。
- [ ] 1.2 补齐无名、非 boolean、多个 Store/不同作用域与第二 Phi use 的永久拒绝控制，并逐例断言 quote/source map 与失败原因。

## 2. 一次 local Store 消费

- [x] 2.1 在既有短路图与唯一 Phi 证明中增加严格 `istore` anchor；定向证明必须确认 stack Phi、local slot、variable identity、名字与 Boolean declaration decision 全一致。
- [x] 2.2 在既有 `decide_types`/声明决策之前实现窄的 local Boolean-use 证据：证明 Phi 为精确 1/0、唯一直接 Store 到 SSA/slot 同一 local，且该 local 的全部后续 definitions/loads 完整可追踪、没有其它类型或未解析消费者，所有 load 都流向显式 Boolean 描述符消费；使证据仅为该 local 写入 `BooleanEvidence` 并得到 `Type::Boolean`。特别覆盖现状 `boolean_proof(Definition::Phi) = None` 会回落为 `Type::Int`，不得把这个类型扩展遗漏。
- [x] 2.3 将惰性条件值只交给现有 local declaration/assignment AST；用 source map 与 AST 检查验证 Store、测试、producer 来源，并确认后续 `iload` 仅引用 local 名。
- [x] 2.4 对 `one(Z)Z` 完整类进行 Java 8 重编和 `-Xverify:all` 八路径对照；断言 return、字段结果及 `b/c` 次数与原 class 逐行相同，两次 local read 不重复 RHS。

## 3. 独立验收

- [x] 3.1 复跑字段、直接返回、invocation argument 短路验收，以及类型/作用域/第二 use/ownership overlap 拒绝控制；定向套件全部通过且调用实参实现不被本 change 改动。
- [x] 3.2 运行受影响定向测试、格式、适用 Clippy 和 `openspec validate recover-short-circuit-local-values --strict`；记录命令结果、剩余边界并以 `cargo clean --target-dir` 清理私有 target。
