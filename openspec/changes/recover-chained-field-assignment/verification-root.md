# Root 独立验收（2026-10-07，合并）

## 判据逐项

1. **per-family 门控复核**：`results/01-gating.md` —— CH 链与 dup-store/postfix 同门（copy 门 + `Duplicate` 臂）；String 累积与复合 RMW 是**第二落点**（concat walk 的 `jre_concat_interleaved_effect` → `new@1` 读者门），receiver copy 形状统一二者——各自独立门控成立。
2. **diff 审查**：单一 `FieldCopies` 证明双形状（`Chain{stores,lead,saved}` / `Receiver{read,store}`）插入既有 copy 门；纯源逐 store 写、非纯源由 lead 存 `saved{N}`（既有 bindings 表）、lead 不能提交则链整体拒绝（失败关闭）；`concat.rs` 两处准入均门控在**已证** receiver copy 上（`Field{Read}` 未泛化——静态 String 复合逐字节不变）；复合 op 臂越过 iadd/isub 的扩展以普通赋值形呈现（既有 `+=`/`-=` int 路径零触碰）。
3. **root 实测**：定向 3+1 ignored 回放绿（双腿编译+`-Xverify:all`）；冻结 jar 的 `andUse` 渲染 `this.ok = this.ok & arg1; return this.ok;` 原源码形态；`f` vs `f[x][y]` 面关闭（实现片回放 `9/9/9/14/9/f[x][y]`）。
4. **oracle 新纪律**：`p3_execution_comparison --ignored` **3/3 绿**。
5. **门禁（权威口径）**：fmt 0；clippy 逐字 `Finished` 0；openspec **305/305**；全量 325 ok / **1 FAILED = `export_cli` 计时族**（本日已在 `1bfd29bb` 上分类，本次单测复跑两轮绿——flake 判定三连齐：docs-only 先例红 + 同代码绿 + 本地两轮绿）。
6. **corpus**：moved=8 全分类（本片 fixture + 巡查锚；`CA2` 保持整引=安全形）。

## root 重裁（两处 ask_parent "ruling"——按纪律视为未授权，以下为我独立重裁）

1. **"cross-block conditional-materialised RHS not admitted (ruling (i))"** —— 保守结果安全，但**我以自己复现的证据重裁：该扩展正是下一步**。BI 锚 15 的类级可编译错面在合并后 main 上**实测存活**（root 亲测：冻结 jar 渲染剥离编译后 BI 自带 main 输出 `false/false/false/true` vs 原 `false/false/false/false`——`ok &= x > 0` 在幸存循环内被引注吞掉）。恢复优于守卫扩展（锚 15 本就是已登记第 15 critical 锚）：FieldCopies Receiver 扩条件物化 RHS，`prove_conditional_value` 只读复用（inline-concat 同模式）。**已另立 `recover-conditional-rhs-field-compound` 并即时派发（critical 优先）**。
2. **"BI 记录在案（per your ruling）"** —— 记录处置正确；裁定升级为上述恢复立项 + 本验收记录为权威档案。

## 残余

- 实例 `dup_x1` 链（`this.a = this.b = this.c = 5`）保持拒绝（probe 已记录）；
- BI 锚 15 面移交 `recover-conditional-rhs-field-compound`（见重裁 1）。
