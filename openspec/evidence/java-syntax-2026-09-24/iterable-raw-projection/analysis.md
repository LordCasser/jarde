# Raw `Iterable` 的合法增强 `for` 投影探针

[手工探针](StringIterableForeach.java)特意把方法头降成原始 `Iterable`／`Supplier`，以 `for (Object item : values)` 隐式执行迭代器，再在循环体内显式保留 `(String) item`。这不是 Jarde 已实现的输出；它只检验类型策略是否可行。用原[十行 runner](../../java-syntax-2026-09-22/enhanced-for/root-replay/StringIterableForeachRunner.java)一同 `javac 23.0.1 --release 8 -g:none` 编译，`java -Xverify:all` 运行。探针 class SHA-256 `b3b5bf2f93625242da21adb99e439dab3e3121f78d1af84b7699b8be19016e11`；独立运行记录在 `/tmp/jarde-iterable-raw-projection/`。

空、多元素、null、一次 supplier 调用、supplier 抛错、`iterator()`／`hasNext()`／`next()` 抛错的前九行与原 class 完全相同。第十行的 null 元素同为 `NullPointerException`，迭代器计数 `1/2/2` 相同；Java 23 helpful-NPE 文本把局部从 `<local3>` 写为 `<local4>`。`javap -c -p` 显示手工探针的 `next()` 结果先保存为 `Object`，随后才 `checkcast String`；原 class 在 `next()` 后直接 `checkcast String`。两种可编译 Java 8 源形式的可观察值和异常顺序相同，局部编号及诊断措辞不同。

因此当前 Jarde 输出的 raw 方法头本身不排除增强 `for`：可将隐式 `next()` 绑定为 `Object`，保留原位显式转换。实现时仍须证明 `Iterable` 源类型、迭代器唯一用途、转换与循环体绑定、异常边和真实来源，不能拿这份手工探针代替完整 SSA 证明。更完整的泛型方法签名恢复是另一个在途 OpenSpec，不应成为此处准入的无谓硬前置。
