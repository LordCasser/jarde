# DT-11 String varargs 根验收

主线合入 `457c43e9` 后独立重放 `run_audit.py`：冻结的原 class SHA 未变，原始、JADX 与修后 Jarde 的 `LiteralOnly`、`IntArgs`、`StringVarargs` 完整源码均通过 `javac --release 8` 重编与 `java -Xverify:all`。`StringVarargs` 的 `PAIR(".dex", ".class")`、`SINGLE(".xml")`、`EMPTY()` 三常量和 `private StringVarargs(java.lang.String... arg0)` 均从完整枚举组证书发射。runner 检查元素值、长度、同一常量字段读取的数组身份稳定，以及三个常量持有不同数组；三方均输出 `OK StringVarargs`。重放的临时 class 和 Cargo target 已自动清理。

同次 `<clinit>` raw Code 证书限定 `String[]` 新建、最多 64 个按 `0..n-1` 顺序写入的 ASCII 字符串 literal，以及唯一构造消费和字段发布；零长度仍需真实分配。构造器需同时满足唯一物理 descriptor、`ACC_VARARGS`、源 Signature、隐式 Enum 调用及唯一实例数组字段写入。非 ASCII MUTF-8、非 literal、额外数组使用、字段歧义、Signature/flag 不符和一项常量失败均不产生部分枚举源码；物理字段和方法继续可查询。这个边界不宣称一般 String 表达式或所有 Java 枚举构造形态已恢复。

根验收在预算计费只作用于 String-varargs 新切片、保留已有 int/no-arg 路径后运行：`cargo fmt --all -- --check`、`cargo test -p jarde --lib --locked`（113/113）、`cargo check --workspace --locked`、`openspec validate recover-proved-enum-string-varargs --strict` 及全量 OpenSpec strict（140/140）通过。`run_audit.py` 的生成文件复放无路径/HEAD 漂移；报告工具链记录固定 JADX checkout，而不记录每次验收都会变化的主线提交。
