# Java `boolean[]` 的 raw-int `bastore` 低位语义

日期：2026-09-25。此目录记录一项只读语法审计，为下一项 OpenSpec 规划提供 JVM 证据。没有修改生产代码、测试、已有 OpenSpec change、roadmap 或任务。

## 结论

当前 Jarde 对 verifier-valid 的 `boolean[]` `bastore` 写入仍有明确拒绝缺口。JVM 写入时只消费 int 值的最低位：偶数读回 `false`，奇数读回 `true`，负数同样适用。当前 `array_write` 能根据已证明的 `[Z` 数组把 `bastore` 识别为 boolean 元素写入，但当写入值尚无 boolean 证明时，会在真实 store BCI 拒绝。紧随其后的 `baload` 和 `ireturn Z` 则能生成正确的 boolean 返回读取。

该位置可以直接复用 `crates/jarde-java/src/build.rs:12912-12932` 已有的 `integer_low_bit_boolean(value, bci)`。它构造 `value % 2 != 0`，并把转换作为消费位置拥有的子树。对任意 Java `int`，包括 `Integer.MIN_VALUE`，除数 2 非零且不会溢出；余数是否非零与最低位是否为 1 完全一致。这里无需新 AST 节点、位运算规则、通用转换管线或第二套来源图。

下一项 OpenSpec 的最小准入建议：真实 opcode 必须是 `bastore`（`0x54`），数组自身事实必须证明元素类型为 `boolean`，且值的呈现类型必须是 `byte`、`char`、`short` 或 `int`。已有 boolean 证明和 boolean literal 继续走当前 `boolean_spelling`。未知数组元素类型不得从 `bastore` 猜成 boolean；值是未知、引用、`long`、`float` 或 `double` 时仍拒绝。转换只包住 store 的值，保留 array/index/value 的单次求值和 store BCI 来源。现有 `array_store_opcode_matches`（`build.rs:3156-3166`）已经区分 `[Z`/`[B` 的相同 opcode 所代表的两种元素事实。

## 冻结样本与复现

`fixtures/RawBool.java` 和 `fixtures/Order.java` 是普通 Java 源码，先用 `javac --release 8 -g:none` 编译成 `int[]` 形状。`patch_classfiles.py` 只替换目标方法 descriptor 里同长度的 `[I` / `I` 类型标记，并仅在 `put` 方法 Code attribute 内将唯一的 `iastore` (`0x4f`) 改为 `bastore` (`0x54`)，将唯一的 `iaload` (`0x2e`) 改为 `baload` (`0x33`)。Code 长度不变；patch JSON 记录原始与 patched SHA-256、descriptor 改动数、Code hex 与 opcode 偏移。

在仓库根目录运行以下命令可把重新生成的全部输出写到调用者指定的目录。脚本只在 `--out` 下创建类文件、patch JSON、JVM/JADX/Jarde/Javac/Javap 日志与汇总 JSON；所需二进制通过参数提供。

```sh
CARGO_TARGET_DIR=/tmp/jarde-bastore-audit-target cargo build --locked -p jarde-cli
python3 openspec/evidence/java-syntax-2026-09-25/boolean-array-lowbit/reproduce.py \
  --out /tmp/boolean-array-lowbit-reproduced \
  --jarde-cli /tmp/jarde-bastore-audit-target/debug/jarde-cli \
  --jadx /opt/homebrew/bin/jadx
cargo clean --target-dir /tmp/jarde-bastore-audit-target
```

基线流水线曾在私有 `/tmp/jarde-bastore-evidence.aBYQFV/out` 执行，复制出的 `reproduced/` 文件未再加工；私有构建目录已删除。`reproduced/summary.json` 和各阶段 JSON 日志保留那次实际运行的命令、stdout/stderr、绝对临时路径与状态，类文件哈希可由归档副本复核。类 SHA-256：

| fixture | 原始 Java 8 class | patched class |
| --- | --- | --- |
| `RawBool` | `e38d0e46a055d5183001d2e51ed329c2a0f121409f27ead257d34f6ab76317e3` | `32286d138ace6a328a8c5ba7d0a0d66fadb4bf9dbbab79385e6dc3a01dcd6c2b` |
| `Order` | `63dc18e93763ffbf9d6ce73a299a7d140023eb278dd2d430f6e47d208bb55ef0` | `31d8fc3394c39cfe9a9e62da435af9245d2f26cd24e62e077e7310e1bf97d414` |

