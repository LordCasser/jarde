## 1. 冻结类级作用域证据

- [x] 1.1 复用 `ClassVariableBoundary<U>` 的原/JADX/Jarde 三方记录，补 `-g`/`-g:none` class 哈希、class/method `Signature`、原类与 JADX 的 Java 8 重编、泛型调用方 `-Xverify:all` 执行及反射；冻结 Jarde 当前类头与方法头的失败位置，写可重放脚本。
- [x] 1.2 构造一个带明确 class/interface 边界和无正文方法的 Java 8 正例，并用合法/变造 class 分别覆盖父类或接口擦除不符、未声明变量、方法同名遮蔽及内类外层作用域负例；每例记录 verifier、`javap`、三方声明与预期拒绝，不把缺 classpath 当无效 `Signature`。

## 2. 复用解析与证明接缝

- [x] 2.1 在 reader 的既有 `ClassSignature` 语法树上证明唯一变量作用域、边界擦除及物理父类/接口逐位置一致性；将已证类变量作用域交给方法擦除证明，缺变量、冲突边界和遮蔽不猜。用 1.1/1.2 的正反例及 reader/query 定向测试核验预算、取消和错误代码。
- [x] 2.2 在类源码层由物理 flags、已证 class `Signature` 和结构化类型构造完整类头候选，成功后原子发布类变量及边界；失败保留物理头和来源拒绝。用正反例核对 `ClassSourceDeclaration`、essential/all 文本与低预算输出。
- [x] 2.3 将已发布的类变量作用域传入现有方法投影，支持有正文的直接参数返回和已证无正文声明；保留同轮参数槽、annotation、varargs、`Exceptions`、Methodref 绑定及拒绝门，不允许方法头出现未发布的 `U`。用 1.1/1.2 重编与方法独立报告、来源验证。

## 3. 独立整类验收

- [x] 3.1 用重新构建的 CLI 对有无调试信息的 1.1 正例和 1.2 有界/无正文正例生成完整类；原/JADX/Jarde 分别 `javac --release 8` 重编同一泛型调用方，`java -Xverify:all` 核对值、异常、class/method 泛型反射；负例不得发布孤儿类变量。
- [x] 3.2 root 审读 reader 证明、类/方法作用域交接与类源码原子发布，跑 reader/query、泛型方法/普通参数化、class-source、预算和来源定向回归，以及 `cargo fmt --all -- --check`、适当 Clippy、`git diff --check`、`openspec validate recover-class-type-variable-signatures --strict`；记录剩余字段、内类与外部依赖债务并清理私有 Cargo target。
