# DT-11：枚举 String varargs 构造实参

本目录冻结了 Java 8 输入源码与原始 class、固定 JADX 输出、Jarde class-source 输出和三方重编运行结果。原始 enum class 的 SHA-256 由 `run_audit.py` 校验，避免证据输入悄然变化。

从仓库根目录重放：

```sh
python3 openspec/evidence/java-syntax-2026-09-27/enum-constructor-arguments/run_audit.py
```

脚本只重建本目录的 `generated/`；javac 产物、JADX 工作目录和 Rust Cargo target 都在自动清理的临时目录中。JADX 默认使用 `/Users/lordcasser/workspace/testzone/jadx` 的固定 checkout，也可设置 `JADX_CHECKOUT`、`JADX`；可用 `JARDE_CLI` 指定 Jarde CLI，否则脚本从当前 checkout 临时构建。`generated/summary.json` 保存三方状态，`generated/sha256.txt` 保存输入、完整源码、原始 class 和工具 JAR 哈希，`generated/javap-StringVarargs.log` 保存完整物理构造和初始化字节码。

## 对照结果

| 输入 | 原 class | JADX 1.5.6 | 当前 Jarde |
| --- | --- | --- | --- |
| `LiteralOnly`：`FIRST(7), NEXT(-2)` | Java 8 重编、`-Xverify:all` 运行通过 | 重编和运行通过 | 重编和运行通过 |
| `IntArgs`：literal、静态字段读与加法 | 重编和运行通过 | 重编和运行通过 | 同次 Code 证明、重编和运行通过 |
| `StringVarargs`：两个、一个和零个字符串元素 | 重编和运行通过 | 重编和运行通过 | 有界数组证明后重编和运行通过 |

`StringVarargsRunner` 检查元素内容与顺序、每次访问同一常量保留数组身份，以及不同 enum 常量持有不同数组；`EMPTY` 的数组确实是零长度且与其他常量数组不同。Jarde 对完整 class-source 输出运行 `javac --release 8`，随后用 `java -Xverify:all` 执行同一个 runner。

## 被证明的 classfile 形状

原始构造器的物理 descriptor 为 `(Ljava/lang/String;I[Ljava/lang/String;)V`，flags 是 `ACC_PRIVATE | ACC_VARARGS`，源 Signature 是 `([Ljava/lang/String;)V`。构造器调用 `Enum(String,int)` 后，仅将 slot 3 写入自身唯一实例 `String[]` 字段。`<clinit>` 对每个常量都实际分配新数组；PAIR 和 SINGLE 逐个执行 `dup/index/ldc/aastore`，EMPTY 执行零长度分配后直接传入构造器。

源码投影只接受最多 64 个元素、按 `0..n-1` 顺序写入的字符串 literal。当前 classfile reader 向证明提供 MUTF-8 原始字节，因此首切片严格接受 ASCII 字节并使用既有 Java literal 转义器；遇到非 ASCII 字节或 MUTF-8 NUL 编码时整组拒绝，不进行 lossy 解码。`EMPTY()` 在 Java varargs 源码中会重建新空数组，冻结 runner 已检查其身份语义。

真实 classfile 负例覆盖缺少 `ACC_VARARGS`、source Signature 改写、额外 `String[]` 字段、非 literal 元素、数组额外 consumer、其他数组类型、重复 index 写、非 ASCII literal 和无效 `<clinit>` suffix。可用负例在 `-Xverify:all` 下不初始化加载，证明拒绝来自窄证书边界而不是 classfile 不可加载。
