# 增强 for 的接口 owner 对照

## 固定输入与字节码

用 `javac 23.0.1 --release 8` 分别以 `-g`、`-g:none` 编译完整 fixture；subject class major version 为 52。源码包括 `List<String>`、`Collection<String>` 两个形参，以及 `TextIterable extends Iterable<String>` 用户接口形参，runner 对三者均使用 `"a", "bc", "def"`。冻结源码和每个输入 class 的 SHA-256 见 [`source-hashes.txt`](source-hashes.txt) 与 [`class-hashes.txt`](class-hashes.txt)，两个版本的完整调用字节码见 [`original/g/SubtypeOwners.javap.txt`](original/g/SubtypeOwners.javap.txt)、[`original/nodebug/SubtypeOwners.javap.txt`](original/nodebug/SubtypeOwners.javap.txt)。debug 和 no-debug 输出的调用 owner 一致：

| 方法形参源码类型 | `invokeinterface` owner | descriptor |
|---|---|---|
| `List<String>` | `java/util/List` | `()Ljava/util/Iterator;` |
| `Collection<String>` | `java/util/Collection` | `()Ljava/util/Iterator;` |
| `TextIterable` | `TextIterable` | `()Ljava/util/Iterator;` |

这三项分别使用 `List`、`Collection` 和用户接口自己的 owner，都不是 `java/lang/Iterable`。元素读取均为 `Iterator.next()Ljava/lang/Object;` 后 `checkcast java/lang/String`。原 class 的 `java -Xverify:all` 输出：

```text
list=6
collection=6
textIterable=6
```

## JADX 与 Jarde 输出

JADX 1.5.6：`-g` 时把 List 与 Collection 两个循环输出为 `for (String value : values)`；用户自定义 `TextIterable` 循环仍是显式 `Iterator`/`while`。`-g:none` 时三者均为 `while`，其中 List/Collection 使用 `Iterator<String>`。定向源码和运行输出保存在 [`jadx/g`](jadx/g) 与 [`jadx/nodebug`](jadx/nodebug)；使用仅由这些源码、runner 与对应 `TextIterable` 声明构成的项目，以 `javac --release 8 -g:none` 重编成功，`java -Xverify:all` 三项均输出 6。

当前 Jarde CLI（哈希见 [`jarde-cli.sha256.txt`](jarde-cli.sha256.txt)）对 `-g` 与 `-g:none` 的三种 owner 都保留 `Iterator` 局部变量和 `while (it.hasNext())`，body 中保留 `(String) it.next()`。List/Collection 两个方法还报告 generic Signature projection refused；但 Jarde 完整 class 文本仍可与原接口、source-only runner 一起用 `javac --release 8 -g:none` 重编，`java -Xverify:all` 结果与原 class 相同。源码和运行日志见 [`jarde/g`](jarde/g)、[`jarde/nodebug`](jarde/nodebug)。

## 对准入边界的含义

JADX 在 debug 控制下至少能认领 List/Collection 形状，但并未认领调用 owner 为自定义子接口的对应形状；去掉调试表后，它也退回 while。因此这组结果不能作为 Jarde 可安全投影这些 owner 的证明。Jarde 当前确实能从 classfile 读取方法形参类型/Signature 与调用的 `CallTarget`，也能读取当前定义类的 direct superclass/interfaces；但 `class-source --policy single-class` 没有参数类型引用所指向接口的层级定义，现有类型层也没有通用的传递子类型判定。fixture 的 `TextIterable` 层级只存在于独立 class，而 `SubtypeOwners` 自身 `interfaces: 0`。所以现有直接 `java/lang/Iterable.iterator` 准入不能自动推出 List、Collection 或用户子接口可准入；要支持它们，需另行提供有来源的接口层级/类型证明，并证明 Java 8 foreach 表达式类型合法。当前实现正确保持 while，是保守回退，不是该功能已支持。

复现时用上述冻结输入执行 `jadx -d <out> <SubtypeOwners.class>` 与 `jarde-cli class-source --input <SubtypeOwners.class> --class SubtypeOwners --policy single-class --release 8 --format text`，随后将完整输出、`TextIterable` 声明和 runner 单独编译运行即可。版本、class major、CLI 哈希和编译产物哈希分别保存在本目录的 `tool-versions.txt`、`class-version.txt`、`jarde-cli.sha256.txt` 与 `rebuilt-hashes.txt`。
