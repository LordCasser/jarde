# varargs 内联数组于构造实参位巡查（2026-10-02）

泛型/通配符域巡查（`List<? extends>`/`Class<?>` 传参、通配符捕获全部健康）引出的构造呈现缺口（主线 `ee35101b`）。固定转录 [fixture](fixture/)（W1 复合 / W3 判别 / W4 对照；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)）。

## 结果矩阵

| 场景 | 主线 Jarde |
| --- | --- |
| 裸 `Arrays.asList(1,2)`（return/赋值/拼接消费）、`String.format` varargs | 恢复——内联数组已按 `new Integer[]{…}` 初始器呈现 |
| `nameOf(W1.class)`（类字面量实参）、`new ArrayList<>(of(1,2))`（普通调用作拷贝构造实参） | 恢复 |
| 嵌套构造作实参（前切片已闭合） | 恢复 |
| **`new ArrayList<>(Arrays.asList(1,2,3))`——varargs 内联匿名数组存储链位于构造实参位** | 拒绝：`jre_new_interleaved_effect`（anewarray+dup/index/box/aastore 交错于外层 new-dup 与 ctor 之间）→ 级联局部声明拒绝（W1.use 的 `sumExt(ints)` 链随之断裂） |

## 根因

`init.rs` 构造站点走查（nested-ctor-argument-sites 切片已接受实参位的**完整嵌套构造**）未建模实参位的**内联匿名数组存储链**（varargs 降低：`anewarray; [dup; index; box…; aastore]×n`）——被当作外层构造的交错效果拒绝。高频形态：`new ArrayList<>(Arrays.asList(…))`、`new HashSet<>(Arrays.asList(…))`、任何集合拷贝构造接 varargs 工厂。

## 处置方向

`recover-varargs-array-ctor-args`：走查接受实参位的内联匿名数组链（元素链与既有 varargs 内联数组证明判据同源——W4 显示该判据在调用位已建）；呈现复用 `new T[]{…}` 初始器拼写于实参位。W3 两形/W1.use 恢复；裸调用位与嵌套构造位逐字不变；链不完整/双用途保持拒绝。

原 class 为行为基准。

## 实施结果（`recover-varargs-array-ctor-args`）

走查已扩展：构造区间扫描遇数组分配起始的完整链时经 `ArrayInitializers::inline_argument_chain_bcis` 复用既有证明（consumer 为实参依赖走查到达的 invoke、源区间封闭于 `(dup, ctor)`、单用途），空 varargs 裸分配同判据；接受集进入区间与异常边界检查（`jre_new_inline_array_exception_boundary`）。W3 两形/W1.use 及变体前后、三方对照与门禁记录见 [results-vca/varargs-replay.md](results-vca/varargs-replay.md)（变体源与 SHA 在 [variants-vca/](variants-vca/)）。跨类型 LUB 混合装箱（`Number[]` 组件）与裸位同口径拒绝（`array@1` 元素兼容性边界，独立切片面）；`String.<init>([C)V` 与直接数组实参形态保持既有边界。
