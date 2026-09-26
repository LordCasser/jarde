## Context

DT-11 冻结的 `StringVarargs` 原始 class 与 JADX 输出都能在 Java 8 下重编和运行；Jarde 原先只接受无源码尾参或 `int` 尾参，因此把 enum 常量保留为物理字段。原构造器是 private `ACC_VARARGS`，物理 descriptor 为 `(Ljava/lang/String;I[Ljava/lang/String;)V`，源码 Signature 为 `([Ljava/lang/String;)V`。其 Code 调用 `Enum(String,int)` 后，将 slot 3 写入唯一 `String[]` 实例字段。`<clinit>` 为每个常量分配新 `String[]`，按序写入字符串 literal，再调用该构造器并写入 enum 字段。`EMPTY` 也执行真实的 `new String[0]`。证据见 `openspec/evidence/java-syntax-2026-09-27/enum-constructor-arguments/`，JADX 对照为 `TestEnums4` 和 `EnumVisitor`。

## Goals / Non-Goals

**Goals:** 在不放宽既有类级和 group gates 的前提下，将这一精确的 String-varargs 形状作为普通 enum 源码整体投影，包括空数组。对完整生成类执行 Java 8 重编，并将运行行为与原始 class、JADX 对照。

**Non-Goals:** 任意数组或表达式反编译、构造器重载、匿名 enum 常量体、null 或计算元素、新公开 API、通用方法恢复。

## Decisions

### 1. 扩展现有封闭 enum 证书

复用 `src/enum_constants.rs` 的同次 `EnumMethodCodeCandidate` 和 group proof，不增加第二个 enum recognizer 或 parser。源码尾参使用 typed closed variant，区分无参、受限 int 表达式和有序 String literals，并携带各元素 BCI。`src/class_source.rs` 将两个相互独立的 constructor source-tail bool 合并为一个内部枚举类型。物理 descriptor、源 Signature、varargs flag 和 constructor body 各自验证，只有它们一致时才投影。

### 2. 精确证明数组构造和消费

初始化 matcher 从每个常量的 `new; dup; name; ordinal` 后开始，只接受不大于 64 的整数长度 literal、精确的 `anewarray java/lang/String`，随后是 `n` 组连续的 `dup; index; ldc-string; aastore`，index 严格为 `0..n-1`，之后立即跟随唯一预期的 `invokespecial` 和 `putstatic`。检查 constant-pool class/string references、无 handlers、完整连续指令覆盖、唯一使用和既有 `$VALUES`/suffix gates。首切片只接受 ASCII modified-UTF-8 字节；MUTF-8 NUL 或非 ASCII 字节均拒绝，不进行 lossy 解码。零长度实参仍必须来自真实数组分配，随后源码可写成 `EMPTY()`；三方 replay 检查不同常量持有不同数组。

构造器必须是精确八指令形状，只调用 Enum 构造器并将 `aload_3` 写入自身唯一 `String[]` 字段；现有 int 证明保持不变。若 AST sidecar 不能表示初始化器中的交错数组指令，由完整同次 raw Code 证书证明前缀，方式与已接受的 int-expression slice 相同；AST 仍提供选中 run identity 和 handler 状态，物理方法仍可查询。

### 3. 仅在整组证明后提交源码投影

所有常量及既有 group gates 全部通过后，class-source 才输出 `java.lang.String...` 和带引号、安全转义的常量实参。复用现有 Java string literal 拼写函数，不得从 constant-pool 原始字节直接拼接源码。保留构造器字段赋值与其他用户可见方法。遇到任何未证明别名、额外数组操作、非 literal、重载、元数据不一致、suffix、读取不完整、预算停止或取消，都保留物理 fallback 与诊断，不发布部分常量。`ProvedEnumIntArgument` 不向此局部 union 之外泛化。

### 4. 使用冻结三方对照验收

对冻结源、原始 class、JADX 源和修后 Jarde 源执行 Java 8 `javac` 和 `java -Xverify:all`。断言字符串内容、顺序、数组身份，并用 JVM 可验证的 flag/Signature、额外字段/consumer、错误数组类型、重复 index、非 ASCII 和无效 suffix 负例检查原子拒绝；覆盖低预算、取消和既有 literal/int enum。参考 JADX `TestEnums4` 的覆盖形状，但保留更强的整组与效果证明；不复制 JADX 代码，不增加依赖。

## Risks / Trade-offs

- [Varargs 源码可能意外共享数组] → 只接受每个常量独立新建、只消费一次的数组；运行对照检查不同身份。
- [构造器 Signature 缺失或与 descriptor/flag 不一致] → 拒绝整组，不只凭 descriptor 推断源码参数。
- [通用 AST 不能表示初始化数组序列] → 保留物理 AST/report，并以精确完整的同次 raw Code 前缀作为权威证据。
- [元素数量增加证明成本] → 以 64 项上限和既有 counted budget 控制；停止状态与拒绝状态继续区分。
