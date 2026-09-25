# 数组 `clone()` 的类型、浅拷贝与重载：正面对照

`CloneProbe.java` 是自写 Java 8 源码，`run_audit.py` 在冻结 CLI SHA-256 `30481c1449b5a1869773d21b934789d42bb1d6a9cab8d81f873d50055af4e34c` 上重放 `javac --release 8 -g:none`、JADX 1.5.6 与 Jarde 完整类文本的 javac/`java -Xverify:all`。原 class 670 B、8 个 Code、SHA-256 `4a52f56e56bb9858bb016f9675a6f35503b905d6e3f28b556d70c50ef478dc80`。三方均编译并执行成功，六行逐项相同，见 `summary.json`。

输入覆盖 `int[]`、`String[]`、二维 `int[][]` 外层浅拷贝、`String[]`→`Object[]` 返回、`accept(Object)`/`accept(int[])` 重载选择和 null NPE。`javap` 表明 `invokevirtual [I/[Ljava.lang.String;/[[I.clone:()Object` 后有真实 `checkcast`；Jarde 保留 `(int[]) arg0.clone()` 等显式转换，重载仍选数组形参且二维内层保持同一引用。这里的额外转换文本不等于错误，它忠实保留了输入 class 的转换指令；该语法点当前无需新 OpenSpec 或新机制。
