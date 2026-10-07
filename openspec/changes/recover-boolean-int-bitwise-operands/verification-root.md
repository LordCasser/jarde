# Root 独立验收（2026-10-07，合并）

## 判据逐项

1. **门控复核**：`results/01-gating-refusal-point.md` —— 单发出点 `render_value` Bitwise 臂（`Expr::presented==None`，`ast::binary_type` 只认双 bool/双整型）；门控实测冻结 BW 拒绝 2→0、负例 BWN 逐字节。
2. **`mix` 裁定**：实现片以巡查证据（README 把 `andNot`/累积 xor 记为**同一发现的两形**）+ 类级探针（恢复文本双腿 `-Xverify:all` 答案 `true/5/true/2/-2147483648/false/3` 与原类**逐字一致**，含第三值 mix）裁定 `mix` 与 `andNot` 同域一并恢复、不加守卫——**追认**（类级不变量由双向运行证明，正是 conditional-rhs 片要求的硬门）。
3. **diff 审查**：`BooleanConsumption` 消费走查（boolean 变量存/`Z` ireturn/已认领字段写/布尔兄弟的位运算，递归）+ `accumulates_boolean` 定点 + 计数器播种；`bitwise_boolean_operand` 从 conditional-rhs 片的字段写位**泛化**到该走查——且 conditional-rhs 锚逐字节不动（其套件 4+1 绿，走查包含而不移动既有位置）。
4. **root 实测**：定向 5+1 ignored 回放绿（双腿编译+`-Xverify:all`）；oracle ignored 3/3。
5. **门禁（权威口径）**：合并初跑 fmt 报一处换行 diff（root `cargo fmt` 修正入验收提交）+ `export_cli` 计时族 1 失败（本日第三次，单测复跑 ×2 绿）；**修正后全量 exit 0、330 targets ok、0 FAILED**；CI 逐字 clippy `Finished` 0；openspec **307/307**。
6. **corpus**：moved=6（巡查锚+提交副本+BW/BWR 双腿）；BI/RC/RCN/CF/NEG/ICM/ICN 逐字节（sweep 自检）；census `(828,3596,290,2211,8)→(835,3649,290,2273,8)`，指纹 +10 纯增。
7. **CI**：合并推送后 run 为准（监控在案）。

## 残余

- `boolean r = a & !b` 物化入**局部**形保持拒绝；分支测试/比较/调用实参/int 存储消费保持拒绝；stack-Phi 合并/重访值/深度限内保守。
