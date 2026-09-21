# verification — fix-descriptor-slot-facts

实现提交：`138524a`（"fix: parse a descriptor once, and let an array occupy one slot"）。

## 反例（修前，本机复现）

```java
// javac --release 8 -g:none；源码 static int f(long[] xs, int n) { return n; }
static int f(long[] arg0, int arg2) {   // 签名
    return arg1;                         // 正文
}
```
`javac` 报 `找不到符号: 变量 arg1`；`int[]` 对照正常。独立复核（`9a2f4ce`）与主 Agent 复现一致。

## 修复形状

- `jarde-reader`：`descriptor_facts` / `DescriptorFacts` / `DescriptorComponent::{slots, dimensions, object_name, span}` / `DescriptorCursor`（一次解析；数组恒 1 槽、`J`/`D` 2 槽、其余 1 槽）；`descriptor_types` 保留为投影，建在事实之上。
- `jarde-jvm`：`parameter_positions` 从事实计算参数槽位；frame 的字节级 `parse_field_type`/`parse_method_descriptor` 删除；`MethodDeclaration::parameter_slots` 走事实。
- `jarde-java`：`type_of_component` 成为唯一的 descriptor→Java 拼写；lambda/build/facts 改用它。
- `src/class_source.rs`：删除自有 `descriptor_type`/`method_descriptor`，签名与正文使用同一套槽位。

## 证据

| 项 | 结果 |
| --- | --- |
| reader 单测（数组/宽基本类型/参数向量/游标/畸形） | 通过 |
| java 层 `parameter_types`（`(Z[JI)J` static `{0,1,2}` / instance `{1,2,3}`、`(D[[DI)J`） | 通过 |
| `tests/p3_parameter_slots.rs`（真实 fixture `p3-parameter-slots/`，声明与正文一致、receiver 未回退） | 2 passed（另 1 个 JDK 用例 `#[ignore]`） |
| `tests/p3_execution_comparison.rs`（单方法 + 批量两个入口，新增参数槽样本） | 3 passed（`--ignored`） |
| 全量门禁 | `cargo test --workspace --all-targets --all-features --locked` = 1494 passed / 0 failed；clippy 0 warning |
| 反例（把数组槽宽改回元素宽度） | 文本用例、fixture 一致性、JDK 对照三处同时变红；还原后全绿 |
| 反例（只把 `class_source` 的拼写改回） | 文本用例与 JDK 对照（`cannot find symbol` / `incompatible types` 共 7 处）变红；还原后全绿 |

## 边界

- 测试侧的独立 oracle（`frame_oracle`、`java::oracle`、harness 的 `descriptor_parts`）**故意**保留自己的读法，作为第二意见；它们不是实现分叉。
- query 层的 Signature 语法（JVMS 4.7.9.1）是另一种产生式，其 descriptor 读取已走 `descriptor_types`。
- `lambda.rs`/`accessor.rs`/`bridge.rs`/`concat.rs` 各有一份私有 `source_name`（`/`→`.`）属既有重复，与本修复无关，未合并。