输入 class 均为 major version 52，原始/patch 后大小分别为 172/172 B 与 532/532 B。patched `RawBool.put` descriptor 为 `([ZII)Z`，Code 长度 8，opcode 偏移 3 和 6 改动；patched `Order.put` descriptor 为 `([ZI)Z`，Code 长度 16，opcode 偏移 11 和 14 改动。具体证据见 `reproduced/patches/*.json` 和 `reproduced/logs/javap-*.json`。

基线工具是 OpenJDK/Javac 23.0.1、JADX 1.5.6 和 Jarde CLI 0.1.0。Jarde CLI SHA-256 为 `f99024d22e000d2e28cb1f8d95acc0880bdfea2445343f38b476d04b7d115273`。实际执行命令及完整 stdout/stderr 保存在 `reproduced/logs/*.json`，摘要在 `reproduced/summary.json`。

## JVM 结果

两个 patched class 都通过 `java -Xverify:all`。`RawBoolRunner` 用反射调用 descriptor 为 `([ZII)Z` 的方法，覆盖原始 int 的边界值：

```text
0 -> false / false
1 -> true / true
2 -> false / false
3 -> true / true
-1 -> true / true
-2 -> false / false
-2147483648 -> false / false
2147483647 -> true / true
```

左侧副作用样本 `Order.put` 按 `array(array)[index()] = value(input)` 形状编译，再对数组 descriptor 和 store/load opcode 作上述受控补丁。三个 helper 依序把 `1`、`2`、`3` 写入 trace。`OrderRunner` 结果：

```text
success: return=false, trace=123
null array: throws=NullPointerException, trace=123
out of bounds: throws=ArrayIndexOutOfBoundsException, trace=123
value throws: throws=IllegalStateException, trace=123
```

这证明 JVM 先执行数组表达式，再执行下标，再执行值表达式；即使数组为 null、索引越界或值表达式随后抛错，三个 producer 均已按该顺序各执行一次。值表达式本身先抛错时，store 的 null/bounds 检查尚未发生。JVM 的 NPE/AIOOBE 文案不作为断言，只记录异常类型、trace 和读回值。

## JADX 与 Jarde 阶段对照

JADX 命令对两个 patched class 均成功反编译，但输出未经转换的 int-to-boolean 赋值：`RawBool` 是 `zArr[i] = i2;`，`Order` 是 `array(zArr)[index()] = value(i);`。两份输出各自以 `javac --release 8 -g:none` 编译均返回 1，错误均为 int 不能转换成 boolean。JADX 在 `TypeUpdate.arrayPutListener`（`/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/typeinference/TypeUpdate.java:609-635`）将数组组件类型传播到写入值；`InsnGen` 的 APUT 分支（同仓库 `jadx-core/src/main/java/jadx/core/codegen/InsnGen.java:470-476`）仍直接输出赋值，没有表达 JVM 对 boolean 元素的最低位转换。

Jarde 对两个 class 的 `class-source --policy single-class --evidence all` 均返回进程状态 0，但方法报告带拒绝 marker，不能视作完整恢复：

- `RawBool.put` 在 `bastore` BCI 3 输出 `@bytecode 3`，理由是值没有 boolean 证明；`return arg0[arg1]` 正确呈现为 boolean 数组读取。
- `Order.put` 在 `bastore` BCI 11 拒绝，并输出 `@bytecode 11 1 4 8`，保留 store 和三个 producer 的来源；同方法里的 `array`、`index`、`value` helper 均分别恢复。

当前 `array_write` 在 `build.rs:10223-10280` 按 array、index、value 取得呈现式；`build.rs:10282-10307` 的 `Some(Type::Boolean)` 分支只接纳 boolean literal 或 `boolean_proven`，其余值 fallback。若在该 boolean consumer 分支为受证明的 B/C/S/I 值调用既有低位 helper，生成的 `IndexAssign` 会自然按 Java 左侧数组、下标、值的顺序求值；低位计算不抛异常、不复制或提前执行 producer。array、index、value 和 store 来源可以继续走原来的 `render_value` / `quoted_bcis` / `IndexAssign` 通道。

## 明确边界

这项证据只支持已证明的 `boolean[]` 对实际 `bastore` 的低位写入和同方法 `baload` 读回。它不授权用 opcode 0x54 区分 `[Z` 与 `[B`，不推导局部变量/phi 的 boolean 类型，不处理缺少数组元素证明的输入，也不扩展至 `ireturn Z`、字段写入、`castore`、`sastore` 或其它转换位置。`recover-narrow-array-stores` 已把 Z 低位列作分离边界；`recover-boolean-field-stores` 也把 boolean[] `bastore` 列为非目标。本案建议作为一项独立、消费位置授权的 OpenSpec 闭环。
