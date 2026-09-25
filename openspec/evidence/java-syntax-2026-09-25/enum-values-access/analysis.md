# 枚举隐式数组的额外读取：JADX 的可编译但不等价改写

本地 JADX checkout `2fb1b163` 的 `EnumVisitor.fixValuesAccess` 在枚举转换后，将用户方法里的 `$VALUES` `SGET` 换成 `values()` 调用。但标准 `values()` 返回的是数组克隆，这一步不能仅凭字段同名或调用形状判定等价。它可以定位需要处理的引用，却不能替代使用关系证明。

`E.java`、`Runner.java` 先以 `javac --release 8` 编译；`replay.py` 只把 `E.raw()` 中唯一的三字节 `invokestatic E.values:()[LE;` 换成同宽 `getstatic E.$VALUES:[LE;`，不改其它方法或常量池。`-g` 和 `-g:none` 两份完整类均通过 `java -Xverify:all`。补丁后的 `javap -v -c -p`、JADX/Jarde 全枚举文本、运行结果和 class SHA 分别保存在本目录；没有保留 class、jar 或构建目录。重放命令：

```sh
JARDE_CLI=/tmp/jarde-generic-accepted-cli python3 openspec/evidence/java-syntax-2026-09-25/enum-values-access/replay.py
```

冻结的 Jarde CLI SHA-256 为 `ca04265a4f412d59c29d6bd4a26b7d9cb961f72ae13e77684831c0b9e57b5145`；JADX 命令版本为 1.5.6，源码位置为本地 `jadx-core/src/main/java/jadx/core/dex/visitors/EnumVisitor.java` 的 `fixValuesAccess`。

| 输入模式 | 原 class，经 verifier | JADX 全类重编后，经 verifier | 冻结 Jarde |
| --- | --- | --- | --- |
| `-g` | `first=B,raw=B` | `first=A,raw=A` | `raw()` 的 `return E.$VALUES;` 仍可见；枚举常量仍写成普通字段，`javac` 在该处报错 |
| `-g:none` | `first=B,raw=B` | `first=A,raw=A` | 同上 |

原 class 的 `raw()` 把**同一 backing array**交给调用者，写入 `E.B` 后编译器生成的 `values()` 克隆该已修改数组。JADX 写回 `return values();`，调用者仅修改克隆，所以两次读取均为 `A`。这是源码无法直接写出隐式 `$VALUES` 的 verifier-valid 边界，不能为追求 enum 常量列表而静默替换成 Java `values()`；基础枚举类级证明必须检查**所有物理方法**对将被隐藏的字段/方法的直接与间接使用，发现未纳入证明的读取时整组拒绝。该场景作为 `recover-proved-enum-constants` 的拒绝测试，不扩成一个“忠实表达任意 `$VALUES` 访问”的新机制。
