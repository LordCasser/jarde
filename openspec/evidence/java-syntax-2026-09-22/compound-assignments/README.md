# 复合左值写入与后置递增：整类三方审计

自写 Java 8 源码按 `javac --release 8 -g:none` 编译；用 `java -Xverify:all` 执行原类、JADX 1.5.6 的完整生成类和 Jarde 的完整 `class-source` 文本。`run_audit.py` 以及 `write-only/run_audit.py` 可独立重放，使用冻结 CLI SHA-256 `30481c1449b5a1869773d21b934789d42bb1d6a9cab8d81f873d50055af4e34c`；脚本同时检查运行前后哈希，逐个保留 javac/java 状态及文本，没有删改恢复方法。`summary.json` 是主类实测，`write-only/summary.json` 是去掉后置递增、专看复合写入的实测。

| 输入 | class/Code | 原 class | JADX 整类 | Jarde 整类 |
| --- | --- | --- | --- | --- |
| 主类（含 `return receiver().value++`、`return data[index()]++`） | 947 B / 10，SHA-256 `f38c8ec956ecce1ea0d63556f777e98717fe1bb67d10fc1e123df73e2d6a830d` | javac/run 0，7 行 | javac/run 0，7 行全同 | CLI 0；javac 1，两个后置方法缺返回，不报告运行等价 |
| write-only（含 RHS 修改同一字段/元素） | 1113 B / 12，SHA-256 `889f36d3a5c06951d32762c8914829c059d8b1d6692001c08df73404c92f673f` | javac/run 0，7 行 | javac/run 0，7 行全同 | javac/run 0；7 行中只有 local 一行相同，其余 6 行均不同 |

write-only 原类的字段/数组结果为 `9`/`14`、接收者/索引调用次数均为 1、RHS 调用次数均为 1；Jarde 为 `7`/`10`、1、0。RHS 将同一左值改为 100 的两个快照测试中，原类仍用先读出的 7/10 做加法，得到 9/14；Jarde 既没有执行 RHS，也没有执行写入。原类在 null 接收者和越界索引时先抛 NPE/AIOOBE，不调用 RHS；Jarde 先调用接收者/索引，随后正常返回。纯局部 `value += rhs(3)` 在现有局部赋值路径已忠实，作为正面对照。

`javap` 明确给出两种待认领形状：`receiver@0; dup@3; getfield@4; rhs@8; iadd@11; putfield@12` 与 `data@0; index@3; dup2@6; iaload@7; rhs@9; iadd@12; iastore@13`。快照变体保持相同的复制/读/加/写骨架。后置值分别用 `dup_x1` / `dup_x2` 在更新前保留旧值，与无返回值的 `+=` 不同，应另案设计。若将 `receiver().value += rhs()` 写成重复左值的简单 `receiver().value = receiver().value + rhs()`，有副作用接收者会调用两次，且可能改变异常先后；这不是安全降级。

源码 SHA-256：主类 `CompoundProbe.java` 为 `9ba303b045403baf3083d5b6d7b54d2b5c200b9f7f864c937b507917c8e16fb7`，write-only 为 `18d257a39d8c749183cdcc8916ff7049569b00a17a41e5f3eefa4f3f4dd660f1`。类型与执行顺序以 [JLS 8 §15.26.2](https://docs.oracle.com/javase/specs/jls/se8/html/jls-15.html#jls-15.26.2) 为准；该节区分普通赋值与复合数组赋值的边界检查时机。当前只提案 `int` 字段/`int[]` 元素的有界 `+=` 语句，不把后置返回、窄化、其它运算符或一般栈复制混进此项。

永久复现夹具与最终边界证据在 [`tests/fixtures/p3-compound-lvalue-updates/README.md`](../../../../tests/fixtures/p3-compound-lvalue-updates/README.md) 和 [`permanent-fixture-replay/`](permanent-fixture-replay/)：包括主类七行整类对照、合法 Code/Fieldref patch 负例、普通赋值与 `long` 边界、字段/元素旧值快照，以及单独的 null-array 和 bounds-before-RHS 检查。Jarde 的负例输出维持原样；边界未来可保守引用来源，但不得将当前编译成功的错值当作通过。
