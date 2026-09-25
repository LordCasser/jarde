## 1. 冻结字段证据

- [x] 1.1 固定 `-g`/`-g:none` 的 Java 8 字段正例及无本类字段引用的安全子集，重放源码、`javap`、JADX、当前 Jarde，核对原类 `-Xverify:all`、字段泛型反射、完整类与调用方重编，并记录 SHA-256 和工具版本。
- [x] 1.2 保留 descriptor 不变地构造 verifier-valid 的擦除不符及正文泛型冲突负例，记录 JADX 与当前 Jarde 字段输出、`javac --release 8` 的成败以及 Jarde 应拒绝的原因；用可重放脚本和证据文档验收。

## 2. 复用 reader 和类源码接缝

- [x] 2.1 在 reader 的现有字段 `Signature` 树上证明类变量作用域闭合及完整 descriptor 擦除，覆盖参数化、通配符、`T`、`T[]`、未绑定变量、擦除不符、预算和取消；跑 reader Signature 定向测试。
- [x] 2.2 在字段类源码候选中复用已有类型拼写器，按唯一属性、已发布类作用域、源名称/flags、注解及同类 Fieldref 缺席门原子投影；失败保留物理声明与局部拒绝，字段 pool 按需读取并计费；用完整类正反例及 essential/all、低预算测试验收。
- [x] 2.3 用 1.1/1.2 的 fixture 为字段类源码增加定向回归，检查类变量字段和数组、无本类引用的通配符、正文冲突、擦除不符、annotation 拒绝及物理属性来源；完整类及调用方 `javac --release 8`、`java -Xverify:all` 的输出与原类对照。

## 3. 独立验收

- [x] 3.1 root 独立审读字段擦除、类作用域和 Fieldref 门，重放三方证据与正反例，运行 reader/query、普通泛型/类级泛型、class-source、预算定向回归及格式/适当 Clippy、`git diff --check`、`openspec validate recover-proved-field-signatures --strict`；记录正文兼容证明的后续边界并清理私有 Cargo target。
