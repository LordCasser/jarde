# Root 独立验收（2026-10-07，合并）

## 判据逐项

1. **门控复核**：`results/01-gating-refusal-point.md` —— 两位拒绝机制不同（`MD.partSet`=提交走查的 blanket skip"子数组存储独占消费不可独立"；`MD3.bareIdx2`/`MD2.retPos`=消费者门读到的是下标 `iconst_0` 而非值自身单用 `iaload`）；判别变量=每个 javac 子 store 的数组操作数是 `dup` 输出而锚的是字段读。
2. **diff 审查**：`build.rs` +145/−5——`array_initializer_reader`（reader=值自身单用，中间 run=reader 其它操作数，用元素值既有的同一表达式分类判定）；消费者门增下标/长度接收者臂；提交走查只跳子几何（`store_writes_into_a_constructed_array`）。拒绝文本/所有权集/引注走查/postfix 记账检查逐字未动。
3. **root 实测**：MD（巡查冻结 jar）0 引注 ✓；MD2/MD3/AV（本片冻结 fixture，jar+自述头）全部 0 引注 ✓；定向 5+1 ignored 回放绿（双腿编译+`-Xverify:all`，MD `21/9/9/10` 等）；oracle ignored 3/3 ✓。
4. **范围裁定**：实现片按冻结 spec 的**通用单读者判据**实现（`new int[]{9}[i]`/`[i+1]`/`.length` 均恢复），较窄常量下标变体已测量并弃用——**追认**（spec 如文所写；窄形无安全收益）。
5. **corpus 移动裁定**：moved=24——本片 11 + 巡查 `md.jar!MD` + **12 个他族 driver 的非末实参位恢复**（巡查 `argPos` fixture 恰用末实参故读作已呈现；`bsearch(new int[]{1,3,5,7}, 5)` 形现准入）——**追认为机制内外沿**（与 ladder 同级 switch、postfix 矩阵外消费位同规：判据只证 dance 自身，消费位泛化同判据外推），12 个全部双腿重编+`-Xverify:all` 逐个行为回放一致（`04-corpus-delta-replay.sh`）；loop-else-if 套件 doc 注释同步（无断言改动）。
6. **门禁（权威口径）**：全量 exit 0、**328 targets ok、0 FAILED 行**；fmt 0；CI 逐字 clippy `Finished` 0；openspec **306/306**；census `(811,3445,290,2145,8)→(822,3564,290,2161,8)`，指纹 +16 纯增。
7. **CI**：合并推送后 run 为准（监控在案）。

## 残余

- "已提交 dance 的读者语句自身拒绝"路径（`deferred_producers` `at_allocation` 臂）从代码论证、无输入可达（IR 先拒手构形）——记账检查与值级守卫未动、套件绿；
- `AVN` 两手构负例（双读者/弃消费）与父提交逐字节同拒。
