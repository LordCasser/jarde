## 1. 冻结原/JADX/Jarde 基线与拒绝边界

- [x] 1.1 将 `generic-method-signatures/` 的 346 B Java 8 subject、source-only runner 和三方源码做参数化重放：独立重编 subject SHA `928311a0…`，原/JADX `-Xverify:all` 均为 `3|1`，基线 Jarde 为 `3|0`；保存 CLI/JADX 版本、javap 签名、javac/JVM 与源哈希。证据：`../../evidence/java-syntax-2026-09-24/generic-method-signatures/replay-results.txt`（隔离 CLI 重放）。
- [x] 1.2 构造 JVM 可验证的第一界改为 `Object`、未绑定类型变量、类级变量或未支持复杂签名反例；另造签名擦除匹配但正文返回 `Integer` 不能作为任意 `T extends Number` 的反例。逐例固定属性原字节、descriptor、验证器与反射/源码现状，不以无效 classfile 充当拒绝证据。证据：`../../evidence/java-syntax-2026-09-24/generic-method-signatures/negative-fixtures/`；包含合法 classfile 与原/JADX/冻结 Jarde 源结果、正文不兼容的 javac 拒绝，以及 `Exceptions` 与无 throws Signature 后缀的正向边界。

## 2. 复用签名语法并证明方法声明

- [x] 2.1 将 query 现有签名 grammar 提炼到 reader 的共享按需解析入口，产生结构化方法结果并完整消费、限制深度/节点；query 改由该结果遍历引用且保持原顺序。用有效简单签名与 1.2 反例运行 reader/query 定向测试，证明 Decompiler 不依赖 XRef。
- [x] 2.2 从同一物理成员唯一 `Signature` 与 descriptor 建立方法局部变量作用域和逐位置擦除证明；reader 只提供语法/擦除事实，不在这一层判定源码形状。`<T:Number>(TT;TT;Z)TT;` 准入，界/返回不符和未知变量拒绝；javac 有效 `Signature` 无 `^`、`Exceptions` 有 IOException 的类不能误拒。证据：[reader 原字节与负例测试](verification-reader-2.1-2.2.md)。
- [ ] 2.3 在 class-source 投影层决定已支持形状的完整拼写与复杂合法形状的明确拒绝；核对类型变量名、上界、参数/返回、`Exceptions`、varargs/type-use 注解接缝；复用 AST/SSA 证明正文中类型变量赋值、返回、表达式合流与调用目标，以及当前类其他方法对目标的调用绑定。无法保真时整项拒绝；用相邻重载、无调用、正文类型不兼容、注解与复杂签名 fixture 测试。

## 3. 完整类投影与主代理验收

- [ ] 3.1 类级方法头只在签名及正文源级类型关系均已证时加入 `<T extends ...>` 并替换对应类型位置；原物理身份、属性、独立恢复报告、body 与参数 slot 名保留。重编完整 Jarde 类，普通调用及 `getTypeParameters/getGenericParameterTypes/getGenericReturnType` 对原/JADX 逐项相同；擦除匹配而正文不兼容的反例保持 descriptor 声明且可重编。
- [x] 3.2 属性读取、语法、擦除、来源与输出按既有预算计费并轮询取消；低预算/取消不发布半泛型头，essential/all 正文相同，定向库/CLI 测试核对物理记录和停止原因。证据：[Root 预算/取消及 CLI 复核](verification-root-3.2.md)；预取消发生在类选择前，物理成员保留由方法体及内部预算停止覆盖。
- [ ] 3.3 root 在独立目录重放正负例，审读语法唯一性、擦除/绑定证明及来源；运行 query/class-source/相邻调用测试、reader census/fingerprint、`cargo fmt --all -- --check`、严格 Clippy 和 `openspec validate recover-generic-method-signatures --strict`，仅证据齐全后勾选。
