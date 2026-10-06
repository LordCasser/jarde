# 零回退差分：全语料渲染 修前 vs 修后（2026-10-06）

本片（`recover-twr-javac8-close-sequence`，方向 A）落在 `crates/jarde-java/src/guard.rs` 的
新读法上。为了把"零回退"从"相关测试全绿"提升为**全语料逐条差分**，用同一份语料（`tests/fixtures`
+ `openspec/evidence` 下全部 `.class`，**2678** 个文件）跑两份 `jarde-cli class-source --policy
single-class --format text`，逐字节比对（仅归一化 `elapsed_millis` 计时字段）：

```text
cargo build -p jarde-cli --locked          # HEAD（修前）→ /tmp/twr2/jarde-cli-base
cargo build -p jarde-cli --locked          # 工作树（修后）→ /tmp/twr2/jarde-cli-new
for class in $(find tests/fixtures openspec/evidence -name '*.class'); do
  diff <($BASE class-source --input $class --policy single-class --class $(basename $class .class) --format text 2>&1) \
       <($NEW  class-source --input $class --policy single-class --class $(basename $class .class) --format text 2>&1)
done
```

结果（2678 条比对）：

| 分类 | 数量 | 说明 |
| --- | --- | --- |
| 输出逐字节相同 | 2669 | 其中 1978 条是完整类渲染（`--class` 解析成功），700 条两侧同为同名/作用域错误输出（文件路径不含包名时 `--class <文件干名>` 无法解析，两侧一致） |
| 仅 `elapsed_millis` 不同 | 8 | 计时字段，非行为差异 |
| **行为差异** | **1** | `twr-javac8-codegen-patrol/fixture/real-javac8-TR/TR.class` —— 即本片的目标形：`one()` 由"整方法 not recovered + 引注"变为 `try (TR local1 = new TR(arg0)) { … }` |

即：**语料内唯一被本片改变的呈现就是真 javac 8 腿的 TWR 单资源形**；javac 9+ 语料（全部既有
fixture，含 CF-17 两族）逐字节不变。

> 覆盖边界：`--class` 取文件干名，故带包/多定义的 700 条只能比错误输出；它们是"两侧一致"的
> 有效对照，但不构成渲染覆盖。渲染覆盖 1978 条里包含全部 `p3-*`/`finally`/TWR 族 fixture。
