## 1. 封闭证明模型

- [x] 1.1 在现有 enum group 中增加带元素 BCI 的 String literal varargs 证明值与构造器 source-tail 类型；用单元测试验证空数组和多元素 ASCII 数组值、顺序及现有 int/no-arg 分支仍成立。
- [x] 1.2 证明唯一 `ACC_VARARGS` 构造器、物理 descriptor、源 Signature、`Enum` super 调用和唯一 `String[]` 实例字段赋值；用真实 Java 8 classfile 的正例与 flag/Signature/字段目标负例测试验证。

## 2. 初始化前缀与源码投影

- [x] 2.1 扩展同次 raw Code prefix matcher，仅接受有界 `String[]` 新建、按序 ASCII string literal 写入、唯一构造消费及精确常量字段写；用真实 classfile 正例和 JVM-loadable 的非 literal/额外消费/数组类型负例测试验证整组拒绝。
- [x] 2.2 在既有完整组证书通过后输出 `String...` 构造参数及安全转义的 ASCII 常量字符串，并拒绝非 ASCII MUTF-8 字节；用 `StringVarargs` 冻结样本完整 Java 8 重编验证，不改变物理报告、来源与未证明 fallback。
- [x] 2.3 验证预算、取消、缺失 Code/成员事实、无效 suffix 和某一常量失败均不会发布部分投影，且 stopped 与 refused 保持区分；运行针对性的 class-source 测试。

## 3. 三方验收与回归

- [x] 3.1 更新冻结三方对照并运行原 class、JADX、修后 Jarde 完整 Java 8 编译及 `-Xverify:all` runner，断言数组值、顺序和不同常量的数组身份。
- [x] 3.2 运行 `cargo fmt --all -- --check`、`cargo test -p jarde --lib --locked`、`cargo check --workspace --locked` 与 `openspec validate recover-proved-enum-string-varargs --strict`，记录结果并清理临时 Rust target。
