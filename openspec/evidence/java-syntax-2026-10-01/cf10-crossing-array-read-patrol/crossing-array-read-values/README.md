# `recover-crossing-array-read-values` 实现证据（CF-10 登记债务，2026-10-01）

[巡查](../../README.md)追踪到的唯一失败点——跨保护区域局部写值白名单不含数组元素读——在本片闭合：
`presented_int_store_value` 新增同块元素读臂，`F2` 完整恢复且行为一致（`14`），`F1` 对照逐字不变。
本文记录失败点复证、变体前后行为、恢复输出与 SHA。基线一律为分片前主线 `16ddef5b` 的产物。

## 失败点复证（任务 1.1）

fixture SHA 与巡查一致（`../../results/fixture-sha256.txt`，本片复核通过）。基线
`results/F2.before.java` 复现整方法拒绝：

```
// local 1 crosses a protected region, but SSA does not prove that every path to its reads reaches a presented write
```

读侧 phi 链闭合的证据：本片唯一代码变更是写值白名单（`build.rs` 的 `presented_int_store_value`
臂 + `presented_array_operand`），界门与读侧 walk 零改动，`F2` 即恢复——若读侧链未闭合，
仅放宽写值形状不可能翻转载决。`results/F1.before.java` 与巡查 `../../results/F1.jarde.java`
逐字节一致（恢复、`9`/`3915`），无保护区域对照路径未被触及。

附带复证（探针法，已移除）：`double[]` 累计在白名单放行后仍被拒，根因是类型决策而非白名单——
`written_type` 经 `array_of_value` 元素读臂返回 `(Double, 0)`，`array_spelling` 零维拼写为
`Type::Reference("double")`，与帧类型的 `Double` 在冲突检查中判 `ConflictingWrites → Unknown`，
`store_type_is_proven=false` 直接短路（`int[]` 不受影响仅因 `iaload` 不在 `array_of_value`
的匹配里）。本片将 `array_spelling` 零维改为元素自身，并给
`cross_exception_store_type_is_proven` 补上 Long/Float/Double（与 Int 同答：`iinc` 不可能落
在类别 2 槽位）。

## 实现（任务 2.1，design 决策 1）

`crates/jarde-java/src/build.rs`：

1. `presented_int_store_value` 新元素读臂：`Operation::ArrayLoad`（iaload）与
   `Operation::ArrayElementLoad`（laload/faload/daload/aaload/baload/caload/saload，以解码层
   实际形态为准），操作数须为 2；入口类型门放宽为数值四类，`Value::Ref` 仅在元素读上放行
   （裸引用 copy 的 store 仍拒）。
2. 数组操作数（深度 0）走新 `presented_array_operand`：同块、单用途
   （`single_use_at_with_budget`）、区域可呈现、`Operation::Load` 的引用局部；共享 `seen` 集
   合，故首生产者到 store 的区间封闭检查天然把 aload 计入表达式树。下标与其余操作数递归过
   既有白名单。
3. 既有各臂行为保持：Push 仍仅常量字面量（扩为数值四类）、Load/Add 数值类别化、Invoke 仍
   限保护臂内返回 `Int` 的静态调用；区间封闭、单用途、非 fallback 区域检查不变。

引用数组元素类型走 `array_of_value` 既有答案（`lastRef` 的 `String` 即其零维拼写）。

## 变体族（任务 1.2，fixture `Cf10Values`/`Cf10Refused`/`Cf10Nested`）

原类 `java -Xverify:all` 输出：`Cf10Values` = `14 8.0 f 15 12`（`results/Cf10Values.orig.out`），
`Cf10Refused` = `10 9 17`，`Cf10Nested` = `14 14`。前后行为：

| 方法 | 基线 `16ddef5b` | 本片 |
| --- | --- | --- |
| `intSum`（F2 同形：加法树含 iaload、体内 try/catch + iinc） | 整方法 quote（crossing 拒绝） | 恢复；`sum = sum + data[i];` + catch `sum = sum - 1;` |
| `doubleSum`（double 累计，catch 内 daload 写） | 整方法 quote（类型决策 `ConflictingWrites` 短路） | 恢复；零维拼写修复后同形呈现 |
| `lastRef`（`String[]` 元素累计，aaload 三处写） | 整方法 quote | 恢复；`cur = parts[i];` / `cur = parts[0];` |
| `catchMultiWrite`（catch 内两次写：daload 直写 + 加法树） | 整方法 quote | 恢复；两写按序呈现 |
| `twoTries`（一体内两个保护区域，跨越两区） | 整方法 quote | 恢复；两 try 均呈现 |
| `shared`（`pick = sum = data[i]`，元素值经 dup 共享） | quote | 逐字一致（单用途纪律拒绝） |
| `crossBlock`（写值树叶为三元 join 的 phi） | quote（`crosses a quoted fallback region`） | 逐字一致 |
| `intervalEffect`（`sum + (k = 2) + data[i]`，生产者区间插 store） | quote | 逐字一致（单用途 + 区间封闭双重拒绝） |
| `Cf10Nested.inLoop`（体内嵌套 try：handler 二次入环） | quote（`the graph is not reducible over 4 block(s) …`） | 逐字一致——区域/图构建层既有缺口，非白名单，按 design Non-Goals 不动 |
| `Cf10Nested.aroundLoop`（try 包住循环） | quote（`crosses a quoted fallback region`） | 逐字一致——同上 |

`Cf10Values` 整类恢复后经 `javac --release 8` 重编、`java -Xverify:all` 运行，输出与原类逐行
一致（`14`/`8.0`/`f`/`15`/`12`，catch 路径均由输入真实触发）；`F2` 重编运行输出 `14`。固定
JADX（`jadx -d`，无 `--release`）对 `F2`/`Cf10Values` 的参照呈同一源形（`sum += data[i];` +
try/catch），语义基准仍以原 class 运行为准。

## 门禁与登记（任务 3.1/3.2）

* `cargo test --workspace --tests --locked --no-fail-fast`：全绿——2748 通过 / 0 失败 / 272
  套件（基线 2743 + 本片 5），exit 0。
* `cargo fmt --all -- --check`：通过。
* `cargo clippy`（CI `ci.yml` 完整 30 项 `-A` 清单 + `-D warnings`）：通过。
* `openspec validate --all --strict`：231 项全过。
* 新增 `tests/p3_crossing_array_read_values.rs`：F1 逐字断言、F2/变体恢复与重编运行、负例与
  嵌套边界逐字断言、饥饿预算不声称 Complete。
* 巡查 README 附带的 enumswitch 噪声观察：本片全部捕获产物中出现 0 次
  `jre_enumswitch_shape`（该观察的形态——非静态字段数组读——不在本片 fixture），无需登记。

SHA 见 [results/sha256.txt](results/sha256.txt)；`Cf10Refused`/`Cf10Nested` 的
`before`/`after` 逐字节同哈希，即负例与边界零移动的直接证据。
